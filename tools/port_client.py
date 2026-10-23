#!/usr/bin/env python3
"""Replay a scenario in the port on a fresh vanilla server, and record its packets.

The port's counterpart of `oracle_client.py capture`: a fresh `vanilla_server.py`, `packet-proxy`
in front of it, and the port joining as `rustcraft`, which sends the oracle client's setup
commands and its scenario-start marker, then replays the scenario's ticks
(src/rustcraft/oracle/client/oracle_game_test.rs). Writes target/oracle/<scenario>/trace.jsonl and
capture.jsonl, to compare with vanilla's:

    python3 tools/port_client.py capture walk --accept-eula
    cargo run -p oracle-diff -- packets --join vanilla/26.3/oracle/walk/capture.jsonl target/oracle/walk/capture.jsonl
    cargo run -p oracle-diff -- dir --strict vanilla/26.3/oracle/walk target/oracle/walk

`--ping` pings the scenario's server like the server list does (`rustcraft --ping`) instead of
joining it, into target/oracle/<scenario>-ping/, to compare with `oracle_client.py capture
<scenario> --ping`'s.

`check-join` does both, gate 1 end to end, and exits 0 when the join phases match:

    python3 tools/port_client.py check-join walk --accept-eula
    python3 tools/port_client.py check-join walk --accept-eula --compression-threshold 256

`soak` joins the port straight to a fresh generated world (no proxy) and teleports the player
around it for a few minutes: the spawn, plains, a forest, a desert, the caves under it, an
ocean, a village and the caves under the spawn. It exits 1 when the client exits early or its
log has a panic or unported code, printing those lines with their counts and the log's end, and
0 otherwise, with the stops it went to (client log: target/soak/client.log):

    flock /tmp/rustcraft-oracle.lock python3 tools/port_client.py soak --preset normal --accept-eula
    flock /tmp/rustcraft-oracle.lock python3 tools/port_client.py soak --preset normal --accept-eula --seed 0 --seconds-per-stop 20

The server's ticks are frozen (`tick freeze`), so no fluid flows into what the soak builds, and
each stop has the player standing on a block it built, in a chunk section with no fluid: the
port's player panics near one, and while falling fast, which isn't what the soak is about.

`stall` joins the port to a fresh flat world, stalls its game thread for 40 seconds once the
level is ready (`rustcraft --stall`), longer than a server waits for a keep-alive's answer, and
exits 0 when the client stayed connected through it and the 20 seconds after, which it does when
its connection's own thread answers the keep-alives meanwhile, like vanilla's Netty thread
(client log: target/stall/client.log):

    flock /tmp/rustcraft-oracle.lock python3 tools/port_client.py stall --accept-eula

`RUSTCRAFT_BINARY=path` runs that build of the port instead of building the tree's, so a long check
can run on a build while the tree changes.

Wrap it in `flock /tmp/rustcraft-oracle.lock` on a shared machine: the server and the proxy use
fixed ports. `--compression-threshold` turns the server's compression on, which captures don't
show (the proxy records packets inflated) but for the server's `login_compression`. So a run with
compression is compared with vanilla's run with the same threshold, in
vanilla/<version>/oracle-compressed-<threshold>/, which `check-join` records with
`oracle_client.py capture` when it isn't there yet.
"""

import argparse
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import oracle_client  # noqa: E402
import vanilla_server  # noqa: E402
from oracle_client import PROXY_PORT, SERVER_PORT, VERSION_DIR, resolve_scenario, wait_for_line  # noqa: E402
from vanilla_server import OP_NAME  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
PORT_BINARY = Path(os.environ.get("RUSTCRAFT_BINARY", ROOT / "target" / "release" / "rustcraft")).resolve()

