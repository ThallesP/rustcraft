#!/usr/bin/env python3
"""Replay a scenario in the vanilla client and record what it does.

Runs the Fabric mod in oracle/client/ under Xvfb with Mesa's software renderer, so
the output doesn't depend on the machine's GPU. Writes trace.jsonl and screenshots
in the formats from TESTING.md. Needs tools/setup_vanilla.py to have run, and xvfb.

    python3 tools/oracle_client.py run walk                          # singleplayer
    python3 tools/oracle_client.py run walk --server 127.0.0.1:25566   # join a running server
    python3 tools/oracle_client.py capture walk --accept-eula        # fresh vanilla server + packet-proxy
    python3 tools/oracle_client.py capture walk --accept-eula --compression-threshold 256 --out vanilla/26.3/oracle-compressed-256
    python3 tools/oracle_client.py capture walk --accept-eula --ping   # the server list's ping, into oracle/walk-ping
    python3 tools/oracle_client.py check-determinism walk            # run twice, diff with oracle-diff
    python3 tools/oracle_client.py check-determinism walk --capture --accept-eula
    python3 tools/oracle_client.py capture walk --accept-eula --gl-profile macos --out vanilla/26.3/oracle-gl41

`--gl-profile macos` runs the client on llvmpipe restricted to what Apple's OpenGL 4.1 offers
(oracle/gl-profiles.json, TESTING.md's "GL profiles"); port_client.py takes the same option.
`--backend vulkan` runs it on vanilla's Vulkan backend (`--graphicsBackend vulkan`) on Mesa's
lavapipe, into references of their own (TESTING.md's "Backends"):

    python3 tools/oracle_client.py capture walk --accept-eula --backend vulkan --out vanilla/26.3/oracle-vulkan
"""

import argparse
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "MINECRAFT_VERSION").read_text().strip()
VERSION_DIR = ROOT / "vanilla" / VERSION
CLIENT_DIR = ROOT / "oracle" / "client"
# The game reads its options from the run directory and writes them back on exit, and the client game test API keeps
# every option it doesn't set itself, so a run would otherwise depend on what earlier runs left there (TESTING.md).
OPTIONS_FILE = CLIENT_DIR / "options.txt"
SCENARIOS_DIR = ROOT / "oracle" / "scenarios"
LAVAPIPE_ICD = Path("/usr/share/vulkan/icd.d/lvp_icd.json")
# Mesa's environment for each GL profile both clients can run under (TESTING.md, "GL profiles").
GL_PROFILES = json.loads((ROOT / "oracle" / "gl-profiles.json").read_text())
# The graphics backends both clients can render with: vanilla's `--graphicsBackend` (TESTING.md, "Backends").
BACKENDS = ("opengl", "vulkan")
# What vanilla logs once it made a backend's device; with `--graphicsBackend vulkan` it falls back to OpenGL when
# Vulkan fails, so a run on Vulkan has to say so.
BACKEND_LOG_LINES = {"opengl": "Using graphics backend OpenGL", "vulkan": "Using graphics backend Vulkan"}
SERVER_PORT = 25565
PROXY_PORT = 25566
# OracleGameTest prints this when it waits for the server's ready file.
WAITING_FOR_SERVER = "rustcraft oracle: waiting for the server"
# What a paced run fails with when one of the scenario's ticks missed its slot (ServerTicks), which the machine's load
# makes now and then: such a run is no reference, so it's made again, this many times in all at most (TESTING.md).
FELL_BEHIND = "fell behind the server's ticks"
PACED_ATTEMPTS = 3


def resolve_scenario(name_or_path):
    path = Path(name_or_path)
    if path.suffix == ".json" and path.exists():
        return path.resolve()
    path = SCENARIOS_DIR / f"{name_or_path}.json"
    if not path.exists():
        raise SystemExit(f"no scenario {name_or_path} (looked for {path})")
    return path


def find_jdk():
    version_json = VERSION_DIR / "version.json"
    if not version_json.exists():
        raise SystemExit(f"{VERSION_DIR} is missing, run python3 tools/setup_vanilla.py first")
    major = json.loads(version_json.read_text())["javaVersion"]["majorVersion"]
    return ROOT / "vanilla" / "toolchain" / f"jdk-{major}"


