#!/usr/bin/env python3
"""Fetch a vanilla Minecraft version and turn it into a porting reference.

Downloads the client and server jars plus the client's libraries, decompiles the
client with Vineflower, adds the sources of the libraries the port translates too
(JOML), extracts its bundled resources and runs the server's data generator.
Everything lands in vanilla/, which is gitignored: Mojang's code and assets can't
be redistributed.

    python3 tools/setup_vanilla.py           # the version pinned in MINECRAFT_VERSION
    python3 tools/setup_vanilla.py 26.4      # another version, e.g. to diff an update
    python3 tools/setup_vanilla.py --skip-decompile   # just what the golden tests need (CI)
"""

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import tarfile
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VANILLA_DIR = ROOT / "vanilla"
TOOLCHAIN_DIR = VANILLA_DIR / "toolchain"
LIBRARIES_DIR = VANILLA_DIR / "libraries"
PINNED_VERSION = (ROOT / "MINECRAFT_VERSION").read_text().strip()

VERSION_MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
ADOPTIUM_URL = "https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture={arch}&image_type=jdk&os=linux&vendor=eclipse"
VINEFLOWER_VERSION = "1.12.0"
VINEFLOWER_SHA256 = "1dfcfe974395734fa467ce620661c7623d05ba83670de0529b1fbd63ff548b9d"
VINEFLOWER_URL = f"https://repo1.maven.org/maven2/org/vineflower/vineflower/{VINEFLOWER_VERSION}/vineflower-{VINEFLOWER_VERSION}.jar"
MAVEN_CENTRAL = "https://repo1.maven.org/maven2/"
# Libraries the port translates like the game's own classes, from their published sources.
SOURCE_LIBRARIES = {"org.joml:joml"}


# Adoptium's API answers 403 to urllib's default User-Agent.
def open_url(url):
    return urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "rustcraft-setup"}))


def fetch_json(url):
    with open_url(url) as response:
        return json.load(response)