SOAK_OUT = ROOT / "target" / "soak"
STALL_OUT = ROOT / "target" / "stall"
SOAK_VIEW_DISTANCE = 4
# The soak's stops after the spawn, where the client joins: (name, where, underground). `where` is a console command
# that finds the place, or an earlier stop and how many chunks east of it, past the chunks the client has around it.
SOAK_STOPS = [
    ("plains", "locate biome minecraft:plains", False),
    ("forest", "locate biome minecraft:forest", False),
    ("desert", "locate biome minecraft:desert", False),
    ("desert caves", ("desert", 2 * SOAK_VIEW_DISTANCE + 2), True),
    ("ocean", "locate biome minecraft:ocean", False),
    ("village", "locate structure #minecraft:village", False),
    ("spawn caves", ("spawn", 0), True),
]
# Blocks -48 to -33: deep in the deepslate, over the lava lakes, among the caves' water and ores.
CAVE_SECTION = -3
# Tags the marker entity that finds a column's surface. Markers are never sent to clients.
SURFACE_MARKER = "rustcraft_soak"
SERVER_LOG_PREFIX = re.compile(r"^\[[^\]]*\] \[[^\]]*\]: ")
LOCATED = re.compile(r"is at \[(-?\d+), (?:~|-?\d+), (-?\d+)\]")
CHUNK = re.compile(r"Chunk at \[(-?\d+), (-?\d+)\]")
ENTITY_POS = re.compile(r"has the following entity data: \[(\S+)d, (\S+)d, (\S+)d\]")
JOINED_AT = re.compile(rf"{OP_NAME}\[[^\]]*\] logged in with entity id \d+ at \((\S+), (\S+), (\S+)\)")
CLIENT_PROBLEM = re.compile(r"panic|isn't ported|aren't ported")
# Rust's panic lines name the thread with its id, which changes from run to run.
THREAD_ID = re.compile(r"^(thread '[^']*') \(\d+\)")
LONG_LINE = 200


def build_port():
    """Builds the tree's port, unless RUSTCRAFT_BINARY names a build to run."""
    if "RUSTCRAFT_BINARY" not in os.environ:
        subprocess.run(["cargo", "build", "--quiet", "--release", "-p", "rustcraft"], cwd=ROOT, check=True)


def has_screenshots(scenario):
    """Whether the scenario takes screenshots, which the port renders."""
    return any("screenshot" in step for step in json.loads(Path(scenario).read_text())["steps"])


def run(scenario, out_dir, server, ping=False):
    """Replays the scenario in the port, joining `server`, or pings it with `ping`."""
    scenario_out = out_dir / (f"{scenario.stem}-ping" if ping else scenario.stem)
    shutil.rmtree(scenario_out, ignore_errors=True)
    scenario_out.mkdir(parents=True)

    command = [
        PORT_BINARY, "--quickPlayMultiplayer", server, "--username", "rustcraft", "--headless",
        "--oracleScenario", scenario, "--oracleOut", out_dir,
    ]
    env = dict(os.environ)
    if ping:
        command = [PORT_BINARY, "--ping", server]
    elif has_screenshots(scenario) and shutil.which("xvfb-run"):
        # Screenshots render offscreen, on Mesa's llvmpipe under a fresh Xvfb like the oracle client's
        # (tests/opengl_frame.rs runs the same way).
        command = ["xvfb-run", "-a", *command]
        env.update(LIBGL_ALWAYS_SOFTWARE="1", GALLIUM_DRIVER="llvmpipe", SDL_VIDEO_DRIVER="x11")
    log_path = scenario_out / "client.log"
    print(f"running {scenario_out.name} in the port (log: {log_path})")
    with open(log_path, "w") as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=600, env=env)
    if result.returncode != 0:
        print("".join(log_path.read_text().splitlines(keepends=True)[-40:]))
        raise SystemExit(f"the port exited with {result.returncode}, see {log_path}")
    return scenario_out


