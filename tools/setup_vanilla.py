#!/usr/bin/env python3
"""Fetch a vanilla Minecraft version and turn it into a porting reference.

Downloads the client and server jars plus the client's libraries, decompiles the
client with Vineflower, adds the sources of the libraries the port translates too
(JOML, Netty's buffers and Guava, or authlib and DataFixerUpper decompiled, since they
publish no sources), extracts its bundled resources, lays out the launcher's assets the
client reads (below) and runs the server's data generator.
Everything lands in vanilla/, which is gitignored: Mojang's code and assets can't
be redistributed.

    python3 tools/setup_vanilla.py           # the version pinned in MINECRAFT_VERSION
    python3 tools/setup_vanilla.py 26.4      # another version, e.g. to diff an update
    python3 tools/setup_vanilla.py --skip-decompile   # just what the golden tests need (CI)
    python3 tools/setup_vanilla.py --no-jdk  # just what running the port needs: no JDK, decompile or reports
    python3 tools/setup_vanilla.py --skip-decompile --natives macos-arm64   # and a Mac's natives jars too

It runs on Linux and macOS. The libraries are the ones Mojang's launcher puts on this platform's
classpath (version.json's rules), listed in vanilla/<version>/classpath.txt, natives jars
included; --natives also fetches another platform's (LWJGL's names: linux, macos, macos-arm64,
windows…) and lists them in classpath-<platform>.txt, so the port's tests can check what that
platform's loader would find. The JDK is Adoptium's for this platform; only the decompile, the
data generator, the golden harness and the oracle need it, not the port.

The launcher's asset index has some of the client's resources that the jar only has placeholders
of (the title screen's panorama): vanilla/<version>/assets/ holds the index's textures under their
names, the directory vanilla reads as its external asset source when it's started without
--assetIndex. Its fonts (unifont, whose unihex provider the port hasn't got yet), sounds and other
languages aren't fetched.
"""

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
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
ADOPTIUM_URL = "https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture={arch}&image_type=jdk&os={os}&vendor=eclipse"
VINEFLOWER_VERSION = "1.12.0"
VINEFLOWER_SHA256 = "1dfcfe974395734fa467ce620661c7623d05ba83670de0529b1fbd63ff548b9d"
VINEFLOWER_URL = f"https://repo1.maven.org/maven2/org/vineflower/vineflower/{VINEFLOWER_VERSION}/vineflower-{VINEFLOWER_VERSION}.jar"
MAVEN_CENTRAL = "https://repo1.maven.org/maven2/"
# Libraries the port translates like the game's own classes, from their published sources.
# Gson's JsonReader and JsonWriter read and write text components' JSON (LenientJsonParser, the status).
SOURCE_LIBRARIES = {"org.joml:joml", "io.netty:netty-buffer", "com.google.guava:guava", "com.google.code.gson:gson"}
# Ones it translates that publish no sources, so they're decompiled like the game: authlib, DataFixerUpper, whose
# codecs read and write text components and the vanilla pack's JSON, and Brigadier, whose reader parses the selectors
# and NBT paths text components hold.
DECOMPILED_LIBRARIES = {"com.mojang:authlib", "com.mojang:datafixerupper", "com.mojang:brigadier"}


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
# side by side with vanilla/<version>/src/org/joml/Matrix4f.java, and src/io/netty/buffer/
# with vanilla/<version>/src/io/netty/buffer/.
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


# Decompiles a library into the game's sources: authlib's GameProfile lands next to the game's
# com/mojang/ packages, at vanilla/<version>/src/com/mojang/authlib/GameProfile.java.
def decompile_library(java, vineflower, library_jar, libraries, source_dir):
    partial = source_dir.with_name(source_dir.name + f".{library_jar.stem}.part")
    shutil.rmtree(partial, ignore_errors=True)
    print(f"decompiling {library_jar} into {source_dir}")
    subprocess.run(
        [
            java, "-jar", vineflower,
            "--folder", "--skip-extra-files", "--indent-string=    ", "--log-level=warn",
            *(f"-e={library}" for library in libraries if library != library_jar),
            library_jar, partial,
        ],
        check=True,
    )
    shutil.copytree(partial, source_dir, dirs_exist_ok=True)
    shutil.rmtree(partial)


def resolve_version(version_id):
    manifest = fetch_json(VERSION_MANIFEST_URL)
    entry = next((version for version in manifest["versions"] if version["id"] == version_id), None)
    if entry is None:
        raise SystemExit(f"unknown version {version_id}")
    return fetch_json(entry["url"])


# LWJGL's names for the platforms vanilla ships natives jars for (`lwjgl-sdl-3.4.3-natives-macos-arm64.jar`),
# with the launcher's name for the OS its rules match. A Mac's classpath holds both its jars, the
# x64 one (`natives-macos`) and the arm64 one, and LWJGL finds its architecture's library in them.
NATIVES_PLATFORMS = {
    "linux": "linux",
    "macos": "osx",
    "macos-arm64": "osx",
    "windows": "windows",
    "windows-arm64": "windows",
}


