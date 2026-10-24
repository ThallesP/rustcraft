#!/usr/bin/env python3
"""Run a vanilla dedicated server with deterministic settings, for packet captures.

Every run starts from a fresh world in vanilla/<version>/server-run/<name>/: offline mode,
no compression, a flat world with a fixed seed, peaceful, no daylight or weather cycle, no
mob spawning and players spawning on the exact world spawn. The offline player "rustcraft"
is op, so a client can run setup commands.

    python3 tools/vanilla_server.py --accept-eula
    python3 tools/vanilla_server.py --accept-eula --name capture --port 25570
    python3 tools/vanilla_server.py --accept-eula --level-type minecraft:normal --generate-structures --console

`--level-type` and `--generate-structures` give a generated world instead of the flat one, with
its villages and the rest (`port_client.py soak` uses them). With `--console`, each line the
script reads on its stdin goes to the server's console once the world is set up, so a parent
process can drive it (`tp`, `locate`, ...) and read the answers in the server output, which the
script echoes to its stdout. The end of its stdin stops the server, so a parent that dies
doesn't leave it running.

Prints "ready on 127.0.0.1:<port>" once the world is set up, and stops the server cleanly
on Ctrl-C or SIGTERM. Needs tools/setup_vanilla.py to have run for the version.
"""

import argparse
import hashlib
import json
import shutil
import signal
import subprocess
import sys
import threading
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VANILLA_DIR = ROOT / "vanilla"
TOOLCHAIN_DIR = VANILLA_DIR / "toolchain"
EULA_URL = "https://aka.ms/MinecraftEULA"
OP_NAME = "rustcraft"

SERVER_PROPERTIES = {
    "online-mode": "false",
    "white-list": "false",
    "enforce-secure-profile": "false",
    "server-ip": "127.0.0.1",
    "level-type": "minecraft:flat",
    "generate-structures": "false",
    "difficulty": "peaceful",
    "spawn-protection": "0",
    "view-distance": "4",
    "simulation-distance": "4",
    "pause-when-empty-seconds": "0",
    "sync-chunk-writes": "false",
}

SETUP_COMMANDS = [
    "gamerule advance_time false",
    "gamerule advance_weather false",
    "gamerule random_tick_speed 0",
    "gamerule spawn_mobs false",
    "gamerule spawn_monsters false",
    "gamerule spawn_patrols false",
    "gamerule spawn_phantoms false",
    "gamerule spawn_wandering_traders false",
    "gamerule respawn_radius 0",
    "time set 6000",
    "weather clear",
]

# Console commands run in order, so once `list` answers, every setup command before it has run.
BARRIER_COMMAND = "list"
BARRIER_OUTPUT = "players online"
COMMAND_ERRORS = ("Unknown or incomplete command", "Incorrect argument for command")


# Same as Java's UUID.nameUUIDFromBytes("OfflinePlayer:" + name), which offline-mode servers use.
def offline_uuid(name):
    return uuid.UUID(bytes=hashlib.md5(f"OfflinePlayer:{name}".encode()).digest(), version=3)


def prepare_run_dir(run_dir, port, seed, compression_threshold, level_type, generate_structures, view_distance, watchdog=True):
    shutil.rmtree(run_dir, ignore_errors=True)
    run_dir.mkdir(parents=True)

    # Overriding keys keeps their place, so the defaults write the same file as ever.
    properties = {
        **SERVER_PROPERTIES,
        "level-type": level_type,
        "generate-structures": "true" if generate_structures else "false",
        "view-distance": str(view_distance),
        "server-port": str(port),
        "level-seed": str(seed),
        "network-compression-threshold": str(compression_threshold),
    }
    if not watchdog:
        # A tick that takes a minute (a busy machine generating chunks) doesn't stop the server.
        properties["max-tick-time"] = "-1"
    (run_dir / "server.properties").write_text("".join(f"{key}={value}\n" for key, value in properties.items()))
    (run_dir / "eula.txt").write_text("eula=true\n")
    ops = [{"uuid": str(offline_uuid(OP_NAME)), "name": OP_NAME, "level": 4, "bypassesPlayerLimit": False}]
    (run_dir / "ops.json").write_text(json.dumps(ops, indent=2))