def capture(scenario, out_dir, accept_eula, compression_threshold, ping=False):
    """Replays the scenario against a fresh vanilla server through packet-proxy, adding capture.jsonl to the run."""
    build_port()
    subprocess.run(["cargo", "build", "--quiet", "-p", "packet-proxy"], cwd=ROOT, check=True)
    logs = Path(tempfile.mkdtemp(prefix=f"port-{scenario.stem}-"))
    seed = json.loads(scenario.read_text())["world"]["seed"]

    server_command = [sys.executable, ROOT / "tools" / "vanilla_server.py", "--name", f"port-capture-{scenario.stem}"]
    server_command += ["--port", str(SERVER_PORT), "--seed", str(seed), *(["--accept-eula"] if accept_eula else [])]
    if compression_threshold is not None:
        server_command += ["--compression-threshold", str(compression_threshold)]
    with open(logs / "server.log", "w") as server_log:
        server = subprocess.Popen(server_command, stdout=server_log, stderr=subprocess.STDOUT)
    proxy = None
    try:
        wait_for_line(logs / "server.log", "ready on", server)
        proxy_command = [
            ROOT / "target" / "debug" / "packet-proxy", "--once",
            "--packets", VERSION_DIR / "generated" / "reports" / "packets.json", "--out", logs / "capture.jsonl",
            "--listen", f"127.0.0.1:{PROXY_PORT}", "--upstream", f"127.0.0.1:{SERVER_PORT}",
        ]
        with open(logs / "proxy.log", "w") as proxy_log:
            proxy = subprocess.Popen(proxy_command, stdout=proxy_log, stderr=subprocess.STDOUT)
        wait_for_line(logs / "proxy.log", "capturing to", proxy)

        try:
            scenario_out = run(scenario, out_dir, f"127.0.0.1:{PROXY_PORT}", ping)
        except SystemExit:
            # Why the connection ended is often only in the server's or the proxy's log.
            for name in ("server.log", "proxy.log"):
                print(f"--- the last lines of {name}:")
                print("".join((logs / name).read_text().splitlines(keepends=True)[-30:]))
            raise
        proxy.wait(timeout=30)
    finally:
        if proxy is not None and proxy.poll() is None:
            proxy.terminate()
            proxy.wait(timeout=30)
        server.send_signal(signal.SIGTERM)
        server.wait(timeout=60)

    shutil.move(logs / "capture.jsonl", scenario_out / "capture.jsonl")
    print(f"wrote {scenario_out / 'capture.jsonl'} (server and proxy logs: {logs})")
    return scenario_out


def check_join(scenario, accept_eula, compression_threshold):
    """Gate 1: the port's join phase against vanilla's (`oracle-diff packets --join`), True when they match."""
    if compression_threshold is None:
        reference, out_dir = VERSION_DIR / "oracle", ROOT / "target" / "oracle"
    else:
        reference = VERSION_DIR / f"oracle-compressed-{compression_threshold}"
        out_dir = ROOT / "target" / f"oracle-compressed-{compression_threshold}"
    expected = reference / scenario.stem / "capture.jsonl"
    if not expected.exists():
        if compression_threshold is None:
            raise SystemExit(f"no vanilla capture at {expected}: run oracle_client.py capture {scenario.stem} first")
        oracle_client.capture(scenario, reference, accept_eula, compression_threshold)

    actual = capture(scenario, out_dir, accept_eula, compression_threshold) / "capture.jsonl"
    command = ["cargo", "run", "--quiet", "-p", "oracle-diff", "--", "packets", "--join", expected, actual]
    print(" ".join(str(part.relative_to(ROOT)) if isinstance(part, Path) else part for part in command), flush=True)
    return subprocess.run(command, cwd=ROOT).returncode == 0


class ClientGone(Exception):
    """The port's client exited, or its player left the server, while the soak still had stops to go."""