def host_platform():
    os_name = {"linux": "linux", "darwin": "osx", "win32": "windows"}[sys.platform]
    arch = {"x86_64": "x86_64", "amd64": "x86_64", "aarch64": "arm64", "arm64": "arm64"}[platform.machine().lower()]
    return os_name, arch


# Mojang's launcher rules: no rules means allowed, otherwise the last matching rule wins. A rule's
# `arch` only ever names 32-bit x86, which no platform here is.
def is_library_allowed(library, os_name):
    allowed = "rules" not in library
    for rule in library.get("rules", []):
        rule_os = rule.get("os", {})
        if rule_os.get("name", os_name) == os_name and "arch" not in rule_os:
            allowed = rule["action"] == "allow"
    return allowed


# Downloads the libraries the launcher puts on os_name's classpath, and lists them in a classpath file.
def fetch_libraries(version, os_name, classpath_file):
    libraries = []
    for library in version["libraries"]:
        if not is_library_allowed(library, os_name):
            continue
        artifact = library["downloads"]["artifact"]
        path = LIBRARIES_DIR / artifact["path"]
        download(artifact["url"], path, "sha1", artifact["sha1"])
        libraries.append(path)
    classpath_file.write_text("".join(f"{library}\n" for library in libraries))
    return libraries


def ensure_jdk(major):
    jdk_dir = TOOLCHAIN_DIR / f"jdk-{major}"
    java = jdk_dir / "bin" / "java"
    if java.exists():
        return java

    os_name, arch = host_platform()
    if os_name == "windows":
        raise SystemExit("no JDK download for Windows here: pass --no-jdk, or put a JDK in " + str(jdk_dir))
    adoptium_os = {"linux": "linux", "osx": "mac"}[os_name]
    adoptium_arch = {"x86_64": "x64", "arm64": "aarch64"}[arch]
    package = fetch_json(ADOPTIUM_URL.format(major=major, arch=adoptium_arch, os=adoptium_os))[0]["binary"]["package"]
    archive = TOOLCHAIN_DIR / package["name"]
    download(package["link"], archive, "sha256", package["checksum"])

    with tarfile.open(archive) as tar:
        top_level = tar.getnames()[0].split("/")[0]
        tar.extractall(TOOLCHAIN_DIR, filter="data")
    # macOS's JDK is a bundle, its home in Contents/Home: the tools read vanilla/toolchain/jdk-<major>/bin.
    bundle_home = TOOLCHAIN_DIR / top_level / "Contents" / "Home"
    if bundle_home.is_dir():
        bundle_home.rename(jdk_dir)
        shutil.rmtree(TOOLCHAIN_DIR / top_level)
    else:
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


# The asset index's objects the client reads, by name: what vanilla opens through IndexedAssetSource.
ASSET_PREFIXES = ("minecraft/textures/",)


def fetch_assets(version, assets_dir):
    index_info = version["assetIndex"]
    index_path = assets_dir.with_name("assets-index.json")
    download(index_info["url"], index_path, "sha1", index_info["sha1"])
    objects = json.loads(index_path.read_text())["objects"]
    for name, info in sorted(objects.items()):
        if name.startswith(ASSET_PREFIXES):
            digest = info["hash"]
            download(f"https://resources.download.minecraft.net/{digest[:2]}/{digest}", assets_dir / name, "sha1", digest)


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
    parser.add_argument(
        "--no-jdk", action="store_true",
        help="skip the JDK and what needs it (the decompile, the data generator): what running the port needs",
    )
    parser.add_argument(
        "--natives", action="append", default=[], choices=sorted(NATIVES_PLATFORMS), metavar="PLATFORM",
        help="also fetch another platform's libraries, listed in classpath-PLATFORM.txt (repeatable: "
        + ", ".join(sorted(NATIVES_PLATFORMS)) + ")",
    )
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

    os_name, _ = host_platform()
    libraries = fetch_libraries(version, os_name, version_dir / "classpath.txt")
    for natives in args.natives:
        fetch_libraries(version, NATIVES_PLATFORMS[natives], version_dir / f"classpath-{natives}.txt")
    extract_resources(client_jar, version_dir / "resources")
    fetch_assets(version, version_dir / "assets")
    if args.no_jdk:
        print(f"done: {version_dir} (no JDK: the port runs, the golden harness and the oracle need one)")
        return

    java = ensure_jdk(version["javaVersion"]["majorVersion"])
    if not args.skip_decompile:
        decompile(java, ensure_vineflower(), client_jar, libraries, version_dir / "src")
        for library in version["libraries"]:
            if library["name"].rsplit(":", 1)[0] in SOURCE_LIBRARIES:
                add_library_sources(library, version_dir / "src")
            if library["name"].rsplit(":", 1)[0] in DECOMPILED_LIBRARIES:
                library_jar = LIBRARIES_DIR / library["downloads"]["artifact"]["path"]
                decompile_library(java, ensure_vineflower(), library_jar, libraries, version_dir / "src")
    run_data_generator(java, server_jar, version_dir)
    print(f"done: {version_dir}")


if __name__ == "__main__":
    main()
