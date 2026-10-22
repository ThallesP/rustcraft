# Goal

Re-create the vanilla Minecraft Java client (the version in `MINECRAFT_VERSION`) in Rust. First, it joins a Minecraft server and renders the world in a real window. Then it matches vanilla in everything the oracle can check (packets, physics, pixels, screens) while running faster.

Thalles's brief (2026-10-22), in full in README.md: join a server and render the world in a real GUI, using the decompiled game as the reference. Rendering libraries are free to choose: Metal directly on Apple, for instance. Compare against the real client: screenshots over time, opening GUIs, packets, and physics, with the C++ bot-client tests if they help. Aim for more FPS while staying equal to Java in everything. Have agents compare, find what's missing, and keep a feature table in the README.

## Done means

Gates, roughly in order. Each one stays green once reached.

1. **Join.** `cargo run --release -- --server HOST:PORT` joins a vanilla server and stays in the world: keep-alives, pings, chunk batches and teleports are answered like vanilla answers them. `tools/port_client.py capture walk --accept-eula` records the port's run the way `oracle_client.py capture` records vanilla's, into `target/oracle/walk/`. `oracle-diff packets` passes on the join phase.
2. **The world in a window.** The port opens a real window (SDL3, like vanilla) and renders the world it received: terrain, sky and fog, then the HUD and the hand. Mouse look and WASD/jump/sneak/sprint work, so a person can walk around in it.
3. **Walk, pixel for pixel.** The port replays `oracle/scenarios/walk.json`, and `cargo run -p oracle-diff -- dir vanilla/<version>/oracle/walk target/oracle/walk` exits 0 with the default settings:
   - the tick trace is identical;
   - client packets match: the join phase as a multiset, strictly from the scenario-start marker;
   - every screenshot is pixel-identical (threshold 0).
4. **More of the game, still identical.** More scenarios, each passing `oracle-diff dir`:
   - screenshots over time, not just stills: a screenshot every few ticks while walking, turning and jumping;
   - screens: the inventory, the pause menu, chat, F3;
   - physics: collisions, steps, stairs and slabs, jumping onto blocks, sprint-jumping, sneaking at an edge, water, ladders. The vanilla client's tick trace is the physics oracle. The C++ bot-client tests the brief mentions can supply the cases.
5. **Faster.** A benchmark scenario runs the same scene with the same settings in both clients and records frame times (CPU per frame, and FPS on the GPU at hand) and chunk-meshing throughput. The port beats vanilla, and README.md shows the numbers and the machine they come from.
6. **The feature table.** README.md has a table of the client's features: whether the port has each one, how it's checked against vanilla, and how it compares. Every step updates it. A comparison agent checks it against vanilla's source now and then and adds what's missing.

## Rendering backends

- **OpenGL is the reference backend.** It renders the way the oracle does: OpenGL on Mesa's llvmpipe under Xvfb, with the game's own shaders. Vanilla 26.3 compiles its GLSL to SPIR-V with shaderc, then translates that SPIR-V to GLSL 330 with SPIRV-Cross for OpenGL (`com/mojang/renderpearl/frontend/shaders/GlslCompiler.java`, `backend/opengl/GlPipelineRecompiler.java`). The port does the same through the very same native libraries: the `libshaderc` and `libspirv-cross` from vanilla's LWJGL natives jars (`vanilla/libraries/org/lwjgl/lwjgl-{shaderc,spvc}/`). That makes its shader text identical to vanilla's. Pixel-identical output also needs the same vertex data, draw order, pipeline state, texture atlas layout and matrix math as vanilla's renderpearl OpenGL backend.
- **Other backends are optimizations.**
  - A **Vulkan** backend can be checked pixel for pixel against vanilla's own Vulkan backend (`backend/vulkan/`), with both on Mesa's lavapipe.
  - A **Metal** backend (SPIRV-Cross also writes MSL) has no vanilla counterpart. It's checked by rendering the same draw stream as the OpenGL backend, which is checked against vanilla. Its screenshots are compared with vanilla's on the same Mac and the difference is reported, but it's no gate.
- Port renderpearl's split: `api` (what the game calls), `frontend` and `backend`. The game code then stays the same whatever the backend.

## Tracks

The work runs as parallel tracks. Each has its own git worktree under `../rustcraft-tracks/<track>`, a branch `track/<track>`, and a section in PROGRESS.md's "Next". A step rebases on `origin/main`, ends green, and pushes to `main`.

- **net**: the protocol up to and through play, `Connection`, the runner that joins a server. Gate 1.
- **world**: blocks and block states, chunk data, `ClientLevel` and its chunk storage, light. Feeds the renderer and physics.
- **render**: the window, renderpearl with its OpenGL backend, resources, the texture atlas, block models, chunk meshing, sky, fog, then HUD and hand. Gates 2 and 3.
- Later: **player** (input and `LocalPlayer` physics, gate 3's trace and gate 4's physics), **gui** (screens), **perf** (gate 5) and **backends** (Metal, Vulkan).

## Rules

- Follow PORTING.md and TESTING.md. Port from `vanilla/<version>/src/` file by file, keeping Java's layout. Check every ported class against vanilla (goldens, traces, captures or screenshots) before building on it.
- **Breadth first.** Port what the next gate needs. A play packet the port can't decode yet is skipped by its frame (`Unported` in the protocol, named in README's table), as long as vanilla sends no answer to it. Every packet vanilla answers gets ported. Match the depth of each check to the risk: exhaustive goldens for math, codecs, meshing and physics, enough for glue code. Mutation runs are a tool to use when a suite's coverage is in doubt, not a step every pass must take.
- Never weaken a check to make it pass:
  - no image threshold above 0;
  - no new entries in oracle-diff's ignore lists;
  - no `pending` without a real reason;
  - no editing generated golden data;
  - no changes to the oracle client that only make vanilla's output easier to match.

  When vanilla's output looks odd, it's still the reference. Changing a check is fine only when the check itself was wrong; say why in the commit and in PROGRESS.md.
- Never commit anything from `vanilla/`: no decompiled code, no assets, no natives. The port loads assets from `vanilla/<version>/resources/` and natives from `vanilla/libraries/` at runtime (later from the player's own install).
- The machine is shared by the tracks. Wrap anything that starts a vanilla server, the oracle client or a capture in `flock /tmp/rustcraft-oracle.lock …`. Never kill processes you didn't start.
- Every step ends green and pushed:
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and `python3 tools/hotspot_port_check.py` pass;
  - it's committed, rebased on `origin/main` and pushed to `main`;
  - CI passes.
- PROGRESS.md is the loop's memory. Read it first. Update it last, with what was done, the evidence, what's next on your track and what's blocked. Keep README.md's feature table true.
- System packages may be installed with `sudo apt`. List them in README.md.

## When stuck

- If a check fails for a reason outside the port (the oracle being nondeterministic, a tool bug), fix the tool, document it in TESTING.md, and note it in PROGRESS.md.
- If a decision only Thalles can make blocks the way forward, add it under "Needs Thalles" in PROGRESS.md and work on something else in the meantime.
