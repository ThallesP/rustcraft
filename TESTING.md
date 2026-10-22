# Testing against vanilla

The real game is the source of truth. Every check runs vanilla and the port on the same input and compares the outputs, from most precise to least:

| Layer | Vanilla side | Compared by |
| --- | --- | --- |
| 1. Function goldens | `oracle/golden/` calls real methods from `client.jar` | `cargo test` |
| 2. Tick traces | `oracle/client/` (a Fabric mod) records the player every tick of a scenario | `oracle-diff trace` |
| 3. Packets | `packet-proxy` records client ↔ vanilla server traffic | `oracle-diff packets` |
| 4. Screenshots | `oracle/client/` takes screenshots at scenario checkpoints | `oracle-diff image` |

Draw-call streams from the renderer (renderpearl) will become layer 5 once the port renders.

Everything vanilla produces lands in `vanilla/<version>/`, which is gitignored. `MINECRAFT_VERSION` pins the version.

## Commands

```sh
python3 tools/setup_vanilla.py                                    # once per version
python3 tools/golden.py && cargo test                             # layer 1
python3 tools/oracle_client.py run walk                           # layers 2 and 4, singleplayer
python3 tools/oracle_client.py capture walk --accept-eula         # layers 2–4 on a fresh vanilla server
python3 tools/oracle_client.py check-determinism walk --capture --accept-eula
cargo run -p oracle-diff -- dir vanilla/26.3/oracle/walk target/oracle/walk
```

`--accept-eula` accepts Mojang's EULA (https://aka.ms/MinecraftEULA) for the local server. Running a server requires it.

## Encoding rules

Shared by every file below:

- Floats are the lowercase hex of their raw IEEE 754 bits, as JSON strings: 8 digits for `float`/`f32`, 16 for `double`/`f64` (`1.0f` → `"3f800000"`). Comparisons are bit-exact, so `-0.0 != 0.0`. The one exception is that any NaN matches any NaN.
- `int`, `long`, `short` and `byte` are JSON numbers. A `char` is a number (its UTF-16 code unit). A `boolean` is a JSON bool.
- A `String` is a JSON string. Arrays and Java records are JSON arrays (records in component order). A `UUID` is `[most, least]`.
- Files are JSON Lines: one object per line.

## 1. Function goldens

`python3 tools/golden.py` compiles `oracle/golden/` against the client jar and writes one file per class to `vanilla/<version>/golden/<binary class name>.jsonl`, e.g. `net.minecraft.util.Mth.jsonl`.

Static methods are covered by reflection. Every public static method whose parameter and return types have a generator gets:
- edge cases (zeros, signed zeros, NaN, infinities, MIN/MAX, angles, powers of two, non-ASCII strings);
- 500 seeded random inputs;
- for `Mth.sin`/`cos`, every entry of their lookup table;
- for `Math.sin`/`cos`/`tan`/`log`/`pow`, which HotSpot computes with its own assembly, 20,000 more inputs each, spread over every path through it, since last-bit differences between libms are rare;
- for `Mth.atan2`, angles that read every entry of its tables.

A `RandomSource` argument is a `LegacyRandomSource` seeded with the argument. Its next `nextLong()` after the call is recorded in `after` (one slot per argument), so how many values a method draws must match too. Stateful classes get hand-written suites (`RandomSourceSuite`).

```json
{"field":"PI","value":"40490fdb"}
{"method":"sin(D)F","args":["3ff0000000000000"],"result":"3f57695c"}
{"method":"floorDiv(II)I","args":[0,0],"throws":"java.lang.ArithmeticException"}
{"method":"nextInt(Lnet/minecraft/util/RandomSource;II)I","args":[42,90,-525],"after":[-5025562857975149833,null,null],"result":90}
{"method":"lerp(DLnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;","skipped":"no generator for net.minecraft.world.phys.Vec3"}
```

`method` is the JVM name and descriptor, which keeps overloads apart. When vanilla throws, the port must panic.

- **Pending ledger.** Each ported file ends with a test that checks every method and constant in its golden file, or marks it pending with a reason. Anything neither checked nor pending fails the test, so a class is either fully verified or says what's left.
- **Methods that hang.** Some vanilla methods never return on extreme inputs. A watchdog names the call, and the suite keeps the inputs in range with `where()`.

## 2. Scenarios and tick traces

A scenario, `oracle/scenarios/<name>.json`, is the input both clients replay:

```json
{
  "world": { "seed": 0, "preset": "flat", "time": 6000, "game_mode": "survival" },
  "player": { "pos": [0.5, -60.0, 0.5], "y_rot": 0.0, "x_rot": 0.0 },
  "options": { "window": [854, 480], "fov": 70, "gui_scale": 2, "render_distance": 4 },
  "steps": [
    { "ticks": 20, "keys": ["forward"] },
    { "ticks": 1, "keys": ["forward", "jump"] },
    { "ticks": 10, "keys": [], "look": [90.0, 10.0] },
    { "screenshot": "after-walk" }
  ]
}
```

