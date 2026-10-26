#!/usr/bin/env python3
"""Benchmark the port against vanilla: the same world, the same settings, the same measurements.

A bench (oracle/bench/<name>.json) is a world (seed and level type), the clients' options, a warm-up at the world's
spawn, a path of teleports that loads chunks, and a view held far from the path:

    flock /tmp/rustcraft-oracle.lock python3 tools/bench.py terrain --runs 5 --accept-eula
    flock /tmp/rustcraft-oracle.lock python3 tools/bench.py flat --runs 3 --accept-eula --clients port --perf
    flock /tmp/rustcraft-oracle.lock python3 tools/bench.py terrain --runs 2 --accept-eula --clients port@/tmp/before,port

One vanilla server serves every run: it generates the bench's chunks first (force loaded, then released), with the
server's ticks frozen, the entities gone and a barrier under each spot, so every run sees the same chunks. Then each
client joins it in turn (vanilla, then the port, then the other way round), windowed under its own Xvfb on Mesa's
llvmpipe like the oracle's runs, and the console takes the player along: the world's spawn for `warmup_seconds`, each
spot of the path for its `seconds`, then the hold's spot, where none of the path's chunks are, which the view settles
on for `settle_seconds` and holds for `seconds`. Between runs the player's saved data is deleted, so each run joins at
the world's spawn.

Vanilla is the client jar launched like the launcher launches it (the launcher's default JVM flags for a new
installation, the version's JVM arguments, the launcher's assets), not the Fabric oracle: it's unmodified, measured by
JFR's method tracing (JDK 25, tools/JfrEvents.java), started once the player has joined (`jcmd JFR.start`), so it
doesn't slow the startup. The warm-up gives its JIT the spawn's chunks and frames before anything is measured. Its
`options.txt` is the oracle client's with the bench's options, no inactivity limit and no sound (the port has none).
The port runs its release build with `--bench`, which sets the same options, and `--benchOut`, which records the same
points (src/rustcraft/bench.rs). TESTING.md, "Benchmark", has the definitions; per run:

- startup: the process's start to the server's "joined the game";
- held view: frames a second, each frame's time (one start to the next), its time to the end of its GL calls
  (`GlCommandEncoder.submit`, the port's present), and the render thread's and the whole process's CPU time a frame
  (/proc); with `--perf`, 10 more seconds of the render thread sampled by `perf`, split into the client's own code,
  the driver's (Mesa, its JIT-compiled shaders, X11) and the kernel's;
- loading (the path): frame times while chunks come, section compiles a second and each compile's time
  (`SectionCompiler.compile`), each chunk's handling on the game thread (`handleLevelChunkWithLight`) and the light
  updates (`ClientLevel.update`) per chunk;
- settle: the hold's teleport to the end of its last section compile, and the compiles meanwhile;
- ticks (`Minecraft.tick`), and the resident memory (VmRSS, VmHWM) at the end of the hold.

It prints the median of the runs with their range, for each client, and writes every run's numbers and events into
target/bench/<bench>/. A client `port@BINARY` is the port built at BINARY, to compare two builds on the same world in
one go. With `--capture`, the port first joins once through packet-proxy, stays the warm-up and leaves:
its capture, target/bench/<bench>/capture.jsonl, is what the offline benchmark reads (src/rustcraft/bench.rs's test),
which times the chunks' decoding and handling, their light and every section's compile without a server, and prints a
digest of the meshes.
"""

