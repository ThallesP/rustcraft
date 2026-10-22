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

Then, on 2026-10-22, the goal:

> I want to re-create the Minecraft Client in Rust, focus on being able to join a Minecraft Server and rendering the world in a real GUI.
>
> Use the decompiled Minecraft Java in here for reference, and feel free to optimize or using any other library for rendering, like using one that does Metal (the apple thing) directly if possible.
>
> For validation, compare against the real Minecraft Java client, for example joining a test server, comparing if something is pixel perfect overtime (like in a video or a screenshot), opening GUIs, comparing network packets to the original client and see if they match and if needed, use the C++ Minecraft Server tests which focus a lot on physics of a Minecraft Bot client compared to the real thing, and maybe even do some of our own if you want to.
>
> The goal here is to get a better optimized version of the Minecraft Client from Java to Rust, like getting even more FPS while being equal to the Java one in everything. Feel free to have agents that compare each one, and see what's missing, a table in the readme describing the features and how the Rust port compares would be really nice.

## How it fits together

- **The port** lives in `src/`, mirroring Java's packages: `src/net/minecraft/util/mth.rs` is `net/minecraft/util/Mth.java`. [PORTING.md](PORTING.md) has the translation rules, including the places where Java and Rust disagree on numbers.
- **Vanilla is the oracle.** The real game runs alongside the port and the outputs get compared, from single functions up to screenshots. [TESTING.md](TESTING.md) has the layers and file formats.
- **The goal** is in [GOAL.md](GOAL.md): join a server and render the world in a real window, then match vanilla in packets, physics, pixels and screens while running faster. The work runs as parallel tracks (net, world, render, and later player, screens, performance and backends). [PROGRESS.md](PROGRESS.md) logs how far each has come.
- **Mojang's code and assets stay out of git.** `tools/setup_vanilla.py` downloads and decompiles them into `vanilla/`, which is gitignored, because they can't be redistributed. The Rust translation is derived from that code too, so think twice before making this repository public.

## Features

How the port compares with vanilla 26.3, area by area. **Checked by** says how the port is compared with the real game ([TESTING.md](TESTING.md)):
- **goldens**: vanilla's own answers to the same calls, matched bit for bit;
- **traces**: tick-by-tick player state while both clients replay a scenario;
- **captures**: the packets each client sends to a vanilla server;
- **screenshots**: pixel-identical frames.

Every step of the loop updates this table.

| Area | Feature | Port | Checked by | vs vanilla |
| --- | --- | --- | --- | --- |
| Math | Java's `Math` (HotSpot's x86-64 `sin`, `cos`, `tan`, `log`, `pow`; FdLibm's `asin`, `acos`, `atan`, `atan2`, `hypot`), `Mth` | Done | goldens | same bits |
| Math | JOML: the `Matrix4f`, `Vector3f`, `Quaternionf`… methods the client calls | Done | goldens | same bits |
| Math | Random sources (legacy, Xoroshiro128++, `nextGaussian`) | Done | goldens | same sequences |
| Core | `Vec3`, `Vec3i`, `BlockPos`, `Direction`, `AABB`, `Identifier`, `PoseStack`, `Transformation`, `ResourceKey`, `Registries` | Done | goldens | |
| Data | NBT: every tag, `NbtIo`, size accounting | Partial: no SNBT `toString`, no `HashMap` tree bins | goldens | |
| Data | Text components, DataFixerUpper codecs | Not started | | |
| Network | Byte layer: VarInt, `FriendlyByteBuf`, stream codecs, frames | Done | goldens | |
| Network | Compression | Done (zlib, through `flate2`) | unit tests, captures (`check-join walk --compression-threshold 256`) | join phase matches vanilla's with compression on |
| Network | Encryption, online-mode login | Not started: the port joins offline-mode servers, and leaves one that asks to encrypt | | |
| Network | Handshake | Done (1 of 1 packets) | goldens, captures | same bytes |
| Network | Server list ping | Not started (0 of 4) | | |
| Network | Login, offline mode | Partial: 10 of 11 packets (disconnect waits for text components, `Unported`) | goldens, captures | same packets |
| Network | Configuration | Partial: 10 of 10 serverbound, 16 of 21 clientbound (disconnect, resource pack push, server links, tags and dialogs are `Unported`) | goldens, captures | same packets |
| Network | Play | Partial: 16 of 144 clientbound, 17 of 69 serverbound; the rest are `Unported`: skipped, and answered by nothing (the world data's packets are the world track's) | goldens (the login packet reads the registries, and vanilla's captured login checks it) | |
| Network | Joining a server: `Connection`, the client's listeners, the tick loop, `--server HOST:PORT` | Partial: joins and stays (keep-alives, pings, known packs, teleports, chunk batches, `player_loaded`), headless; a still player, no movement yet | captures (`tools/port_client.py check-join walk`: `oracle-diff packets --join`) | gate 1: the join phase matches vanilla's, plain and compressed |
| Network | Server addresses: lookups, SRV records, the server blocklist | Partial: lookups like Java's (the handshake names `127.0.0.1` `localhost`); SRV and the blocklist not started | unit tests, captures | |
| World | Blocks and block states (1,286 blocks, 35,723 states) | Partial: every block and state with vanilla's ids, properties, default states and what each keeps from its `Properties` (light, occlusion, map colour, random ticks…); no shapes, fluid states or behaviour yet | goldens (every state), `blocks.json` | same ids, values and flags for all 35,723 states |
| World | Chunk data, level storage, light | Partial: the paletted containers sections keep blocks and biomes in (bit storages, the four palettes, resizes, reading and writing); no sections, chunk packets, level or light yet | goldens | same values and bytes for every bits per entry |
| Player | Input, movement, collisions | Not started | traces | |
| Rendering | Window (SDL3) and input | Partial: the window and OpenGL 3.3 core context made like vanilla's (LWJGL's own SDL3 build), frames presented; no input yet | device as vanilla logs it, on llvmpipe | same driver, flags and extensions as the oracle's log |
| Rendering | renderpearl with its OpenGL backend, shaders through shaderc and SPIRV-Cross | Partial: every pipeline of `RenderPipelines`, compiled through vanilla's own shaderc and SPIRV-Cross builds; the GL device, textures, buffers, render passes that clear, screenshots read back; no draws yet | goldens (SPIR-V, reflection, bindings, GLSL); frames under Xvfb | SPIR-V and GLSL byte-identical for all 192 pipelines |
| Rendering | Metal backend (macOS) | Not started | draw stream checked against OpenGL's | |
| Rendering | Vulkan backend | Not started | screenshots, against vanilla's Vulkan backend | |
| Rendering | Resources, texture atlases, block models | Not started | | |
| Rendering | Terrain, sky, fog, clouds | Not started | screenshots | |
| Rendering | HUD (hotbar, hearts, hunger, crosshair), first-person hand | Not started | screenshots | |
| Screens | Title, pause, inventory, chat, F3 | Not started | screenshots | |
| Entities | Players, mobs, items, block entities | Not started | | |
| Sound | | Not started | | |
| Performance | FPS, frame time, chunk-meshing throughput | Not measured | benchmark scenario | |