# Keep the game off the real desktop and GPU: X11 on Xvfb, llvmpipe for OpenGL, lavapipe for Vulkan, no audio device,
# and llvmpipe's OpenGL restricted to a GL profile's if there's one.
def headless_env(gl_profile=None):
    env = dict(os.environ)
    env.pop("WAYLAND_DISPLAY", None)
    env["JAVA_HOME"] = str(find_jdk())
    env["SDL_VIDEO_DRIVER"] = "x11"
    env["SDL_VIDEODRIVER"] = "x11"
    env["LIBGL_ALWAYS_SOFTWARE"] = "1"
    env["GALLIUM_DRIVER"] = "llvmpipe"
    env["__GLX_VENDOR_LIBRARY_NAME"] = "mesa"
    env["VK_DRIVER_FILES"] = str(LAVAPIPE_ICD)
    env["VK_ICD_FILENAMES"] = str(LAVAPIPE_ICD)
    env["ALSOFT_DRIVERS"] = "null"
    if gl_profile is not None:
        env.update(GL_PROFILES[gl_profile])
    return env


def expected_ticks(scenario):
    """The scenario's ticks, and every screenshot it takes, on the title screen and after the join."""
    root = json.loads(scenario.read_text())
    steps = root["steps"]
    screenshots = [step["screenshot"] for step in root.get("title", []) + steps if "screenshot" in step]
    return sum(step.get("ticks", 0) for step in steps), screenshots


def start(scenario, out_dir, server=None, server_ready=None, ping=False, gl_profile=None, backend="opengl"):
    """Starts the client replaying the scenario, joining `server` once `server_ready` exists if it's given.

    With `ping`, the client pings `server` the way the server list does instead, into `<out_dir>/<scenario>-ping`.
    """
    if shutil.which("xvfb-run") is None:
        raise SystemExit("xvfb-run is missing: sudo apt install xvfb")

    scenario_out = out_dir / (f"{scenario.stem}-ping" if ping else scenario.stem)
    shutil.rmtree(scenario_out, ignore_errors=True)
    scenario_out.mkdir(parents=True)
    (CLIENT_DIR / "run").mkdir(exist_ok=True)
    shutil.copyfile(OPTIONS_FILE, CLIENT_DIR / "run" / "options.txt")

    command = [
        "xvfb-run", "--auto-servernum", "--server-args=-screen 0 1920x1080x24 -nolisten tcp",
        "./gradlew", "--no-daemon", "--console=plain", "runOracle",
        f"-Poracle.scenario={scenario}",
        f"-Poracle.out={out_dir.resolve()}",
        f"-Poracle.backend={backend}",
    ]
    if server:
        command.append(f"-Poracle.ping={server}" if ping else f"-Poracle.server={server}")
    if server_ready:
        command.append(f"-Poracle.serverReady={server_ready}")

    log_path = scenario_out / "client.log"
    print(f"running {scenario_out.name} in the vanilla client (log: {log_path})")
    with open(log_path, "w") as log:
        # Its own process group, so stop() reaches the game under xvfb-run and Gradle.
        client = subprocess.Popen(
            command, cwd=CLIENT_DIR, env=headless_env(gl_profile), stdout=log, stderr=subprocess.STDOUT, start_new_session=True
        )
    return client, scenario_out


def stop(client):
    """Stops a client that's still running, with everything it started."""
    if client.poll() is None:
        os.killpg(client.pid, signal.SIGTERM)
        client.wait(timeout=60)


def finish(scenario, client, scenario_out, ping=False, backend="opengl"):
    """Waits for the client to exit and checks it rendered with the backend asked for and wrote the whole trace and
    every screenshot."""
    log_path = scenario_out / "client.log"
    if client.wait() != 0:
        print("".join(log_path.read_text().splitlines(keepends=True)[-40:]))
        if fell_behind(scenario_out):
            raise SystemExit(f"{FELL_BEHIND}: {scenario_out.name}, see {log_path}")
        raise SystemExit(f"the client exited with {client.returncode}, see {log_path}")
    if BACKEND_LOG_LINES[backend] not in log_path.read_text(errors="replace"):
        raise SystemExit(f"the client didn't render with {backend} (no {BACKEND_LOG_LINES[backend]!r}), see {log_path}")
    if ping:
        return scenario_out

    ticks, screenshots = expected_ticks(scenario)
    trace = scenario_out / "trace.jsonl"
    lines = len(trace.read_text().splitlines()) if trace.exists() else 0
    missing = [name for name in screenshots if not (scenario_out / "screenshots" / f"{name}.png").exists()]
    if lines != ticks or missing:
        raise SystemExit(f"incomplete run: {lines}/{ticks} trace lines, missing screenshots {missing}")
    print(f"wrote {trace} ({lines} ticks) and {len(screenshots)} screenshots")
    return scenario_out


