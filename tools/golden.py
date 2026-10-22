#!/usr/bin/env python3
"""Write vanilla's answers for the Rust golden tests.

Compiles oracle/golden/ against the client jar and runs it, writing one JSON Lines
file per class to vanilla/<version>/golden/. Run tools/setup_vanilla.py first.

    python3 tools/golden.py                         # every suite
    python3 tools/golden.py net.minecraft.util.Mth  # only these suites, keeping the other files
"""

import json
import os
import shutil
import subprocess
import sys
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
    # release the buffers Netty hands back (ByteBuf.readBytes), so its leak detector would only log noise. A compound's
    # encoding has its HashMap's table length, which only reflection into java.util can read.
    #
    # Reading NBT wraps an IOException in a crash report, whose system details (SystemReport) probe the hardware with
    # OSHI each time: about a thousand files under /proc and /sys, and lshw, dmidecode, lspci and cpuid as processes,
    # over 20 ms a report for the thousands of reports the NBT cases make. So the generator runs without OSHI's jar:
    # making a SystemInfo throws NoClassDefFoundError, which SystemReport catches like any detail it can't get, and
    # the details only go into the exception's message, which no golden records.
    runtime_classpath = [classes_dir, *(path for path in classpath if not path.name.startswith("oshi-core-"))]
    output_dir = VERSION_DIR / "golden"
    only = sys.argv[1:]
    if not only:
        shutil.rmtree(output_dir, ignore_errors=True)
    subprocess.run(
        [
            jdk_bin / "java", "-Dio.netty.leakDetection.level=disabled", "--add-opens", "java.base/java.util=ALL-UNNAMED", "-cp", os.pathsep.join(map(str, runtime_classpath)),
            "rustcraft.golden.Golden", output_dir, *only,
        ],
        cwd=VERSION_DIR,
        check=True,
    )


if __name__ == "__main__":
    main()
