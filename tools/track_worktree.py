#!/usr/bin/env python3
"""Make a git worktree for one of GOAL.md's tracks, sharing this checkout's vanilla/.

    python3 tools/track_worktree.py world        # ../rustcraft-tracks/world on branch track/world

The worktree gets its own vanilla/<version>/golden, golden-classes and logs, since
tools/golden.py rewrites them and the tracks' golden harnesses differ. Everything else
in vanilla/ (jars, sources, resources, libraries, the JDK, the data reports, oracle
captures, the server's run directory) is a symlink to this checkout's copy. Runs that
touch the shared server directory or oracle captures take /tmp/rustcraft-oracle.lock
(GOAL.md).
"""

import argparse
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "MINECRAFT_VERSION").read_text().strip()
OWN = {"golden", "golden-classes", "logs"}


def link_vanilla(worktree):
    shared = ROOT / "vanilla"
    vanilla = worktree / "vanilla"
    vanilla.mkdir(exist_ok=True)
    for entry in shared.iterdir():
        if entry.name != VERSION and not (vanilla / entry.name).exists():
            (vanilla / entry.name).symlink_to(entry)

    version_dir = vanilla / VERSION
    version_dir.mkdir(exist_ok=True)
    for entry in (shared / VERSION).iterdir():
        target = version_dir / entry.name
        if entry.name in OWN:
            target.mkdir(exist_ok=True)
        elif not target.exists() and not target.is_symlink():
            target.symlink_to(entry)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("track", help="track name, e.g. net, world, render")
    parser.add_argument("--base", default="origin/main", help="commit to start the branch from (default: origin/main)")
    args = parser.parse_args()

    worktree = ROOT.parent / "rustcraft-tracks" / args.track
    if not worktree.exists():
        subprocess.run(["git", "fetch", "origin"], cwd=ROOT, check=True)
        subprocess.run(["git", "worktree", "add", "-B", f"track/{args.track}", worktree, args.base], cwd=ROOT, check=True)
    link_vanilla(worktree)
    print(worktree)


if __name__ == "__main__":
    main()