## Getting started

Needs Python 3.12+, Rust, about 2 GB of disk, and `xvfb` with Mesa's llvmpipe (`sudo apt install xvfb libgl1-mesa-dri`) for the oracle client and the port's OpenGL tests. The scripts download their own JDK.

```sh
python3 tools/setup_vanilla.py          # download, decompile (authlib too), fetch JOML's, Netty's and Guava's sources, run the data generator
python3 tools/golden.py                 # record vanilla's answers for the golden tests
cargo test                              # check the port against them
python3 tools/oracle_client.py run walk # replay a scenario in the real client
cargo run -- --window                   # a frame of the port in a window (--screenshot out.png renders one offscreen)
cargo run --release -- --server HOST:PORT --username rustcraft   # join a server (offline mode, headless for now)
python3 tools/port_client.py capture walk --accept-eula          # the port's run of a scenario on a fresh vanilla server
python3 tools/port_client.py check-join walk --accept-eula       # gate 1: that run's join phase against vanilla's
```

The client takes vanilla's own options where they exist (`--username`, `--uuid`, `--quickPlayMultiplayer HOST:PORT`, which `--server` is the same as), and `--headless`, which every run that joins a server is until the game draws its level in the window. It stays in the world until the server disconnects it or it's stopped, and exits 1 with the reason when the server disconnects it.

`MINECRAFT_VERSION` pins the game version. Pass another version to `setup_vanilla.py` to study an update next to it.

| Path | What it is |
| --- | --- |
| `src/` | The port. `src/java/` holds JDK behaviour the game relies on (`Math`, `String.hashCode`, `UUID`), `src/org/joml/` the parts of JOML, the game's vector and matrix library, that the client calls, `src/io/netty/` the parts of Netty's buffers, `src/com/google/` Guava's, `src/com/mojang/authlib/` the player profiles of Mojang's authlib, and `src/org/lwjgl/` LWJGL's bindings to shaderc, SPIRV-Cross and SDL3, whose libraries come from vanilla's natives jars at runtime. `src/com/mojang/renderpearl/` is the renderer. |
| `oracle/golden/` | Java harness that calls real game methods and records the answers |
| `oracle/client/` | Fabric mod that drives the real client headless: tick traces and screenshots |
| `oracle/scenarios/` | Scripted inputs both clients replay |
| `tools/packet-proxy/` | Records client ↔ server traffic with each packet decoded |
| `tools/oracle-diff/` | Compares traces, packet captures and screenshots between two runs |
| `tools/*.py` | Setup, golden recording, vanilla server and oracle client runners, and a check for ports of HotSpot's assembly |
| `vanilla/` (gitignored) | Decompiled source, assets, data reports, jars, JDK, oracle outputs |