def run(scenario, out_dir, server=None, gl_profile=None, backend="opengl"):
    client, scenario_out = start(scenario, out_dir, server, gl_profile=gl_profile, backend=backend)
    try:
        return finish(scenario, client, scenario_out, backend=backend)
    finally:
        stop(client)


def wait_for_line(log_path, needle, process):
    while needle not in log_path.read_text():
        if process.poll() is not None:
            print(log_path.read_text()[-4000:])
            raise SystemExit(f"{log_path.name}: the process exited before printing {needle!r}")
        time.sleep(0.5)


def fell_behind(scenario_out):
    """Whether a paced run failed because the client fell behind the server's ticks."""
    log = scenario_out / "client.log"
    return log.exists() and FELL_BEHIND in log.read_text(errors="replace")


def again_if_behind(attempt):
    """Runs `attempt` again while it fails by falling behind the server's ticks, up to PACED_ATTEMPTS times."""
    for remaining in range(PACED_ATTEMPTS - 1, -1, -1):
        try:
            return attempt()
        except SystemExit as failure:
            if remaining == 0 or not str(failure).startswith(FELL_BEHIND):
                raise
            print(f"{failure}: running it again ({remaining} more at most)")


def capture(scenario, out_dir, accept_eula, compression_threshold=None, ping=False, gl_profile=None, backend="opengl"):
    """Replays the scenario against a fresh vanilla server through packet-proxy, adding capture.jsonl to the run,
    again if a paced run fell behind the server's ticks."""
    return again_if_behind(
        lambda: capture_once(scenario, out_dir, accept_eula, compression_threshold, ping, gl_profile, backend)
    )


def capture_once(
    scenario, out_dir, accept_eula, compression_threshold=None, ping=False, gl_profile=None, backend="opengl"
):
    """Replays the scenario against a fresh vanilla server through packet-proxy, adding capture.jsonl to the run.

    The client starts first and waits on the title screen: the server starts once it's up, and the client joins as soon
    as the server is ready, like the port does (port_client.py). The server's timers then fall at the same points of
    both runs, wherever the client's startup takes its time. With `ping`, the client pings the scenario's server like
    the server list does instead of joining it, into `<out_dir>/<scenario>-ping`.
    """
    subprocess.run(["cargo", "build", "--quiet", "-p", "packet-proxy"], cwd=ROOT, check=True)
    logs = Path(tempfile.mkdtemp(prefix=f"oracle-{scenario.stem}-"))
    seed = json.loads(scenario.read_text())["world"]["seed"]
    server_ready = logs / "server-ready"

    client, scenario_out = start(scenario, out_dir, f"127.0.0.1:{PROXY_PORT}", server_ready, ping, gl_profile, backend)
    server = None
    proxy = None
    try:
        wait_for_line(scenario_out / "client.log", WAITING_FOR_SERVER, client)
        server_command = [sys.executable, ROOT / "tools" / "vanilla_server.py", "--name", f"capture-{scenario.stem}"]
        server_command += ["--port", str(SERVER_PORT), "--seed", str(seed), *(["--accept-eula"] if accept_eula else [])]
        if compression_threshold is not None:
            server_command += ["--compression-threshold", str(compression_threshold)]
        with open(logs / "server.log", "w") as server_log:
            server = subprocess.Popen(server_command, stdout=server_log, stderr=subprocess.STDOUT)
        wait_for_line(logs / "server.log", "ready on", server)
        proxy_command = [
            ROOT / "target" / "debug" / "packet-proxy", "--once",
            "--packets", VERSION_DIR / "generated" / "reports" / "packets.json", "--out", logs / "capture.jsonl",
            "--listen", f"127.0.0.1:{PROXY_PORT}", "--upstream", f"127.0.0.1:{SERVER_PORT}",
        ]
        with open(logs / "proxy.log", "w") as proxy_log:
            proxy = subprocess.Popen(proxy_command, stdout=proxy_log, stderr=subprocess.STDOUT)
        wait_for_line(logs / "proxy.log", "capturing to", proxy)
        server_ready.touch()

        finish(scenario, client, scenario_out, ping, backend)
        proxy.wait(timeout=30)
    finally:
        stop(client)
        if proxy is not None and proxy.poll() is None:
            proxy.terminate()
            proxy.wait(timeout=30)
        if server is not None:
            server.send_signal(signal.SIGTERM)
            server.wait(timeout=60)

    shutil.move(logs / "capture.jsonl", scenario_out / "capture.jsonl")
    print(f"wrote {scenario_out / 'capture.jsonl'} (server and proxy logs: {logs})")
    return scenario_out


