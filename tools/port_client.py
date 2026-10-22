#!/usr/bin/env python3
"""Replay a scenario in the port on a fresh vanilla server, and record its packets.

The port's counterpart of `oracle_client.py capture`: a fresh `vanilla_server.py`, `packet-proxy`
in front of it, and the port joining as `rustcraft`, which sends the oracle client's setup
commands and its scenario-start marker (src/rustcraft/oracle/client/oracle_game_test.rs). Writes
target/oracle/<scenario>/capture.jsonl, to compare with vanilla's:

    python3 tools/port_client.py capture walk --accept-eula
    cargo run -p oracle-diff -- packets --join vanilla/26.3/oracle/walk/capture.jsonl target/oracle/walk/capture.jsonl

Wrap it in `flock /tmp/rustcraft-oracle.lock` on a shared machine: the server and the proxy use
fixed ports. `--compression-threshold` turns the server's compression on, which captures don't
show (the proxy records packets inflated).
"""

import argparse
import json
import shutil
import signal
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from oracle_client import PROXY_PORT, SERVER_PORT, VERSION_DIR, resolve_scenario, wait_for_line  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
PORT_BINARY = ROOT / "target" / "release" / "rustcraft"


def run(scenario, out_dir, server):
    """Replays the scenario in the port, joining `server`."""
    scenario_out = out_dir / scenario.stem
    shutil.rmtree(scenario_out, ignore_errors=True)
    scenario_out.mkdir(parents=True)

    command = [
        PORT_BINARY, "--quickPlayMultiplayer", server, "--username", "rustcraft", "--headless",
        "--oracleScenario", scenario,
    ]
    log_path = scenario_out / "client.log"
    print(f"running {scenario.stem} in the port (log: {log_path})")
    with open(log_path, "w") as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=600)
    if result.returncode != 0:
        print("".join(log_path.read_text().splitlines(keepends=True)[-40:]))
        raise SystemExit(f"the port exited with {result.returncode}, see {log_path}")
    return scenario_out


def capture(scenario, out_dir, accept_eula, compression_threshold):
    """Replays the scenario against a fresh vanilla server through packet-proxy, adding capture.jsonl to the run."""
    subprocess.run(["cargo", "build", "--quiet", "--release", "-p", "rustcraft"], cwd=ROOT, check=True)
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

        scenario_out = run(scenario, out_dir, f"127.0.0.1:{PROXY_PORT}")
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


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    capture_parser = commands.add_parser("capture", help="replay a scenario on a fresh vanilla server and capture its packets")
    capture_parser.add_argument("scenario", help="scenario name in oracle/scenarios/, or a path to one")
    capture_parser.add_argument("--out", type=Path, default=ROOT / "target" / "oracle", help="output directory (default: target/oracle)")
    capture_parser.add_argument("--accept-eula", action="store_true", help="accept Mojang's EULA for the server")
    capture_parser.add_argument("--compression-threshold", type=int, help="turn the server's compression on, from this packet size")
    args = parser.parse_args()

    scenario = resolve_scenario(args.scenario)
    if args.command == "capture":
        capture(scenario, args.out, args.accept_eula, args.compression_threshold)


if __name__ == "__main__":
    main()
