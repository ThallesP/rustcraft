#!/usr/bin/env python3
"""Write vanilla's answers for the Rust golden tests.

Compiles oracle/golden/ against the client jar and runs it, writing one JSON Lines
file per class to vanilla/<version>/golden/. Run tools/setup_vanilla.py first.

    python3 tools/golden.py
"""

import json
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "MINECRAFT_VERSION").read_text().strip()
VERSION_DIR = ROOT / "vanilla" / VERSION
SOURCE_DIR = ROOT / "oracle" / "golden" / "src"


def main():
    classpath_file = VERSION_DIR / "classpath.txt"
    if not classpath_file.exists():
        raise SystemExit(f"{classpath_file} is missing: run python3 tools/setup_vanilla.py first")

    version = json.loads((VERSION_DIR / "version.json").read_text())
    jdk_bin = ROOT / "vanilla" / "toolchain" / f"jdk-{version['javaVersion']['majorVersion']}" / "bin"
    classpath = [VERSION_DIR / "client.jar", *map(Path, classpath_file.read_text().splitlines())]

    classes_dir = VERSION_DIR / "golden-classes"
    shutil.rmtree(classes_dir, ignore_errors=True)
    sources = sorted(SOURCE_DIR.rglob("*.java"))
    subprocess.run([jdk_bin / "javac", "-d", classes_dir, "-cp", os.pathsep.join(map(str, classpath)), *sources], check=True)

    # Run inside vanilla/, since loading game classes makes log4j create logs/ in the working directory. Cases never
    # release the buffers Netty hands back (ByteBuf.readBytes), so its leak detector would only log noise.
    output_dir = VERSION_DIR / "golden"
    shutil.rmtree(output_dir, ignore_errors=True)
    subprocess.run(
        [
            jdk_bin / "java", "-Dio.netty.leakDetection.level=disabled", "-cp", os.pathsep.join(map(str, [classes_dir, *classpath])),
            "rustcraft.golden.Golden", output_dir,
        ],
        cwd=VERSION_DIR,
        check=True,
    )


if __name__ == "__main__":
    main()
