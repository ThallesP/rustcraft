# Goal

Re-create the vanilla Minecraft Java client (the version in `MINECRAFT_VERSION`) in Rust, until the `walk` scenario matches vanilla exactly.

## Done means

The port joins a fresh vanilla server through `packet-proxy`, replays `oracle/scenarios/walk.json`, and writes its run to `target/oracle/walk/`. Then:

```sh
python3 tools/oracle_client.py capture walk --accept-eula            # vanilla's run → vanilla/<version>/oracle/walk/
cargo run -p oracle-diff -- dir vanilla/<version>/oracle/walk target/oracle/walk
```

exits 0 with the default settings:
- the tick trace is identical;
- client packets match: the join phase as a multiset, strictly from the scenario-start marker;
- `spawn.png` and `after-walk.png` are pixel-identical (threshold 0).

## Rules

- Follow PORTING.md and TESTING.md. Port from `vanilla/<version>/src/` file by file, keeping Java's layout. Check every pure class with golden tests before building on it.
- Never weaken a check to make it pass:
  - no image threshold above 0;
  - no new entries in oracle-diff's ignore lists;
  - no `pending` without a real reason;
  - no editing generated golden data;
  - no changes to the oracle client that only make vanilla's output easier to match.

  When vanilla's output looks odd, it's still the reference. Changing a check is fine only when the check itself was wrong; say why in the commit and in PROGRESS.md.
- Never commit anything from `vanilla/`: no decompiled code, no assets. The port loads assets from `vanilla/<version>/resources/` (later from the player's own install) at runtime.
- Render the way the oracle does: OpenGL on Mesa's llvmpipe under Xvfb, with the game's own shaders from `resources/assets/minecraft/shaders/`. Pixel-identical output needs the same rasterizer, shaders, vertex data, draw order, texture atlas layout and matrix math as vanilla's renderpearl OpenGL backend.
- Every step ends green and pushed:
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass;
  - it's committed and pushed to `main`;
  - CI passes.
- PROGRESS.md is the loop's memory. Read it first. Update it last, with what was done, the evidence, what's next and what's blocked.
- System packages may be installed with `sudo apt`. List them in README.md.

## Milestones

Suggested order. Split any of them into steps small enough to finish and verify in one session.

1. **Exact math.**
   - Port HotSpot's x86-64 `Math` routines (sin, cos, tan, log, exp, pow…, from Intel's libm in OpenJDK's `macroAssembler_x86_*.cpp`) into `src/java/lang/math.rs`, until the pending golden entries pass.
   - Port the JOML classes the client uses (`Matrix4f`, `Vector3f`, `Quaternionf`…), checked by goldens.
2. **Core types:** `Vec3`, `Vec3i`, `BlockPos`, `Direction`, `AABB`, `Identifier`. Extend the golden harness to cover instance methods of value types.
3. **Networking.**
   - Port `FriendlyByteBuf`, the stream codecs, `Connection` and the protocol states.
   - Join the vanilla server through `packet-proxy` and send the scenario's setup commands and the scenario-start marker, like the oracle client does.
   - Add a runner that does for the port what `oracle_client.py capture` does for vanilla.
   - Check: `oracle-diff packets` on the join phase.
4. **World data:** registry sync, block states, and decoding chunk packets into level storage.
5. **Player.** Input, `LocalPlayer` movement and collision, replaying the scenario's steps. Check: `oracle-diff trace` is identical.
6. **Rendering.**
   - A window at the scenario's size, resource loading, the texture atlas, block models, chunk meshing, and sky and fog.
   - Then the HUD: hotbar, hearts, hunger, crosshair, block outline.
   - Then the first-person hand.
   - Check: screenshots, using `oracle-diff image --heatmap` to find what differs. Add narrower scenarios when they help, e.g. sky only, or HUD only.
7. **Pixel-perfect:** `oracle-diff dir` exits 0.

## When stuck

- If a check fails for a reason outside the port (the oracle being nondeterministic, a tool bug), fix the tool, document it in TESTING.md, and note it in PROGRESS.md.
- If a decision only Thalles can make blocks the way forward, add it under "Needs Thalles" in PROGRESS.md and work on something else in the meantime.