def check_determinism(scenario, with_capture, accept_eula, gl_profile=None, backend="opengl"):
    base = VERSION_DIR / "oracle-determinism"
    runs = [
        capture(scenario, base / name, accept_eula, gl_profile=gl_profile, backend=backend) if with_capture
        else run(scenario, base / name, gl_profile=gl_profile, backend=backend)
        for name in ("a", "b")
    ]
    diff = subprocess.run(["cargo", "run", "--quiet", "-p", "oracle-diff", "--", "dir", *runs], cwd=ROOT)
    if diff.returncode != 0:
        raise SystemExit(f"NOT deterministic: {runs[0]} and {runs[1]} differ")
    print(f"deterministic: {runs[0]} and {runs[1]} match")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="replay a scenario once")
    run_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    run_parser.add_argument("--out", type=Path, default=VERSION_DIR / "oracle", help="output directory (default: vanilla/<version>/oracle)")
    run_parser.add_argument("--server", help="join HOST:PORT instead of creating a singleplayer world")
    run_parser.add_argument("--gl-profile", choices=sorted(GL_PROFILES), help="restrict llvmpipe's OpenGL to a profile's (TESTING.md)")
    run_parser.add_argument("--backend", choices=BACKENDS, default="opengl", help="vanilla's graphics backend (default: opengl; TESTING.md)")
    capture_parser = commands.add_parser("capture", help="replay a scenario on a fresh vanilla server and capture its packets")
    capture_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    capture_parser.add_argument("--out", type=Path, default=VERSION_DIR / "oracle", help="output directory (default: vanilla/<version>/oracle)")
    capture_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    capture_parser.add_argument("--compression-threshold", type=int, help="turn the server's compression on, from this packet size")
    capture_parser.add_argument("--ping", action="store_true", help="ping the server like the server list does instead of joining it")
    capture_parser.add_argument("--gl-profile", choices=sorted(GL_PROFILES), help="restrict llvmpipe's OpenGL to a profile's (TESTING.md)")
    capture_parser.add_argument("--backend", choices=BACKENDS, default="opengl", help="vanilla's graphics backend (default: opengl; TESTING.md)")
    check_parser = commands.add_parser("check-determinism", help="replay a scenario twice and diff the runs")
    check_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    check_parser.add_argument("--capture", action="store_true", help="run on a vanilla server and compare packets too")
    check_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    check_parser.add_argument("--gl-profile", choices=sorted(GL_PROFILES), help="restrict llvmpipe's OpenGL to a profile's (TESTING.md)")
    check_parser.add_argument("--backend", choices=BACKENDS, default="opengl", help="vanilla's graphics backend (default: opengl; TESTING.md)")
    args = parser.parse_args()

    scenario = resolve_scenario(args.scenario)
    if args.command == "run":
        run(scenario, args.out, args.server, args.gl_profile, args.backend)
    elif args.command == "capture":
        capture(scenario, args.out, args.accept_eula, args.compression_threshold, args.ping, args.gl_profile, args.backend)
    else:
        check_determinism(scenario, args.capture, args.accept_eula, args.gl_profile, args.backend)


if __name__ == "__main__":
    main()