def send(server, command):
    try:
        server.stdin.write(command + "\n")
        server.stdin.flush()
    except BrokenPipeError:
        pass


def forward_console(server, ready):
    """`--console`: sends each line of our stdin to the server once it's set up, then `stop` at the end of it."""
    ready.wait()
    for line in sys.stdin:
        send(server, line.rstrip("\n"))
    send(server, "stop")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--accept-eula", action="store_true", help=f"accept Mojang's EULA ({EULA_URL})")
    parser.add_argument("--version", help="Minecraft version id (default: MINECRAFT_VERSION)")
    parser.add_argument("--name", default="default", help="run directory name under server-run/ (default: default)")
    parser.add_argument("--port", type=int, default=25565)
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--compression-threshold", type=int, default=-1, help="packet size to compress from (default: -1, off)")
    parser.add_argument("--level-type", default="minecraft:flat", help="world preset, e.g. minecraft:normal (default: minecraft:flat)")
    parser.add_argument("--generate-structures", action="store_true", help="generate villages and the other structures")
    parser.add_argument("--view-distance", type=int, default=4, help="chunks the server sends around a player (default: 4)")
    parser.add_argument("--console", action="store_true", help="forward stdin to the server's console once it's ready, stopping it at its end")
    parser.add_argument("--no-watchdog", action="store_true", help="don't stop the server when a tick takes over a minute (max-tick-time -1)")
    args = parser.parse_args()

    if not args.accept_eula:
        raise SystemExit(f"Running the server means accepting Mojang's EULA: {EULA_URL}\nRe-run with --accept-eula.")

    version_id = args.version or (ROOT / "MINECRAFT_VERSION").read_text().strip()
    version_dir = VANILLA_DIR / version_id
    server_jar = version_dir / "server.jar"
    if not server_jar.exists():
        raise SystemExit(f"{server_jar} is missing: run python3 tools/setup_vanilla.py {version_id}")

    major = json.loads((version_dir / "version.json").read_text())["javaVersion"]["majorVersion"]
    java = TOOLCHAIN_DIR / f"jdk-{major}" / "bin" / "java"
    run_dir = version_dir / "server-run" / args.name
    prepare_run_dir(
        run_dir, args.port, args.seed, args.compression_threshold,
        args.level_type, args.generate_structures, args.view_distance, not args.no_watchdog,
    )

    # The bundler unpacks the server and its libraries once into the shared repo dir, not every run dir.
    # Its own session keeps a terminal Ctrl-C away from Java, so the server stops through `stop` instead.
    server = subprocess.Popen(
        [
            java, "--enable-native-access=ALL-UNNAMED", "--sun-misc-unsafe-memory-access=allow",
            f"-DbundlerRepoDir={version_dir / 'server'}", "-jar", server_jar, "nogui",
        ],
        cwd=run_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        start_new_session=True,
    )

    def stop(signum, frame):
        send(server, "stop")

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)

    # Lines that come before the setup is done wait in the pipe, so the console's commands run after it.
    console_ready = threading.Event()
    if args.console:
        threading.Thread(target=forward_console, args=(server, console_ready), daemon=True).start()

    ready = False
    setup_failed = False
    for line in server.stdout:
        print(line, end="", flush=True)
        if ready:
            continue
        if "Done (" in line:
            for command in [*SETUP_COMMANDS, BARRIER_COMMAND]:
                send(server, command)
        elif any(error in line for error in COMMAND_ERRORS):
            setup_failed = True
            send(server, "stop")
        elif BARRIER_OUTPUT in line and not setup_failed:
            ready = True
            print(f"ready on 127.0.0.1:{args.port}", flush=True)
            console_ready.set()

    exit_code = server.wait()
    if setup_failed:
        raise SystemExit("a setup command failed, see the server output above")
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