class Console:
    """A `vanilla_server.py --console`'s console: commands on its stdin, their answers from its log."""

    def __init__(self, server, log_path):
        self.server = server
        self.log_path = log_path

    def run(self, *commands, expect=(), timeout=120):
        """Runs console commands and returns the lines the server printed for them, without their time and thread.

        The server runs its console's commands in order, so the lines before the answer to a `list` sent after them
        are theirs. With `expect`, one of the lines must hold one of its strings.
        """
        offset = self.log_path.stat().st_size
        try:
            self.server.stdin.write("".join(f"{command}\n" for command in [*commands, vanilla_server.BARRIER_COMMAND]))
            self.server.stdin.flush()
        except BrokenPipeError:
            raise SystemExit(f"the server exited before {commands[0]!r}, see {self.log_path}") from None

        deadline = time.monotonic() + timeout
        while True:
            with open(self.log_path, "rb") as log:
                log.seek(offset)
                # The last element is the line the server is still writing, or nothing.
                lines = [SERVER_LOG_PREFIX.sub("", line) for line in log.read().decode(errors="replace").split("\n")[:-1]]
            answered = next((index for index, line in enumerate(lines) if vanilla_server.BARRIER_OUTPUT in line), None)
            if answered is not None:
                break
            if self.server.poll() is not None:
                raise SystemExit(f"the server exited while running {commands[0]!r}, see {self.log_path}")
            if time.monotonic() > deadline:
                raise SystemExit(f"the server didn't answer {commands[0]!r} in {timeout} s, see {self.log_path}")
            time.sleep(0.2)

        answer = lines[:answered]
        failed = any(error in line for line in answer for error in vanilla_server.COMMAND_ERRORS)
        if failed or (expect and not any(text in line for line in answer for text in expect)):
            raise SystemExit("the server's answer to " + "; ".join(commands) + ":\n" + "\n".join(answer))
        return answer


def check_client(client, server_log):
    """Raises ClientGone when the client has exited or its player has left the server."""
    if client.poll() is not None:
        raise ClientGone(f"the client exited with {client.returncode}")
    if f"{OP_NAME} left the game" in server_log.read_text(errors="replace"):
        raise ClientGone("the player left the server")