import argparse
import getpass
import json
import os
import re
import shutil
import signal
import statistics
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import oracle_client  # noqa: E402
import port_client  # noqa: E402
import vanilla_server  # noqa: E402
from oracle_client import PROXY_PORT, SERVER_PORT, VERSION, VERSION_DIR, wait_for_line  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
BENCH_DIR = ROOT / "oracle" / "bench"
OUT_DIR = ROOT / "target" / "bench"
ASSETS_DIR = Path.home() / ".gradle" / "caches" / "fabric-loom" / "assets"
# The Minecraft launcher's default JVM arguments for a new installation.
LAUNCHER_JVM_FLAGS = [
    "-Xmx2G", "-XX:+UnlockExperimentalVMOptions", "-XX:+UseG1GC", "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20", "-XX:MaxGCPauseMillis=50", "-XX:G1HeapRegionSize=32M",
]
# What JFR traces of vanilla (tools/JfrEvents.java names them like the port's events).
TRACED = [
    "net.minecraft.client.Minecraft::runTick",
    "com.mojang.renderpearl.backend.opengl.GlCommandEncoder::submit",
    "net.minecraft.client.Minecraft::tick",
    "net.minecraft.client.renderer.chunk.SectionCompiler::compile",
    "net.minecraft.client.multiplayer.ClientLevel::update",
    "net.minecraft.client.multiplayer.ClientPacketListener::handleLevelChunkWithLight",
]
EVENTS = ("frame", "submit", "tick", "compile", "light", "chunk")
JFC = """<?xml version="1.0" encoding="UTF-8"?>
<configuration version="2.0" label="rustcraft bench">
  <event name="jdk.MethodTrace">
    <setting name="enabled">true</setting>
    <setting name="filter">{filter}</setting>
    <setting name="threshold">0 ms</setting>
    <setting name="stackTrace">false</setting>
  </event>
</configuration>
"""
# The oracle client's options.txt, with these and the bench's own on top.
VANILLA_OPTIONS = {
    "enableVsync": "false",
    "inactivityFpsLimit": '"minimized"',
    "pauseOnLostFocus": "false",
    "soundCategory_master": "0.0",
    "realmsNotifications": "false",
}
JOINED = f"{vanilla_server.OP_NAME} joined the game"
LEFT = f"{vanilla_server.OP_NAME} left the game"
# Chunks force loaded at once, a side: the server generates them in its tick, which a bigger block makes longer.
FORCELOAD_BLOCK = 8
CLK_TCK = os.sysconf("SC_CLK_TCK")
PERF_SECONDS = 10
# Where a sample of the render thread is the driver's: Mesa and what it draws through, and code with no symbol, which is
# llvmpipe's JIT-compiled shaders (vanilla's JIT-compiled Java is named by the JVM's perf map).
DRIVER_DSO = re.compile(r"^(libgallium|libGL|libGLX|libglapi|libGLdispatch|libEGL|libgbm|libdrm|libLLVM|libX|libxcb|libxshm|\[unknown\]|//anon|\[anon)")


def load_bench(name):
    path = Path(name) if name.endswith(".json") else BENCH_DIR / f"{name}.json"
    if not path.exists():
        raise SystemExit(f"no bench {name} (looked for {path})")
    return path.resolve(), json.loads(path.read_text())


def spots(bench):
    """Every spot the player is taken to, the hold's last: (x, y, z, y_rot, x_rot)."""
    return [(*spot["pos"], *spot["look"]) for spot in [*bench["path"], bench["hold"]]]


