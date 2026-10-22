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

- Floats are the lowercase hex of their raw IEEE 754 bits, as JSON strings: 8 digits for `float`/`f32`, 16 for `double`/`f64` (`1.0f` → `"3f800000"`). Comparisons are bit-exact, so `-0.0 != 0.0`. The one exception is that any NaN matches any NaN: NaN bits even differ between two runs of the golden generator, since the same operation can give NaNs of either sign depending on when HotSpot's JIT compiles it, which varies from run to run.
- `int`, `long`, `short` and `byte` are JSON numbers. A `char` is a number (its UTF-16 code unit). A `boolean` is a JSON bool.
- A `String` is a JSON string. Arrays and Java records are JSON arrays (records in component order). A `UUID` is `[most, least]`.
- Files are JSON Lines: one object per line.

## 1. Function goldens

`python3 tools/golden.py` compiles `oracle/golden/` against the client jar and writes one file per class to `vanilla/<version>/golden/<binary class name>.jsonl`, e.g. `net.minecraft.util.Mth.jsonl`.

Static methods are covered by reflection. Every public static method whose parameter and return types have a generator (or that returns `void`) gets:
- edge cases (zeros, signed zeros, NaN, infinities, MIN/MAX, angles, powers of two, non-ASCII strings);
- 500 seeded random inputs;
- for `Mth.sin`/`cos`, every entry of their lookup table;
- for `Math.sin`/`cos`/`tan`/`log`/`pow`, which HotSpot computes with its own assembly, 20,000 more inputs each, spread over every path through it, since last-bit differences between libms are rare;
- for `Mth.atan2`, angles that read every entry of its tables;
- for `Math.hypot`, 6,000 more spread over every path through fdlibm's code, with its thresholds and the point where it switches formulas;
- for `MatrixUtil.eigenvalueJacobi` and `svdDecompose`, matrices like the ones `Transformation` decomposes, and ones where `approxGivensQuat` switches to a quarter turn. `eigenvalueJacobi` takes at most 1,024 steps (`where()`): billions would take minutes, and the game takes 5.