def file_digest(path, algorithm):
    digest = hashlib.new(algorithm)
    with open(path, "rb") as file:
        for chunk in iter(lambda: file.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download(url, destination, algorithm, expected):
    if destination.exists() and file_digest(destination, algorithm) == expected:
        return

    destination.parent.mkdir(parents=True, exist_ok=True)
    partial = destination.with_name(destination.name + ".part")
    print(f"downloading {url}")
    with open_url(url) as response, open(partial, "wb") as file:
        shutil.copyfileobj(response, file)

    actual = file_digest(partial, algorithm)
    if actual != expected:
        partial.unlink()
        raise SystemExit(f"{algorithm} mismatch for {url}: expected {expected}, got {actual}")
    partial.replace(destination)


def fetch_text(url):
    with open_url(url) as response:
        return response.read().decode().split()[0]


# Puts a library's sources next to the decompiled game, so src/org/joml/matrix4f.rs reads
# side by side with vanilla/<version>/src/org/joml/Matrix4f.java.
def add_library_sources(library, source_dir):
    artifact = library["downloads"]["artifact"]
    central_sha1 = fetch_text(MAVEN_CENTRAL + artifact["path"] + ".sha1")
    if central_sha1 != artifact["sha1"]:
        raise SystemExit(f"{library['name']} on Maven Central isn't the jar Mojang ships, so its sources may not match")

    sources_path = artifact["path"].removesuffix(".jar") + "-sources.jar"
    sources_jar = LIBRARIES_DIR / sources_path
    download(MAVEN_CENTRAL + sources_path, sources_jar, "sha1", fetch_text(MAVEN_CENTRAL + sources_path + ".sha1"))
    with zipfile.ZipFile(sources_jar) as jar:
        jar.extractall(source_dir, [name for name in jar.namelist() if name.endswith(".java")])


def resolve_version(version_id):
    manifest = fetch_json(VERSION_MANIFEST_URL)
    entry = next((version for version in manifest["versions"] if version["id"] == version_id), None)
    if entry is None:
        raise SystemExit(f"unknown version {version_id}")
    return fetch_json(entry["url"])


# Mojang's launcher rules: no rules means allowed, otherwise the last matching rule wins.
def is_library_allowed(library):
    allowed = "rules" not in library
    for rule in library.get("rules", []):
        os_name = rule.get("os", {}).get("name")
        if os_name is None or os_name == "linux":
            allowed = rule["action"] == "allow"
    return allowed


def ensure_jdk(major):
    jdk_dir = TOOLCHAIN_DIR / f"jdk-{major}"
    java = jdk_dir / "bin" / "java"
    if java.exists():
        return java

    arch = {"x86_64": "x64", "aarch64": "aarch64"}[platform.machine()]
    package = fetch_json(ADOPTIUM_URL.format(major=major, arch=arch))[0]["binary"]["package"]
    archive = TOOLCHAIN_DIR / package["name"]
    download(package["link"], archive, "sha256", package["checksum"])

    with tarfile.open(archive) as tar:
        top_level = tar.getnames()[0].split("/")[0]
        tar.extractall(TOOLCHAIN_DIR, filter="data")
    (TOOLCHAIN_DIR / top_level).rename(jdk_dir)
    archive.unlink()
    return java


def ensure_vineflower():
    jar = TOOLCHAIN_DIR / f"vineflower-{VINEFLOWER_VERSION}.jar"
    download(VINEFLOWER_URL, jar, "sha256", VINEFLOWER_SHA256)
    return jar


def decompile(java, vineflower, client_jar, libraries, source_dir):
    marker = source_dir / ".complete"
    if marker.exists():
        return

    shutil.rmtree(source_dir, ignore_errors=True)
    print(f"decompiling {client_jar} into {source_dir}")
    subprocess.run(
        [
            java, "-Xmx8G", "-jar", vineflower,
            "--folder", "--skip-extra-files", "--indent-string=    ", "--log-level=warn",
            f"--thread-count={os.cpu_count()}",
            *(f"-e={library}" for library in libraries),
            client_jar, source_dir,
        ],
        check=True,
    )
    marker.touch()


def extract_resources(client_jar, resources_dir):
    if resources_dir.exists():
        return

    partial = resources_dir.with_name(resources_dir.name + ".part")
    shutil.rmtree(partial, ignore_errors=True)
    with zipfile.ZipFile(client_jar) as jar:
        members = [name for name in jar.namelist() if not name.endswith(".class") and not name.startswith("META-INF/")]
        jar.extractall(partial, members)
    partial.rename(resources_dir)


# The server jar is a bundler: it unpacks itself into the working directory, then runs bundlerMainClass.
def run_data_generator(java, server_jar, version_dir):
    generated_dir = version_dir / "generated"
    marker = generated_dir / ".complete"
    if marker.exists():
        return

    work_dir = version_dir / "server"
    work_dir.mkdir(exist_ok=True)
    print(f"running the data generator into {generated_dir}")
    subprocess.run(
        [java, "-DbundlerMainClass=net.minecraft.data.Main", "-jar", server_jar, "--all", "--output", generated_dir],
        cwd=work_dir,
        check=True,
    )
    marker.touch()


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("version", nargs="?", default=PINNED_VERSION, help=f"Minecraft version id (default: {PINNED_VERSION})")
    parser.add_argument("--skip-decompile", action="store_true", help="skip the slow decompile, e.g. in CI")
    args = parser.parse_args()

    version = resolve_version(args.version)
    downloads = version["downloads"]
    if "client_mappings" in downloads:
        raise SystemExit(f"{version['id']} ships obfuscated; remapping with Mojang's mappings isn't supported here")

    version_dir = VANILLA_DIR / version["id"]
    version_dir.mkdir(parents=True, exist_ok=True)
    (version_dir / "version.json").write_text(json.dumps(version, indent=2))

    client_jar = version_dir / "client.jar"
    server_jar = version_dir / "server.jar"
    download(downloads["client"]["url"], client_jar, "sha1", downloads["client"]["sha1"])
    download(downloads["server"]["url"], server_jar, "sha1", downloads["server"]["sha1"])

    libraries = []
    for library in filter(is_library_allowed, version["libraries"]):
        artifact = library["downloads"]["artifact"]
        path = LIBRARIES_DIR / artifact["path"]
        download(artifact["url"], path, "sha1", artifact["sha1"])
        libraries.append(path)
    (version_dir / "classpath.txt").write_text("".join(f"{library}\n" for library in libraries))

    java = ensure_jdk(version["javaVersion"]["majorVersion"])
    if not args.skip_decompile:
        decompile(java, ensure_vineflower(), client_jar, libraries, version_dir / "src")
        for library in version["libraries"]:
            if library["name"].rsplit(":", 1)[0] in SOURCE_LIBRARIES:
                add_library_sources(library, version_dir / "src")
    extract_resources(client_jar, version_dir / "resources")
    run_data_generator(java, server_jar, version_dir)
    print(f"done: {version_dir}")


if __name__ == "__main__":
    main()