def block(coordinate):
    return int(coordinate // 1)


def areas(bench):
    """The chunk rectangles the clients see from the spots, a chunk wider: the path's together, and the hold's."""
    distance = bench["options"]["render_distance"] + 2
    path, hold = spots(bench)[:-1], spots(bench)[-1:]
    rectangles = []
    for group in (path, hold):
        xs = [block(x) >> 4 for x, *_ in group]
        zs = [block(z) >> 4 for _, _, z, *_ in group]
        rectangles.append((min(xs) - distance, min(zs) - distance, max(xs) + distance, max(zs) + distance))
    return rectangles


def prepare_world(console, bench):
    """Generates the chunks every run gets, empties them of entities and puts a barrier under each spot.

    Every chunk the clients get was finished, its entities spawned and killed, before any client came.
    """
    console.run("gamerule log_admin_commands false", "gamerule send_command_feedback false", "tick freeze")
    checks = []
    for min_x, min_z, max_x, max_z in areas(bench):
        for from_x in range(min_x, max_x + 1, FORCELOAD_BLOCK):
            for from_z in range(min_z, max_z + 1, FORCELOAD_BLOCK):
                to_x, to_z = min(from_x + FORCELOAD_BLOCK - 1, max_x), min(from_z + FORCELOAD_BLOCK - 1, max_z)
                console.run(f"forceload add {from_x * 16} {from_z * 16} {to_x * 16} {to_z * 16}", expect=("force loaded",))
                checks += [(x * 16, z * 16) for x in (from_x, to_x) for z in (from_z, to_z)]
        print(f"generating chunks {min_x}..{max_x} by {min_z}..{max_z}", flush=True)
    deadline = time.monotonic() + 1800
    while True:
        answer = console.run(*(f"execute if loaded {x} 0 {z}" for x, z in checks))
        if sum("Test passed" in line for line in answer) == len(checks):
            break
        if time.monotonic() > deadline:
            raise SystemExit("the bench's chunks weren't generated in 30 minutes")
        time.sleep(2)
    console.run("kill @e[type=!minecraft:player]")
    # The port draws no entities yet, so there must be none for vanilla to draw either.
    console.run("execute if entity @e[type=!minecraft:player]", expect=("Test failed",))
    for x, y, z, _, _ in spots(bench):
        # Air under a spot gets a barrier to stand on, which nothing draws.
        below = f"{block(x)} {block(y) - 1} {block(z)}"
        console.run(f"execute if block {block(x)} {block(y)} {block(z)} minecraft:air", expect=("Test passed",))
        console.run(f"execute if block {below} minecraft:air run setblock {below} minecraft:barrier")
    console.run("forceload remove all", "save-all flush")


def write_vanilla_options(game_dir, bench):
    options = bench["options"]
    overrides = {
        **VANILLA_OPTIONS,
        "renderDistance": str(options["render_distance"]),
        "guiScale": str(options["gui_scale"]),
        # Options.fov's slider: 70 degrees at 0, 40 degrees a unit.
        "fov": repr((options["fov"] - 70) / 40.0),
        "maxFps": str(options["max_fps"]),
    }
    lines = []
    for line in oracle_client.OPTIONS_FILE.read_text().splitlines():
        key = line.split(":", 1)[0]
        lines.append(f"{key}:{overrides.pop(key)}" if key in overrides else line)
    lines += [f"{key}:{value}" for key, value in overrides.items()]
    game_dir.mkdir(parents=True, exist_ok=True)
    (game_dir / "options.txt").write_text("\n".join(lines) + "\n")


def vanilla_command(game_dir, bench):
    version = json.loads((VERSION_DIR / "version.json").read_text())
    classpath = [str(VERSION_DIR / "client.jar"), *(VERSION_DIR / "classpath.txt").read_text().splitlines()]
    natives = game_dir / "natives"
    width, height = bench["options"]["window"]
    return [
        oracle_client.find_jdk() / "bin" / "java", *LAUNCHER_JVM_FLAGS,
        # The version's own JVM arguments, for Linux.
        "-XX:StackShadowPages=32", "--enable-native-access=ALL-UNNAMED", "--add-exports", "java.base/jdk.internal.misc=ALL-UNNAMED",
        f"-Djava.library.path={natives / 'java'}", f"-Djna.tmpdir={natives / 'jna'}",
        f"-Dorg.lwjgl.system.SharedLibraryExtractPath={natives / 'lwjgl'}", f"-Dio.netty.native.workdir={natives / 'netty'}",
        "-Dminecraft.launcher.brand=rustcraft-bench", "-Dminecraft.launcher.version=1",
        "-cp", os.pathsep.join(classpath), version["mainClass"],
        "--username", vanilla_server.OP_NAME, "--version", VERSION, "--gameDir", game_dir,
        # Loom's cache names the index after the version too.
        "--assetsDir", ASSETS_DIR, "--assetIndex", f"{VERSION}-{version['assetIndex']['id']}",
        "--uuid", vanilla_server.offline_uuid(vanilla_server.OP_NAME).hex, "--accessToken", "0", "--versionType", "release",
        "--width", str(width), "--height", str(height),
        "--quickPlayMultiplayer", f"127.0.0.1:{SERVER_PORT}",
    ]


def port_command(bench_path, events, port=SERVER_PORT, binary=port_client.PORT_BINARY):
    return [
        binary, "--quickPlayMultiplayer", f"127.0.0.1:{port}", "--username", vanilla_server.OP_NAME,
        "--bench", bench_path, "--benchOut", events,
    ]


def find_process(pattern, timeout=60):
    """The pid of the process whose command line matches `pattern`: the client itself, under xvfb-run."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        found = subprocess.run(["pgrep", "-f", pattern], capture_output=True, text=True).stdout.split()
        if found:
            return int(found[0])
        time.sleep(0.2)
    raise SystemExit(f"no process matching {pattern}")


def render_thread(pid, client):
    if client == "port":
        return pid
    for task in Path(f"/proc/{pid}/task").iterdir():
        if (task / "comm").read_text().strip() == "Render thread":
            return int(task.name)
    raise SystemExit("vanilla has no render thread")


def cpu_seconds(stat_path):
    """A process's or a thread's user and system time, from its /proc stat."""
    fields = Path(stat_path).read_text().rsplit(")", 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / CLK_TCK


def memory(pid):
    status = Path(f"/proc/{pid}/status").read_text()
    return {key: int(re.search(rf"{key}:\s+(\d+) kB", status)[1]) * 1024 for key in ("VmRSS", "VmHWM")}


def process_start(pid):
    """When the process started, in seconds since the epoch."""
    start_ticks = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[19])
    uptime = float(Path("/proc/uptime").read_text().split()[0])
    return time.time() - uptime + start_ticks / CLK_TCK


def wait_for_log(log_path, needle, since, process=None, timeout=600):
    """When `needle` showed in the log after offset `since`, polled every 50 ms."""
    deadline = time.monotonic() + timeout
    while True:
        with open(log_path, errors="replace") as log:
            log.seek(since)
            if needle in log.read():
                return time.time()
        if process is not None and process.poll() is not None:
            raise SystemExit(f"the client exited with {process.returncode} before {needle!r}")
        if time.monotonic() > deadline:
            raise SystemExit(f"no {needle!r} in {timeout} s")
        time.sleep(0.05)


def forget_player(server_run_dir):
    """Deletes the player's saved data, so the next run joins at the world's spawn like the first. The server logs the
    player leaving before it saves them (`removePlayerFromWorld` broadcasts it, then `PlayerList.remove` saves), through
    a file it renames, so this deletes what's there until nothing more has come for a second."""
    uuid = str(vanilla_server.offline_uuid(vanilla_server.OP_NAME))
    quiet_since = time.monotonic()
    while time.monotonic() - quiet_since < 1.0:
        for path in server_run_dir.rglob(f"{uuid}.dat*"):
            path.unlink(missing_ok=True)
            quiet_since = time.monotonic()
        time.sleep(0.1)


def stop_client(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=60)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()


def perf_split(pid, thread, client, run_out):
    """Samples the render thread for PERF_SECONDS and splits its samples by where they fell: the client's own code, the
    driver's (DRIVER_DSO, and JIT-compiled code the JVM's map doesn't name, which is llvmpipe's shaders) or the
    kernel's."""
    data = run_out / "perf.data"
    subprocess.run(
        ["sudo", "-n", "perf", "record", "-q", "-F", "499", "-t", str(thread), "-o", data, "--", "sleep", str(PERF_SECONDS)],
        check=True, capture_output=True,
    )
    subprocess.run(["sudo", "-n", "chown", getpass.getuser(), data], check=True)
    if client == "vanilla":
        # The JVM's map of its JIT-compiled code, /tmp/perf-<pid>.map, which names Java's methods in perf's `[JIT]`.
        subprocess.run([oracle_client.find_jdk() / "bin" / "jcmd", str(pid), "Compiler.perfmap"], check=True, capture_output=True)
    report = subprocess.run(
        ["perf", "report", "-f", "-i", data, "--stdio", "--sort", "dso,symbol", "-q", "--percent-limit", "0"],
        check=True, capture_output=True, text=True,
    ).stdout
    (run_out / "perf-symbols.txt").write_text(report)
    split = {"client": 0.0, "driver": 0.0, "kernel": 0.0}
    dsos = {}
    for line in report.splitlines():
        match = re.match(r"\s*([\d.]+)%\s+(.+?)\s+\[([.k])\]\s+(.*?)\s*$", line)
        if not match:
            continue
        share, dso, symbol = float(match[1]) / 100, match[2], match[4]
        if dso.startswith("[kernel") or match[3] == "k":
            kind = "kernel"
        elif dso.startswith("[JIT]"):
            kind = "driver" if symbol.startswith("0x") else "client"
        else:
            kind = "driver" if DRIVER_DSO.match(dso) else "client"
        split[kind] += share
        dsos[dso] = dsos.get(dso, 0.0) + share
    data.unlink()
    Path(f"/tmp/perf-{pid}.map").unlink(missing_ok=True)
    return split, sorted(dsos.items(), key=lambda item: -item[1])


def run_client(client, run, bench_path, bench, console, server_log, server_run_dir, out_dir, perf):
    client, _, binary = client.partition("@")
    binary = Path(binary).resolve() if binary else port_client.PORT_BINARY
    run_out = out_dir / (f"{client}-{binary.name}-{run}" if binary != port_client.PORT_BINARY else f"{client}-{run}")
    shutil.rmtree(run_out, ignore_errors=True)
    run_out.mkdir(parents=True)
    events = run_out / "events.jsonl"
    jdk_bin = oracle_client.find_jdk() / "bin"
    if client == "vanilla":
        game_dir = out_dir / "vanilla-game"
        write_vanilla_options(game_dir, bench)
        command = vanilla_command(game_dir, bench)
        pattern = f"^{command[0]} .*--gameDir {game_dir} "
        # Vanilla's log4j writes logs/ into the working directory, which the launcher makes the game's.
        cwd = game_dir
    else:
        command = port_command(bench_path, events, binary=binary)
        pattern = f"^{binary} .*--benchOut {events}"
        cwd = ROOT
    command = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24 -nolisten tcp", *map(str, command)]

    print(f"{client} run {run}: starting (log: {run_out / 'client.log'})", flush=True)
    log_offset = server_log.stat().st_size
    with open(run_out / "client.log", "w") as log:
        process = subprocess.Popen(
            command, cwd=cwd, env=oracle_client.headless_env(), stdout=log, stderr=subprocess.STDOUT, start_new_session=True
        )
    try:
        pid = find_process(pattern)
        joined = wait_for_log(server_log, JOINED, log_offset, process)
        times = {"process_start": process_start(pid), "joined": joined}
        if client == "vanilla":
            jfc = run_out / "bench.jfc"
            jfc.write_text(JFC.format(filter=";".join(TRACED)))
            subprocess.run([jdk_bin / "jcmd", str(pid), "JFR.start", "name=bench", f"settings={jfc}"], check=True, capture_output=True)
        time.sleep(bench["warmup_seconds"])

        times["path"] = []
        for (x, y, z, y_rot, x_rot), spot in zip(spots(bench), bench["path"]):
            console.run(f"tp {vanilla_server.OP_NAME} {x} {y} {z} {y_rot} {x_rot}")
            times["path"].append(time.time())
            time.sleep(spot["seconds"])
        x, y, z, y_rot, x_rot = spots(bench)[-1]
        console.run(f"tp {vanilla_server.OP_NAME} {x} {y} {z} {y_rot} {x_rot}")
        times["hold_teleport"] = time.time()
        time.sleep(bench["hold"]["settle_seconds"])

        thread = render_thread(pid, client)
        times["hold_start"] = time.time()
        cpu_start = (cpu_seconds(f"/proc/{pid}/task/{thread}/stat"), cpu_seconds(f"/proc/{pid}/stat"))
        time.sleep(bench["hold"]["seconds"])
        cpu_end = (cpu_seconds(f"/proc/{pid}/task/{thread}/stat"), cpu_seconds(f"/proc/{pid}/stat"))
        times["hold_end"] = time.time()
        times["render_thread_cpu"] = cpu_end[0] - cpu_start[0]
        times["process_cpu"] = cpu_end[1] - cpu_start[1]
        times["memory"] = memory(pid)
        times["load"] = os.getloadavg()
        if perf:
            times["perf_start"] = time.time()
            perf_cpu_start = cpu_seconds(f"/proc/{pid}/task/{thread}/stat")
            times["perf_split"], times["perf_dsos"] = perf_split(pid, thread, client, run_out)
            times["perf_render_thread_cpu"] = cpu_seconds(f"/proc/{pid}/task/{thread}/stat") - perf_cpu_start
            times["perf_end"] = time.time()

        if client == "vanilla":
            recording = run_out / "recording.jfr"
            subprocess.run([jdk_bin / "jcmd", str(pid), "JFR.stop", "name=bench", f"filename={recording}"], check=True, capture_output=True)
            with open(events, "w") as out:
                subprocess.run([jdk_bin / "java", ROOT / "tools" / "JfrEvents.java", recording], stdout=out, check=True)
            recording.unlink()
        else:
            # The port writes its events twice a second.
            time.sleep(1.5)
    finally:
        stop_client(process)
    wait_for_log(server_log, LEFT, log_offset, timeout=120)
    forget_player(server_run_dir)

    result = analyze(read_events(events), times)
    (run_out / "result.json").write_text(json.dumps({"times": times, "result": result}, indent=2))
    return result


def capture_join(bench_path, bench, server_log, server_run_dir, out_dir):
    """The port's join through packet-proxy, the warm-up long, into out_dir/capture.jsonl: the offline benchmark's input."""
    subprocess.run(["cargo", "build", "--quiet", "-p", "packet-proxy"], cwd=ROOT, check=True)
    capture = out_dir / "capture.jsonl"
    proxy_log = out_dir / "proxy.log"
    proxy_command = [
        ROOT / "target" / "debug" / "packet-proxy", "--once",
        "--packets", VERSION_DIR / "generated" / "reports" / "packets.json", "--out", capture,
        "--listen", f"127.0.0.1:{PROXY_PORT}", "--upstream", f"127.0.0.1:{SERVER_PORT}",
    ]
    with open(proxy_log, "w") as log:
        proxy = subprocess.Popen(proxy_command, stdout=log, stderr=subprocess.STDOUT)
    client = None
    try:
        wait_for_line(proxy_log, "capturing to", proxy)
        command = ["xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24 -nolisten tcp"]
        command += map(str, port_command(bench_path, out_dir / "capture-events.jsonl", PROXY_PORT))
        log_offset = server_log.stat().st_size
        print(f"capturing the port's join (log: {out_dir / 'capture-client.log'})", flush=True)
        with open(out_dir / "capture-client.log", "w") as log:
            client = subprocess.Popen(
                command, cwd=ROOT, env=oracle_client.headless_env(), stdout=log, stderr=subprocess.STDOUT, start_new_session=True
            )
        wait_for_log(server_log, JOINED, log_offset, client)
        time.sleep(bench["warmup_seconds"])
        stop_client(client)
        wait_for_log(server_log, LEFT, log_offset, timeout=120)
        proxy.wait(timeout=30)
    finally:
        if client is not None:
            stop_client(client)
        if proxy.poll() is None:
            proxy.terminate()
            proxy.wait(timeout=30)
    forget_player(server_run_dir)
    print(f"wrote {capture}", flush=True)


def read_events(path):
    events = {kind: [] for kind in EVENTS}
    with open(path) as lines:
        for line in lines:
            event = json.loads(line)
            events[event["event"]].append((event["start"] / 1e9, event["duration"] / 1e9))
    for kind in events.values():
        kind.sort()
    return events


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int(round(fraction * (len(ordered) - 1))))] if ordered else None


