<img src="assets/icon.svg" width="64" height="64" alt="">

# rustcraft

The Minecraft Java Edition client (26.3), ported to Rust file by file from the decompiled game and checked against the real game at every step.

Created in [T3 Code](https://t3.codes).

## The prompt

This project started from this prompt:

> I want to re-create the Minecraft Client in Rust, wondering if we can setup the environment first.
>
> can we get the decompiled version of Minecraft Java (the latest version), I was thinking of doing a Java to Rust translation, but keeping the Java mannerism (like folder structures and etc).
>
> But i'm also scared of you getting some parts wrong, and even when I tell you to optimize properly, so I'm wondering what we can do here to fix it, maybe screenshots comparison (in a pixel perfect way?), comparing network packets to the original client and maybe even tests from the minecraft server client in C++
>
> what do you think here?

## How it fits together

- **The port** lives in `src/`, mirroring Java's packages: `src/net/minecraft/util/mth.rs` is `net/minecraft/util/Mth.java`. [PORTING.md](PORTING.md) has the translation rules, including the places where Java and Rust disagree on numbers.
- **Vanilla is the oracle.** The real game runs alongside the port and the outputs get compared, from single functions up to screenshots. [TESTING.md](TESTING.md) has the layers and file formats.
- **The goal** is in [GOAL.md](GOAL.md): the `walk` scenario matching vanilla pixel for pixel. [PROGRESS.md](PROGRESS.md) logs how far the port has come.
- **Mojang's code and assets stay out of git.** `tools/setup_vanilla.py` downloads and decompiles them into `vanilla/`, which is gitignored, because they can't be redistributed. The Rust translation is derived from that code too, so think twice before making this repository public.

## Getting started

Needs Python 3.12+, Rust, about 2 GB of disk, and `xvfb` for the oracle client. The scripts download their own JDK.

```sh
python3 tools/setup_vanilla.py          # download, decompile, fetch JOML's and Netty's sources, run the data generator
python3 tools/golden.py                 # record vanilla's answers for the golden tests
cargo test                              # check the port against them
python3 tools/oracle_client.py run walk # replay a scenario in the real client
```

`MINECRAFT_VERSION` pins the game version. Pass another version to `setup_vanilla.py` to study an update next to it.

| Path | What it is |
| --- | --- |
| `src/` | The port. `src/java/` holds JDK behaviour the game relies on (`Math`, `String.hashCode`, `UUID`), `src/org/joml/` the parts of JOML, the game's vector and matrix library, that the client calls, and `src/io/netty/` the parts of Netty's buffers. |
| `oracle/golden/` | Java harness that calls real game methods and records the answers |
| `oracle/client/` | Fabric mod that drives the real client headless: tick traces and screenshots |
| `oracle/scenarios/` | Scripted inputs both clients replay |
| `tools/packet-proxy/` | Records client ↔ server traffic with each packet decoded |
| `tools/oracle-diff/` | Compares traces, packet captures and screenshots between two runs |
| `tools/*.py` | Setup, golden recording, vanilla server and oracle client runners, and a check for ports of HotSpot's assembly |
| `vanilla/` (gitignored) | Decompiled source, assets, data reports, jars, JDK, oracle outputs |
