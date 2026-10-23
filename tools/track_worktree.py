#!/usr/bin/env python3
"""Make a git worktree for one of GOAL.md's tracks, sharing this checkout's vanilla/.

    python3 tools/track_worktree.py world        # ../rustcraft-tracks/world on branch track/world
    python3 tools/track_worktree.py world --relocate   # move an idle worktree's build output to the scratch volume

The worktree gets its own vanilla/<version>/golden, golden-classes and logs, since
tools/golden.py rewrites them and the tracks' golden harnesses differ. Everything else
in vanilla/ (jars, sources, resources, libraries, the JDK, the data reports, oracle
captures, the server's run directory) is a symlink to this checkout's copy. Runs that
touch the shared server directory or oracle captures take /tmp/rustcraft-oracle.lock
(GOAL.md).

The worktree's target/ and its golden and golden-classes directories live on the scratch
volume (RUSTCRAFT_SCRATCH, by default /tmp/rustcraft-work/<track>/), linked from the
worktree, since they're regenerable and the root disk is small. --relocate moves an existing
worktree's real directories there; run it only while no build or test uses them.
"""

import argparse
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "MINECRAFT_VERSION").read_text().strip()
OWN = {"golden", "golden-classes", "logs"}
SCRATCH = {"golden", "golden-classes"}


def scratch_dir(track):
    return Path(os.environ.get("RUSTCRAFT_SCRATCH", "/tmp/rustcraft-work")) / track


def link_to_scratch(path, scratch):
    """Makes `path` a link to `scratch`, moving what `path` held there first."""
    if path.is_symlink():
        return
    scratch.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        shutil.rmtree(scratch, ignore_errors=True)
        shutil.move(str(path), str(scratch))
    else:
        scratch.mkdir(parents=True, exist_ok=True)
    path.symlink_to(scratch)


def link_vanilla(worktree, track):
    shared = ROOT / "vanilla"
    vanilla = worktree / "vanilla"
    link_to_scratch(worktree / "target", scratch_dir(track) / "target")
    if vanilla.is_symlink():
        # A worktree that only reads (the audit's) links vanilla/ whole: writing through it would change this checkout's.
        return
    vanilla.mkdir(exist_ok=True)
    for entry in shared.iterdir():
        if entry.name != VERSION and not (vanilla / entry.name).exists():
            (vanilla / entry.name).symlink_to(entry)

    version_dir = vanilla / VERSION
    version_dir.mkdir(exist_ok=True)
    for entry in (shared / VERSION).iterdir():
        target = version_dir / entry.name
        if entry.name not in OWN and not target.exists() and not target.is_symlink():
            target.symlink_to(entry)
    for name in sorted(OWN - SCRATCH):
        (version_dir / name).mkdir(exist_ok=True)
    for name in sorted(SCRATCH):
        link_to_scratch(version_dir / name, scratch_dir(track) / name)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("track", help="track name, e.g. net, world, render")
    parser.add_argument("--base", default="origin/main", help="commit to start the branch from (default: origin/main)")
    parser.add_argument("--relocate", action="store_true", help="only move an existing worktree's build output to the scratch volume")
    args = parser.parse_args()

    worktree = ROOT.parent / "rustcraft-tracks" / args.track
    if not worktree.exists():
        if args.relocate:
            raise SystemExit(f"{worktree} doesn't exist")
        subprocess.run(["git", "fetch", "origin"], cwd=ROOT, check=True)
        subprocess.run(["git", "worktree", "add", "-B", f"track/{args.track}", worktree, args.base], cwd=ROOT, check=True)
    link_vanilla(worktree, args.track)
    print(worktree)


if __name__ == "__main__":
    main()