def frames_in(events, start, end):
    """Each frame that starts in [start, end): the time to the next frame's start, and to its last submit."""
    starts = [frame for frame, _ in events["frame"]]
    submits = [submit for submit, _ in events["submit"]]
    frames = []
    submit_index = 0
    for index in range(len(starts) - 1):
        if not start <= starts[index] < end:
            continue
        frame_end = starts[index + 1]
        while submit_index < len(submits) and submits[submit_index] < starts[index]:
            submit_index += 1
        submit = None
        while submit_index < len(submits) and submits[submit_index] < frame_end:
            submit = submits[submit_index]
            submit_index += 1
        frames.append((frame_end - starts[index], None if submit is None else submit - starts[index]))
    return frames


def within(events, kind, start, end):
    return [(when, duration) for when, duration in events[kind] if start <= when < end]


def analyze(events, times):
    """A run's numbers, from its events and when the runner did what."""
    ms = 1000.0

    def stat(values, how, unit=ms):
        values = list(values)
        if not values:
            return None
        return {"median": statistics.median, "mean": statistics.mean, "max": max}.get(how, lambda v: percentile(v, how))(values) * unit

    hold = frames_in(events, times["hold_start"], times["hold_end"])
    path_start, path_end = times["path"][0], times["hold_teleport"]
    path = frames_in(events, path_start, path_end)
    path_compiles = [duration for _, duration in within(events, "compile", path_start, path_end)]
    settle_compiles = within(events, "compile", path_end, times["hold_start"])
    loading = (path_start, times["hold_start"])
    chunks = [duration for _, duration in within(events, "chunk", *loading)]
    light = [duration for _, duration in within(events, "light", *loading)]
    ticks = [duration for _, duration in within(events, "tick", path_start, times["hold_end"])]
    hold_cpu = [cpu for _, cpu in hold if cpu is not None]
    result = {
        "startup_s": times["joined"] - times["process_start"],
        "hold_fps": len(hold) / (times["hold_end"] - times["hold_start"]),
        "hold_frame_ms_median": stat((frame for frame, _ in hold), "median"),
        "hold_frame_ms_p99": stat((frame for frame, _ in hold), 0.99),
        "hold_cpu_to_submit_ms_median": stat(hold_cpu, "median"),
        "hold_cpu_to_submit_ms_p95": stat(hold_cpu, 0.95),
        "hold_render_thread_cpu_ms_per_frame": times["render_thread_cpu"] / len(hold) * ms if hold else None,
        "hold_process_cpu_ms_per_frame": times["process_cpu"] / len(hold) * ms if hold else None,
        "hold_compiles": len(within(events, "compile", times["hold_start"], times["hold_end"])),
        "path_fps": len(path) / (path_end - path_start),
        "path_frame_ms_median": stat((frame for frame, _ in path), "median"),
        "path_frame_ms_p95": stat((frame for frame, _ in path), 0.95),
        "path_frame_ms_max": stat((frame for frame, _ in path), "max"),
        "path_compiles_per_s": len(path_compiles) / (path_end - path_start),
        "compile_us_median": stat(path_compiles, "median", 1e6),
        "compile_us_mean": stat(path_compiles, "mean", 1e6),
        "chunks": len(chunks),
        "chunk_us_median": stat(chunks, "median", 1e6),
        "chunk_us_mean": stat(chunks, "mean", 1e6),
        "light_us_per_chunk": sum(light) / len(chunks) * 1e6 if chunks else None,
        "settle_s": (max(start + duration for start, duration in settle_compiles) - path_end) if settle_compiles else None,
        "settle_compiles": len(settle_compiles),
        "tick_ms_median": stat(ticks, "median"),
        "tick_ms_p95": stat(ticks, 0.95),
        "rss_mib": times["memory"]["VmRSS"] / 2**20,
        "peak_rss_mib": times["memory"]["VmHWM"] / 2**20,
        "load_1min": times["load"][0],
    }
    if "perf_split" in times:
        perf_frames = frames_in(events, times["perf_start"], times["perf_end"])
        cpu_per_frame = times["perf_render_thread_cpu"] / len(perf_frames) * ms if perf_frames else None
        for kind, share in times["perf_split"].items():
            result[f"perf_{kind}_share"] = share
            result[f"perf_{kind}_ms_per_frame"] = cpu_per_frame * share if cpu_per_frame is not None else None
    return result