def stay(client, server_log, seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        check_client(client, server_log)
        time.sleep(0.5)
    check_client(client, server_log)


def first_match(pattern, lines):
    return next((match for line in lines if (match := pattern.search(line))), None)


def chunk_middle(x, z):
    """The middle of the chunk holding x and z. A player there has the block around its box, where the port looks for
    fluids, in that chunk too."""
    return (x >> 4 << 4) + 8, (z >> 4 << 4) + 8


def locate(console, command, places):
    """The middle of the chunk a `locate` command finds, or None when it finds nothing.

    A place among the chunks the client got at an earlier stop is looked for again from 512 blocks east of it, a few
    times, so each stop brings the client chunks of its own.
    """
    origin = None
    for _ in range(4):
        positioned = command if origin is None else f"execute positioned {origin[0]} 64 {origin[1]} run {command}"
        match = first_match(LOCATED, console.run(positioned, expect=("The nearest", "Could not find")))
        if match is None:
            return None
        x, z = chunk_middle(int(match[1]), int(match[2]))
        apart = 16 * (2 * SOAK_VIEW_DISTANCE + 1)
        if all(max(abs(x - other[0]), abs(z - other[1])) >= apart for other in places.values()):
            break
        origin = (x + 512, z)
    return x, z


def load_area(console, x, z, client, server_log):
    """Force loads the chunks the server sends around (x, z), waiting until they're generated.

    The client then gets the player's chunk right after a teleport there: in one it hasn't got, the player sinks at
    0.1 a tick (`LivingEntity.travelInAir`), and would end up in its platform. The previous stop's chunks stop being
    forced, the player keeping them loaded while it's there.
    """
    radius = SOAK_VIEW_DISTANCE * 16
    console.run("forceload remove all")
    console.run(f"forceload add {x - radius} {z - radius} {x + radius} {z + radius}", expect=("to be force loaded",))
    checks = [(x, z), (x - radius, z - radius), (x + radius, z - radius), (x - radius, z + radius), (x + radius, z + radius)]
    deadline = time.monotonic() + 600
    while True:
        answer = console.run(*(f"execute if loaded {check_x} 0 {check_z}" for check_x, check_z in checks))
        if sum("Test passed" in line for line in answer) == len(checks):
            return
        if client is not None:
            check_client(client, server_log)
        if time.monotonic() > deadline:
            raise SystemExit(f"the chunks around {x}, {z} weren't generated in 10 minutes")
        time.sleep(1)


def surface(console, x, z):
    """The height of the column's surface: the first block above its ground or water, leaves not counting."""
    marker = f"@e[type=minecraft:marker,tag={SURFACE_MARKER},limit=1]"
    answer = console.run(
        f'execute positioned {x} 0 {z} positioned over motion_blocking_no_leaves run summon minecraft:marker ~ ~ ~ {{Tags:["{SURFACE_MARKER}"]}}',
        f"data get entity {marker} Pos",
        f"kill {marker}",
        expect=("has the following entity data",),
    )
    return int(float(first_match(ENTITY_POS, answer)[2]))


def build_stop(console, x, z, underground):
    """Makes the player a place to stand in the middle of a chunk, returning the height of its feet.

    The port's player panics when a chunk section its box is in holds any fluid (`EntityFluidInteraction.update`
    only gets as far as the sections' `hasFluid`), so the player's section has none: on the surface it's the first
    section above the column's ground, emptied, with a 3x3 stone platform at its bottom; underground it's filled with
    deepslate around a room of 3x3 by 4 high.
    """
    section = CAVE_SECTION if underground else -(-surface(console, x, z) // 16)
    bottom = section * 16
    filled = ("Successfully filled", "No blocks were filled")
    console.run(
        f"fill {x - 8} {bottom} {z - 8} {x + 7} {bottom + 15} {z + 7} minecraft:{'deepslate' if underground else 'air'}",
        expect=filled,
    )
    if underground:
        console.run(f"fill {x - 1} {bottom + 1} {z - 1} {x + 1} {bottom + 4} {z + 1} minecraft:air", expect=filled)
    else:
        console.run(f"fill {x - 1} {bottom} {z - 1} {x + 1} {bottom} {z + 1} minecraft:stone", expect=filled)
    return bottom + 1


def player_position(console):
    """Where the server has the player, and whether it's on the ground, from what the client last sent."""
    answer = console.run(f"data get entity {OP_NAME} Pos", f"data get entity {OP_NAME} OnGround")
    match = first_match(ENTITY_POS, answer)
    position = None if match is None else tuple(float(value) for value in match.groups())
    on_ground = any(line.endswith("has the following entity data: 1b") for line in answer)
    return position, on_ground


def tour(console, client, server_log, spawn, seconds_per_stop, visited, skipped):
    """Takes the player to every stop from the spawn, where it joined, staying `seconds_per_stop` at each."""
    places = {"spawn": spawn}
    for name, where, underground in [("spawn", None, False), *SOAK_STOPS]:
        if name == "spawn":
            x, z, feet = spawn
        else:
            if isinstance(where, str):
                found = locate(console, where, places)
                if found is None:
                    skipped.append((name, f"`{where}` found none"))
                    continue
                x, z = found
            else:
                origin, chunks_east = where
                if origin not in places:
                    skipped.append((name, f"there was no {origin} stop"))
                    continue
                x, z = places[origin][0] + 16 * chunks_east, places[origin][1]
            print(f"{name}: building the stop at {x}, {z}", flush=True)
            load_area(console, x, z, client, server_log)
            feet = build_stop(console, x, z, underground)
            # Console commands run after the level's tick, which sends the blocks they changed: let it go by, so the
            # client has the stop as built when the teleport comes.
            time.sleep(1)
            check_client(client, server_log)
            # A block up, so a chunk that comes a few ticks after the teleport still has the player land on the platform.
            answer = console.run(f"tp {OP_NAME} {x + 0.5} {feet + 1} {z + 0.5}", expect=("Teleported", "No entity was found"))
            if not any("Teleported" in line for line in answer):
                raise ClientGone("the player left the server")
        places[name] = (x, z, feet)

        print(f"{name}: at {x + 0.5}, {feet}, {z + 0.5} for {seconds_per_stop:g} s", flush=True)
        started = time.monotonic()
        stop = {"name": name, "x": x + 0.5, "y": feet, "z": z + 0.5, "seconds": 0, "position": None, "on_ground": False}
        visited.append(stop)
        try:
            stay(client, server_log, seconds_per_stop)
        finally:
            stop["seconds"] = time.monotonic() - started
        stop["position"], stop["on_ground"] = player_position(console)


def stop_process(process, timeout):
    """Stops a process that's still running with SIGTERM, then SIGKILL if it hasn't exited after `timeout` seconds."""
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def stop_server(server):
    """Stops vanilla_server.py, which stops the server with `stop`, killing the server too when it takes too long."""
    if server.poll() is None:
        server.send_signal(signal.SIGTERM)
    try:
        server.wait(timeout=120)
    except subprocess.TimeoutExpired:
        # The server runs in its own session, which a kill of the script doesn't reach.
        for pid in Path(f"/proc/{server.pid}/task/{server.pid}/children").read_text().split():
            os.kill(int(pid), signal.SIGKILL)
        server.kill()
        server.wait()


def soak(preset, seed, seconds_per_stop, accept_eula):
    """Tours a fresh world with the port's client, True when it got through without a panic or unported code."""
    build_port()
    logs = Path(tempfile.mkdtemp(prefix="port-soak-"))
    server_log = logs / "server.log"
    SOAK_OUT.mkdir(parents=True, exist_ok=True)
    client_log = SOAK_OUT / "client.log"
    # So a SIGTERM still stops what the soak started, through the `finally` below.
    signal.signal(signal.SIGTERM, lambda signum, frame: sys.exit(128 + signum))

    server_command = [sys.executable, ROOT / "tools" / "vanilla_server.py", "--name", "port-soak", "--port", str(SERVER_PORT)]
    server_command += ["--seed", str(seed), "--level-type", f"minecraft:{preset}", "--generate-structures"]
    server_command += ["--view-distance", str(SOAK_VIEW_DISTANCE), "--console", *(["--accept-eula"] if accept_eula else [])]
    print(f"starting the server (log: {server_log})", flush=True)
    with open(server_log, "w") as log:
        server = subprocess.Popen(server_command, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
    console = Console(server, server_log)
    client = None
    gone = None
    visited = []
    skipped = []
    started = time.monotonic()
    try:
        wait_for_line(server_log, "ready on", server)
        console.run("tick freeze", expect=("The game is frozen",))
        # The console's answers would go to the op's chat too.
        console.run("gamerule send_command_feedback false", expect=("send_command_feedback",))

        # The player joins on a stop of its own at the world spawn, so it never starts near a fluid either.
        # The console's commands run at the world spawn.
        spawn_chunk = first_match(CHUNK, console.run("forceload query ~ ~", expect=("Chunk at",)))
        x, z = int(spawn_chunk[1]) * 16 + 8, int(spawn_chunk[2]) * 16 + 8
        print(f"spawn: building the stop at {x}, {z}", flush=True)
        load_area(console, x, z, None, server_log)
        feet = build_stop(console, x, z, False)
        console.run(f"setworldspawn {x} {feet} {z}", expect=("Set the world spawn point",))

        command = [PORT_BINARY, "--quickPlayMultiplayer", f"127.0.0.1:{SERVER_PORT}", "--username", OP_NAME, "--headless"]
        print(f"joining in the port (log: {client_log})", flush=True)
        with open(client_log, "w") as log:
            client = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 180
        while f"{OP_NAME} joined the game" not in server_log.read_text(errors="replace"):
            check_client(client, server_log)
            if time.monotonic() > deadline:
                raise SystemExit(f"the port didn't join in 3 minutes, see {client_log}")
            time.sleep(0.5)
        if match := JOINED_AT.search(server_log.read_text(errors="replace")):
            print(f"joined at {match[1]}, {match[2]}, {match[3]}", flush=True)

        tour(console, client, server_log, (x, z, feet), seconds_per_stop, visited, skipped)
    except ClientGone as error:
        gone = str(error)
    finally:
        stop_process(client, 10)
        stop_server(server)

    return report(client_log, server_log, gone, visited, skipped, time.monotonic() - started)


def thread_dump(process):
    """Every thread's backtrace of a running process, through gdb when the machine has it."""
    if process.poll() is not None or not shutil.which("gdb"):
        return "(no thread dump: the process has exited, or there's no gdb)"
    # Attaching to a process that isn't one's child needs ptrace's permission (Yama's ptrace_scope), which sudo has on CI.
    gdb = ["gdb", "-p", str(process.pid), "-batch", "-ex", "thread apply all bt"]
    for command in (gdb, ["sudo", "-n", *gdb]):
        result = subprocess.run(command, capture_output=True, text=True, timeout=120)
        if "Thread 1" in result.stdout:
            return result.stdout
    return result.stdout + result.stderr


def stall(seconds, accept_eula):
    """Stalls the port's game thread on a fresh flat world, True when the client stayed connected through it."""
    build_port()
    logs = Path(tempfile.mkdtemp(prefix="port-stall-"))
    server_log = logs / "server.log"
    STALL_OUT.mkdir(parents=True, exist_ok=True)
    client_log = STALL_OUT / "client.log"
    signal.signal(signal.SIGTERM, lambda signum, frame: sys.exit(128 + signum))

    server_command = [sys.executable, ROOT / "tools" / "vanilla_server.py", "--name", "port-stall", "--port", str(SERVER_PORT)]
    server_command += ["--accept-eula"] if accept_eula else []
    print(f"starting the server (log: {server_log})", flush=True)
    with open(server_log, "w") as log:
        server = subprocess.Popen(server_command, stdout=log, stderr=subprocess.STDOUT)
    client = None
    returncode = None
    try:
        wait_for_line(server_log, "ready on", server)
        command = [PORT_BINARY, "--quickPlayMultiplayer", f"127.0.0.1:{SERVER_PORT}", "--username", OP_NAME, "--headless"]
        command += ["--stall", str(seconds)]
        print(f"joining in the port, which stalls for {seconds} s (log: {client_log})", flush=True)
        with open(client_log, "w") as log:
            client = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=dict(os.environ, RUST_BACKTRACE="1"))
        try:
            returncode = client.wait(timeout=seconds + 180)
        except subprocess.TimeoutExpired:
            print(f"the client hasn't exited after {seconds + 180} s; its threads:", flush=True)
            print(thread_dump(client), flush=True)
    finally:
        stop_process(client, 10)
        stop_server(server)

    lines = client_log.read_text(errors="replace").splitlines()
    print(f"--- the last lines of {client_log}:")
    print("\n".join(shorten(line) for line in lines[-40:]))
    left = [SERVER_LOG_PREFIX.sub("", line) for line in server_log.read_text(errors="replace").splitlines()
            if f"{OP_NAME} lost connection" in line]
    if returncode != 0:
        print(f"--- the last lines of {server_log}:")
        print("".join(server_log.read_text(errors="replace").splitlines(keepends=True)[-30:]))
    for line in left:
        print(f"server: {line}")
    stalled = any(line.startswith("stalled for") for line in lines)
    ok = returncode == 0 and stalled and not any("Timed out" in line for line in left)
    print(f"\nstall: {'stayed connected' if ok else 'FAILED'} (exit {returncode})")
    return ok


def shorten(line):
    """A log line short enough to read: the client logs whole packets on one, which become their type."""
    if len(line) <= LONG_LINE:
        return line
    brace = line.find(" {")
    return f"{line[:brace]} {{…}}" if 0 < brace < LONG_LINE else f"{line[:LONG_LINE]}…"


def report(client_log, server_log, gone, visited, skipped, elapsed):
    """Prints what the soak did and what the client logged, True when it went through cleanly."""
    lines = client_log.read_text(errors="replace").splitlines() if client_log.exists() else []
    problems = Counter()
    others = Counter()
    index = 0
    while index < len(lines):
        # Rust prints a panic's location, then its message on the next line.
        line = THREAD_ID.sub(r"\1", lines[index])
        if "panicked at" in line and index + 1 < len(lines):
            problems[f"{line} {lines[index + 1]}"] += 1
            index += 2
        elif CLIENT_PROBLEM.search(line):
            problems[line] += 1
            index += 1
        else:
            others[shorten(line)] += 1
            index += 1

    print(f"\nsoak: {len(visited)} stops in {elapsed:.0f} s")
    for stop in visited:
        if stop["position"] is None:
            where = "the player wasn't on the server"
        else:
            on_platform = abs(stop["position"][1] - stop["y"]) < 1e-3 and stop["on_ground"]
            where = ", ".join(f"{value:g}" for value in stop["position"])
            where = f"standing at {where}" if on_platform else f"NOT on its platform: at {where}, on the ground: {stop['on_ground']}"
        print(f"  {stop['name']:<12} {stop['x']:g}, {stop['y']}, {stop['z']:g}  {stop['seconds']:.0f} s  {where}")
    for name, reason in skipped:
        print(f"  {name:<12} skipped: {reason}")
    if others:
        print("other lines in the client's log:")
        for line, count in others.most_common(20):
            print(f"  {count:>4}x {line}")
        if len(others) > 20:
            print(f"  ... and {len(others) - 20} more distinct lines")

    if gone is None and not problems:
        print(f"no panic or unported code in {client_log}")
        return True
    if gone is not None:
        stop = visited[-1]["name"] if visited else "join"
        print(f"\nFAILED: {gone}, at {stop}")
        for line in server_log.read_text(errors="replace").splitlines():
            if f"{OP_NAME} lost connection" in line:
                print(f"  server: {SERVER_LOG_PREFIX.sub('', line)}")
    if problems:
        print(f"\nFAILED: panics and unported code in {client_log}:")
        for line, count in problems.most_common():
            print(f"  {count:>4}x {line}")
    print(f"\nthe last 40 lines of {client_log}:")
    print("\n".join(shorten(line) for line in lines[-40:]))
    return False


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    capture_parser = commands.add_parser("capture", help="replay a scenario on a fresh vanilla server and capture its packets")
    capture_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    capture_parser.add_argument("--out", type=Path, default=ROOT / "target" / "oracle", help="output directory (default: target/oracle)")
    capture_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    capture_parser.add_argument("--compression-threshold", type=int, help="turn the server's compression on, from this packet size")
    capture_parser.add_argument("--ping", action="store_true", help="ping the server like the server list does (--ping) instead of joining it")
    check_parser = commands.add_parser("check-join", help="capture the port's run and compare its join phase with vanilla's (gate 1)")
    check_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    check_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    check_parser.add_argument("--compression-threshold", type=int, help="turn the server's compression on, from this packet size")
    soak_parser = commands.add_parser("soak", help="tour a fresh generated world with the port, failing on a panic or unported code")
    soak_parser.add_argument("--preset", choices=("normal", "flat"), default="normal", help="the world's preset (default: normal)")
    soak_parser.add_argument("--seed", type=int, default=0)
    soak_parser.add_argument("--seconds-per-stop", type=float, default=20, help="how long the player stays at each stop (default: 20)")
    soak_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    stall_parser = commands.add_parser("stall", help="stall the port's game thread past a keep-alive's wait, failing if it disconnects")
    stall_parser.add_argument("--seconds", type=float, default=40, help="how long the game thread stalls (default: 40)")
    stall_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    args = parser.parse_args()

    if args.command == "soak":
        if not soak(args.preset, args.seed, args.seconds_per_stop, args.accept_eula):
            raise SystemExit(1)
        return
    if args.command == "stall":
        if not stall(args.seconds, args.accept_eula):
            raise SystemExit(1)
        return
    scenario = resolve_scenario(args.scenario)
    if args.command == "capture":
        capture(scenario, args.out, args.accept_eula, args.compression_threshold, args.ping)
    elif not check_join(scenario, args.accept_eula, args.compression_threshold):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