- `keys` are held for every tick of the step: `forward`, `back`, `left`, `right`, `jump`, `sneak`, `sprint`, `attack`, `use`.
- `look` sets `[y_rot, x_rot]` in degrees at the start of every tick of the step.
- `screenshot` captures the current frame as `<name>.png` without advancing the game.

Before tick 0, both modes put the world in the scenario's state with the same commands. A port joining a server must send the same ones:

```
gamerule advance_time false
gamerule advance_weather false
gamerule random_tick_speed 0
gamerule spawn_mobs false
difficulty normal
time set <time>
weather clear
gamemode <game_mode> @s
tp @e[type=!minecraft:player] 0 -1000 0
tp @s <x> <y> <z> <y_rot> <x_rot>
```

Then the client waits for chunks to render, plus at least 20 ticks until the player is on the ground. The player is the offline player `rustcraft`.

On a server, the client then sends a marker right before tick 0: a serverbound `minecraft:custom_payload` with the identifier `rustcraft:scenario_start` and no data (payload `18` followed by the identifier's ASCII). A vanilla server ignores it. Packet diffs use it to tell the join phase from the scenario, so a port joining a server must send it at the same point.

A run writes `<out>/<scenario>/trace.jsonl`, one line per client tick after the tick ran:

```json
{"tick":0,"pos":["…","…","…"],"delta":["…","…","…"],"y_rot":"…","x_rot":"…","on_ground":true,"horizontal_collision":false,"vertical_collision":true,"sprinting":false,"crouching":false,"pose":"STANDING"}
```

`pos` and `delta` (`getDeltaMovement`) are doubles, and the rotations are floats. Screenshots go to `<out>/<scenario>/screenshots/<name>.png`.

## 3. Packet captures

`packet-proxy` sits between a client and a vanilla server in offline mode. It writes `capture.jsonl`, one line per packet after framing and decompression:

```json
{"seq":0,"connection":0,"ms":0,"direction":"serverbound","state":"handshake","id":0,"name":"minecraft:intention","payload":"…hex…"}
```

`state` follows the protocol (`handshake`, `status`, `login`, `configuration`, `play`). Names come from `vanilla/<version>/generated/reports/packets.json`. `connection` counts connections in the order they were accepted. `ms` is for humans; diffs ignore it.

`tools/vanilla_server.py` gives every run a fresh world with fixed settings:
- offline mode, no compression, no whitelist;
- flat world with the scenario's seed, no structures, peaceful;
- `respawn_radius 0`, otherwise the spawn point is random;
- no time, weather or mob cycles;
- `rustcraft` as an operator, written to `ops.json` with its offline UUID.

`oracle-diff packets` compares the two directions differently, because the vanilla server can't produce byte-identical output twice:

- **Client → server packets** (the client's behaviour) are compared in two parts, split at the scenario-start marker:
  - *From the marker on*, they must match strictly: same order, same bytes.
  - *Before it* is the join and setup. The server runs in real time while the client is stepped one tick at a time, so how many ticks pass before the world is loaded varies. Per-tick packets (`client_tick_end`, `move_player_*`, `player_input`) are dropped, and the rest are compared as a multiset with exact payloads. A missing or changed `chat_command`, `client_information` or `accept_teleportation` still shows up.
  - A marker in only one capture is a difference. With no marker in either capture, the whole stream is strict.
- **Server → client packets** are compared as a multiset. Payloads are compared regardless of order, because the server writes tags, recipes, commands and chunk heightmaps in Java `HashMap` order, which changes from run to run. `--ordered` makes the comparison strict.
- **Ignored by default** (`--no-default-ignore` turns this off) are packets that differ in every run:
  - `keep_alive`, `ping`/`pong`, `ping_request`/`pong_response`: random ids and clocks;
  - `login_finished`: a random session id;
  - `set_time`: sent on a timer;
  - `update_advancements`: timestamps;
  - `chunk_batch_*`: pacing based on the client's clock.

## 4. Screenshots

The oracle client renders with OpenGL on Mesa's llvmpipe under Xvfb, so its output doesn't depend on the GPU. It turns off the vignette, view bobbing, vsync and clouds, sets particles to minimal, and uses Fabric's test defaults (no anisotropic filtering, no chunk fade, no music, no tutorial). The port must render with the same settings for screenshots to be comparable. `oracle-diff image` reports how many pixels differ and the largest channel difference, and can write a heatmap.

## Where outputs go

- Vanilla runs: `vanilla/<version>/oracle/<scenario>/`
- Port runs: `target/oracle/<scenario>/`

`oracle-diff dir A B` compares the trace, the capture and every screenshot that both directories have. Running vanilla twice and diffing the runs (`check-determinism`) checks that the oracle itself is deterministic. It has to be, or no comparison means anything.