METRICS = [
    ("startup_s", "startup to join (s)"),
    ("hold_fps", "held view: frames/s"),
    ("hold_frame_ms_median", "held view: frame (ms), median"),
    ("hold_frame_ms_p99", "held view: frame (ms), p99"),
    ("hold_cpu_to_submit_ms_median", "held view: frame start to submit (ms), median"),
    ("hold_render_thread_cpu_ms_per_frame", "held view: render thread CPU (ms/frame)"),
    ("hold_process_cpu_ms_per_frame", "held view: process CPU (ms/frame)"),
    ("perf_client_ms_per_frame", "held view: render thread outside the driver (ms/frame)"),
    ("perf_driver_ms_per_frame", "held view: render thread in the driver (ms/frame)"),
    ("perf_kernel_ms_per_frame", "held view: render thread in the kernel (ms/frame)"),
    ("hold_compiles", "held view: section compiles"),
    ("path_fps", "loading: frames/s"),
    ("path_frame_ms_median", "loading: frame (ms), median"),
    ("path_frame_ms_p95", "loading: frame (ms), p95"),
    ("path_frame_ms_max", "loading: longest frame (ms)"),
    ("path_compiles_per_s", "loading: section compiles/s"),
    ("compile_us_median", "section compile (us), median"),
    ("compile_us_mean", "section compile (us), mean"),
    ("chunks", "chunks handled"),
    ("chunk_us_median", "chunk handled (us), median"),
    ("chunk_us_mean", "chunk handled (us), mean"),
    ("light_us_per_chunk", "light updates (us per chunk)"),
    ("settle_s", "hold's teleport to its last compile (s)"),
    ("settle_compiles", "sections compiled meanwhile"),
    ("tick_ms_median", "tick (ms), median"),
    ("tick_ms_p95", "tick (ms), p95"),
    ("rss_mib", "resident memory (MiB)"),
    ("peak_rss_mib", "peak resident memory (MiB)"),
    ("load_1min", "machine's load average (1 min)"),
]