A `RandomSource` argument is a `LegacyRandomSource` seeded with the argument. Its next `nextLong()` after the call is recorded in `after` (one slot per argument), so how many values a method draws must match too. A static method taking it first is checked with `golden.check_random`; anything else (`Vec3.offsetRandom`) with `golden.check_mut`, to which `golden.rs` passes a `LegacyRandomSource` decoded from the seed and encoded as its next `nextLong()`. Any other argument a method wrote to gets its state afterwards in its slot (`MatrixUtil.checkProperty` adds to its matrix's properties). An empty slot means the argument didn't change, or is the returned object, which `result` already shows; the port's argument must then equal what it was or the result. A `void` method's result is null, so `after` is all it shows (`svdDecompose` writes its matrix's parts into four arguments). The port checks those with `golden.check_mut`, whose function takes every argument as `&mut`.

Stateful classes get hand-written suites, which run seeded random programs of ops and record a step per op. A program ends at the first op that throws, recorded as `{"throws": …}`:
- `RandomSourceSuite`: each op's output, for every `RandomSource` implementation.
- `PoseStackSuite`: `[output, pose, normal, trustedNormals]`, the op's output and the top pose afterwards (`trustedNormals` is read by reflection). One program in four draws any float, NaN included. The last 256 programs also multiply by transformations, whose output is the transformation afterwards: `mulPose` adds to its matrix's properties.
- `Matrix4fStackSuite`: the current matrix after each op, for stack sizes from 1 to 20 and a few the constructor rejects.
- `Matrix3x2fStackSuite`: the same for the GUI's pose stack, with the ops the client calls on it (push, pop, translate, scale, rotate).
- `ProjectionSuite`: `[output, zNear, zFar, fov, width, height, invertY, matrixVersion]`. `getMatrix` needs `RenderSystem`'s device, so the suite installs a stand-in (a `GpuDevice` proxy that only answers `getDeviceInfo`) whose `isZZeroToOne` is the case's `args[0]`.

Value types (JOML's, and the game's below) get an `InstanceSuite`, which covers the constructors and instance methods it lists the same way. A case calls one method on a fresh receiver, which is `args[0]`, and `after[0]` is the receiver's state afterwards. Any other argument the method wrote to gets its state in `after` too, unless it's the returned object, which `result` already shows. A constructor is an `<init>` case whose result is the new object, and a static method listed with them (a record's factories) gets cases without a receiver. `equals(Object)` takes another object of the same class. The port checks them with `golden.check_method`, which fails a case when vanilla recorded an `after` slot the check can't see, or `golden.check_mut` when the method also writes to another argument. A suite can add cases for a member (`extraInputs`, each a receiver followed by the arguments), and write the class's public constants like a static suite (`withFields`).

JOML objects are encoded as their fields in declaration order: a `Vector2f` is `[x, y]`, a `Vector3f` `[x, y, z]`, a `Vector4f` and a `Quaternionf` `[x, y, z, w]`, a `Matrix3x2f` (or `Matrix3x2fStack`) `[m00, m01, m10, m11, m20, m21]`, a `Matrix3f` `[m00, m01, …, m22]`, a `Matrix4f` (or `Matrix4fStack`) `[m00, m01, …, m33, properties]`, and a `FrustumIntersection` its 24 private plane components `[nxX, nxY, nxZ, nxW, pxX, …, pzW]` followed by its `planes`, an array of six `Vector4f` (read by reflection). An `Axis` is the name of its constant (`"XP"`). A `GivensParameters` is `[sinHalf, cosHalf]`, like any record. A `Transformation` is `[matrix, decomposed, translation, leftRotation, scale, rightRotation, identity]`: its fields (read by reflection), the four parts null until it's decomposed, then whether it's the `IDENTITY` constant, which the game compares with by reference. An `Optional` (only `Transformation.compose` takes one) is its value or null, and so is an argument a method lets be null (a `Transformation`'s missing part). Transformations are made the ways the game makes them (block entities' parts, block model rotations, entities' parts, and matrices of those parts), from any JOML matrix, or decomposed already. The receivers and arguments come from JOML itself (identities, translations, rotations, scales, the camera's projections, products of those, and matrices built from components, NaN and infinities included), so their properties are ones the game's matrices can have. 2D poses are the GUI's (pixel translations, GUI scales, a map decoration's rotation), and frustums are mostly the camera's culling frustums (`Camera.createProjectionMatrixForCulling` times the view rotation) looking anywhere.

The game's own value types get `InstanceSuite.every`, which covers every public constructor, method and constant the class declares, like a static suite does: a member whose arguments have no generator, or whose result has no encoder, is written as skipped, so the port has to mark it pending (the codecs, and `Direction.orderedByNearest`, which takes an `Entity`). An inherited method whose answer depends on the receiver's class is added with `inherited` (`Vec3i.toString` prints `BlockPos{…}` for a `BlockPos`), and `where` drops inputs like a static suite's. A `Vec3i` is `[x, y, z]` (ints), and so are a `BlockPos` and a `MutableBlockPos`; a `Vec3` is `[x, y, z]` (doubles), a `Vec2` `[x, y]` (floats), and a JOML `Vector3i` `[x, y, z]` (ints). An enum constant is its name (`"NORTH"`), an `EnumSet` (`Vec3.align`'s axes) the names of its constants in order, and a collection, stream, iterator or iterable the array of its elements, each encoded by its own class as it comes (the first 512). `BlockPos`'s iterables hand out one `MutableBlockPos` they move, so each element is encoded before the next is asked for. When working out an element throws (`randomBetweenClosed` drawing from an empty box), the call counts as throwing, so the port collects the elements inside the checked call. A `Position` argument is a `Vec3`, and an argument a method lets be null (`Direction.getNearest`'s fallback) gets null among its extra inputs. When every argument is an enum, the edges are every combination already, so there are no random cases.

Their values are like the game's: blocks around the walk scenario's spawn at any height, steps and offsets, blocks anywhere inside the world border, and any ints; the walk scenario's spawn and its deltas, entities' deltas, look vectors, block corners and centers, and any doubles; rotations. Edges include NaNs of both signs: x86's own NaN (`0.0 / 0.0`) is negative, unlike Java's canonical one, and `hashCode` has to make them the same. The extra inputs go where results flip: points exactly as far apart as the distance a strict `<` checks them against (and an ulp either side), vectors whose length is `normalize`'s threshold (along an axis, where the length comes out exact), ties for `getNearest` and `axisStepOrder`, subnormal dot products for `getApproximateNearest`, `fromYRot`'s multiples of 45 degrees, every degree of `isFacingAngle`, and `BlockModelRotation`'s 48 matrices for `Direction.rotate`. `Float.floatToIntBits` and `Double.doubleToLongBits` get NaNs with other payloads.

`BlockPos`'s blocks also include the largest and smallest coordinates `asLong` packs (26 bits for x and z, 12 for y) and one past them, and its extra inputs go where its answers change: every block at those limits for `asLong`, blocks packed into longs for `of`, `getX(J)` and the rest, points on `clampLocationWithin`'s bounds and an ulp either side (its lower bound is a float sum, which rounds far out), Manhattan reaches up to 4 and negative ones, boxes up to 9 blocks across with their corners either way round, columns, every pair of directions for `spiralAround`, boxes walked along movements with zero, -0.0 and NaN components, and draws from small boxes, some empty. A Manhattan search with a negative reach and a large depth finds no block and runs until the depth wraps, so those inputs need every reach non-negative or a depth up to 64.

`FrustumIntersection.intersectAab` and `testPoint` also get boxes and points like the game's (chunk sections, entities and the camera's cube, relative to the camera), boxes and points on a plane to rounding (on the frustum's face, where the order the plane's terms are added in decides which side they land on), and every float edge in each coordinate of a box in front of every edge frustum.

```json
{"method":"translate(FFF)Lorg/joml/Matrix4f;","args":[["3f800000",…,"3f800000",30],"3f000000","c2700000","3f000000"],"after":[[…,26],null,null,null],"result":[…,26]}
```

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