def summarize(results):
    """The median of each metric over the runs, with the smallest and largest."""
    summary = {}
    for client, runs in results.items():
        summary[client] = {}
        for key, _ in METRICS:
            values = [run[key] for run in runs if run.get(key) is not None]
            if values:
                summary[client][key] = {"median": statistics.median(values), "min": min(values), "max": max(values), "runs": len(values)}
    return summary


def client_label(client):
    """A client's column: `port@BINARY` by the binary's name."""
    name, _, binary = client.partition("@")
    return f"{name}:{Path(binary).name}" if binary else name


def print_summary(summary, clients):
    print(f"{'':56}" + "".join(f"{client_label(client):>28}" for client in clients))
    for key, label in METRICS:
        if not any(key in summary.get(client, {}) for client in clients):
            continue
        row = f"{label:56}"
        for client in clients:
            value = summary.get(client, {}).get(key)
            row += f"{'-':>28}" if value is None else f"{value['median']:.2f} ({value['min']:.2f}..{value['max']:.2f})".rjust(28)
        print(row)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("bench", help="bench name in oracle/bench/, or a path to one")
    parser.add_argument("--runs", type=int, default=3, help="runs of each client (default: 3)")
    parser.add_argument("--clients", default="vanilla,port", help="which clients, in order (default: vanilla,port)")
    parser.add_argument("--perf", action="store_true", help=f"sample the render thread with perf for {PERF_SECONDS} s after the hold (sudo)")
    parser.add_argument("--capture", action="store_true", help="first capture the port's join for the offline benchmark")
    parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    args = parser.parse_args()

    bench_path, bench = load_bench(args.bench)
    clients = args.clients.split(",")
    if "port" in clients or args.capture:
        port_client.build_port()
    for client in clients:
        if client.partition("@")[0] not in ("vanilla", "port"):
            raise SystemExit(f"no client {client}: vanilla, port or port@BINARY")
    if "vanilla" in clients and not ASSETS_DIR.exists():
        raise SystemExit(f"no assets in {ASSETS_DIR}: run the oracle client once (tools/oracle_client.py run walk)")
    out_dir = OUT_DIR / bench_path.stem
    out_dir.mkdir(parents=True, exist_ok=True)

    name = f"bench-{bench_path.stem}"
    server_log = out_dir / "server.log"
    world = bench["world"]
    server_command = [
        sys.executable, ROOT / "tools" / "vanilla_server.py", "--name", name, "--port", str(SERVER_PORT),
        "--seed", str(world["seed"]), "--level-type", world["level_type"],
        "--view-distance", str(bench["options"]["render_distance"]), "--console", "--no-watchdog",
        *(["--accept-eula"] if args.accept_eula else []),
    ]
    with open(server_log, "w") as log:
        server = subprocess.Popen(server_command, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
    results = {client: [] for client in clients}
    try:
        wait_for_line(server_log, "ready on", server)
        console = port_client.Console(server, server_log)
        console.run(f"time set {world['time']}")
        prepare_world(console, bench)
        server_run_dir = VERSION_DIR / "server-run" / name
        if args.capture:
            capture_join(bench_path, bench, server_log, server_run_dir, out_dir)
        for run in range(args.runs):
            for client in clients if run % 2 == 0 else reversed(clients):
                result = run_client(client, run, bench_path, bench, console, server_log, server_run_dir, out_dir, args.perf)
                results[client].append(result)
                print(f"{client} run {run}: " + ", ".join(f"{key} {value:.2f}" for key, value in result.items() if value is not None), flush=True)
    finally:
        if server.poll() is None:
            server.stdin.close()
            try:
                server.wait(timeout=120)
            except subprocess.TimeoutExpired:
                server.terminate()
                server.wait()

    if not args.runs:
        return
    summary = summarize(results)
    (out_dir / "summary.json").write_text(json.dumps({"bench": bench_path.stem, "summary": summary, "runs": results}, indent=2))
    print(f"\n{bench_path.stem}: the median of {args.runs} runs (smallest..largest), load average at the start {os.getloadavg()[0]:.1f}")
    print_summary(summary, clients)


if __name__ == "__main__":
    main()
