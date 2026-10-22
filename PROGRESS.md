# Progress

The loop's memory: newest entries first. Each step adds an entry and rewrites "Next".

## Next

Milestone 2, part 3: `AABB`, then `Identifier`, each with `InstanceSuite.every` and generators like the game's values.

- `AABB` (56 public members) needs `BlockPos`, `Vec3`, `Direction` and `Vector3f`. `clip(Iterable, Vec3, Vec3, BlockPos)` returns a `BlockHitResult`, so it needs `HitResult` and `BlockHitResult` (small) or stays pending; `of(BoundingBox)` waits for `BoundingBox`. `AABB.Builder` is its own suite. Generators: entity boxes (0.6 by 1.8 around the walk scenario's positions), block shapes, boxes that touch or overlap by an ulp, and rays for `clip`. With an `AABB` generator, `BlockPos.betweenClosed(AABB)`, `betweenClosedStream(AABB)` and `betweenCornersInDirection(AABB, Vec3)` get cases, which un-pends them.
- `Identifier`: parsing and validation, with `IdentifierException`. `read(StringReader)` can wait for brigadier.

Then Minecraft's `Frustum` on top of `FrustumIntersection` (it needs `AABB` and `BoundingBox`), `PoseStack.translate(Vec3)` (appended to `PoseStackSuite`'s programs), and `BlockMath` (`Quaternionf.rotateX/Y`, `Direction.getApproximateNearest`), which `BlockModelRotation` builds its face transformations with, along with `OctahedralGroup` (and its `SymmetricGroup3` and `FrontAndTop`), which also un-pends `Rotation.rotation()`. When block models and render states come, settle how they hold shared `Transformation`s: Java's `mulPose(Transformation)` and `BlockMath.getFaceTransformation` add properties to a shared transformation's own matrix (static ones like `TheEndPortalRenderer.TRANSFORMATION` included), so every holder sees them, and that changes which shortcuts later JOML calls take.

`ProjectionType.applyLayeringTransform` and `VertexSorting` come with the renderer. `log10` (one call site, `Options`) and `cbrt` (one, `ClientExplosionTracker`) are small HotSpot stubs; port them when those classes come. The client never calls `Math.exp`. ARGB's `Vector3f`, `Vector4f` and `Vec3` methods have golden cases (its `pending_rest` covers them) and can be ported whenever ARGB is next touched. `Vec3.toString` needs Java's `Double.toString` (shortest digits, but a one-digit answer may become a closer two-digit one, and its own exponent format), which only logs and error messages print so far.

## Needs Thalles

- **Licence of the code translated from OpenJDK and JOML.** `src/java/lang/math/hotspot_x86/` is translated from OpenJDK's HotSpot, which is GPLv2 only (no Classpath Exception), and keeps its notices. `src/java/lang/fd_libm.rs` comes from the JDK's `FdLibm.java`, GPLv2 with the Classpath Exception. `src/org/joml/` comes from JOML, MIT, whose notice a distribution has to carry. That's fine while the repo stays private. Shipping a build of rustcraft would put it under GPLv2 too. Not blocking. Decide before the repo or any build goes public.

## Log

### 2026-10-22: BlockPos, MutableBlockPos, Rotation and AxisCycle

- Ported and checked against vanilla, milestone 2's second slice. Each class is ported whole, apart from what waits for other types:
  - `BlockPos`: 53 members and its 4 constants (56,627 cases). It holds its `Vec3i` and derefs to it, and the methods it overrides return a `BlockPos`. Its iterables are iterators over the fields of Java's anonymous `AbstractIterator`s: `betweenClosed` and its streams, `withinManhattan`, `withinClippedManhattan`, `withinBoxByManhattanDistance`, `neighborColumn`, `spiralAround`, `betweenCornersInDirection`, `randomBetweenClosed` and `randomInCube`. Pending: the codecs, the `AABB` and `BoundingBox` overloads, and `breadthFirstTraversal`, which only the server's blocks call (`SpongeBlock`, `CreakingHeartBlockEntity`) and whose callbacks have no generator.
  - `BlockPos.MutableBlockPos`: all 26 members (21,373 cases), through `Vec3i`'s protected setters, now `pub(super)`. `move` is `r#move`.
  - `BlockPos.TraversalNodeStatus` (539 cases), `AxisCycle` (7 members and 5 constants, 1,794 cases) and `Rotation` (8 members, 2,273 cases). Pending: `Rotation`'s codecs and `BY_ID`, and `rotation()`, which waits for `OctahedralGroup`.
  - `Mth.log2` and the functions it calls are `const fn`s now, so `BlockPos`'s packing constants are worked out as in Java.
  - `toString` prints the receiver's class name (`BlockPos{x=1, y=2, z=3}`), so `Vec3i`'s `Display` goes through a helper that takes the name.
- Golden harness:
  - An `Iterable` result is encoded like an iterator, each element as it comes, since `BlockPos`'s hand out one `MutableBlockPos` they move. When working out an element throws, the call counts as throwing (`Values.ElementThrew`), so the port collects the elements inside the checked call.
  - `InstanceSuite.where`, like `StaticSuite`'s, and `InstanceSuite.inherited`, for an inherited method whose answer depends on the receiver's class (`Vec3i.toString` for a `BlockPos` and a `MutableBlockPos`).
  - New generators: `BlockPos` and `MutableBlockPos` (`Vec3i`'s blocks plus the largest and smallest coordinates `asLong` packs, and one past them), and the constants of `AxisCycle`, `Rotation` and `TraversalNodeStatus`.
  - Extra inputs where results change: every block at the packing limits for `asLong`, packed blocks for `of`, `getX(J)` and the rest, points on `clampLocationWithin`'s bounds and an ulp either side, Manhattan reaches from -2 to 4, small boxes with their corners either way round, columns, every pair of directions for `spiralAround`, walks along movements with zero, -0.0 and NaN components, draws from small and empty boxes, and every rotation of 4 and 16 steps.
- Found:
  - `clampLocationWithin`'s lower bound is a float sum (`getX() + 1.0E-5F`, widened afterwards), and the upper one a double minus `1.0E-5F` widened; the bytecode shows both. From 256 blocks out the 1.0E-5 rounds away, so the lower bound is the block's own face. Past 2^24 the coordinate itself rounds, and the lower bound can land a block below, or above the upper bound.
  - `manhattanOrdered` looks only at the origin at depth 0, whatever the reaches, since its fields start at 0, and its `for` loop moves `y` on after a find too. With a negative reach, later depths start outside it. With a negative reach and a large depth it never finds a block and runs until the depth wraps, so the suite passes such inputs only with a depth up to 64.
  - `betweenClosed` never divides by zero: a side of 0 makes the end 0, so it visits nothing.
  - `spiralAround` starts at the center: the cursor starts a step along the second direction and steps back first.
- Mutations (35, run once and not committed): 34 fail the goldens. Two of them make a Manhattan search's depth the sum of the reaches instead of the largest, or the reverse: they hang on inputs where vanilla's ends, which counts as failing. The other mutation is equivalent: `neighborColumn`'s `>` as `>=` only changes the direction when the column is one block high, where `index % 1` is always 0.
- With NaNs normalized, every line of the other golden files is identical to before.
- Evidence: `cargo test --workspace` passes 73 tests, including 39 golden suites (68 and 34 before). fmt, clippy and the port check are clean.

### 2026-10-22: Vec3i, Direction, Vec3 and Vec2

- Ported and checked against vanilla, milestone 2's first slice. The bytecode scan (PORTING.md) shows the client calling nearly every method of these classes, so each is ported whole, apart from what waits for other types:
  - `Vec3i`: 41 members, all but the codecs (28,808 cases). The protected setters come with `MutableBlockPos`.
  - `Direction` (42 members, 18,304 cases) with its nested `Axis` (17, 3,383), `AxisDirection` (6, 543) and `Plane` (9, 2,091). Pending: the codecs and `BY_ID`, and `orderedByNearest` and `getFacingAxis`, which take an `Entity`.
  - `Vec3`: 56 members (42,128 cases). Pending: the codecs, and `toString`, which needs Java's `Double.toString`.
  - `Vec2`: 13 members (10,809 cases).
  - `Mth.getSeed(Vec3i)` and `Mth.lerp(double, Vec3, Vec3)`, un-pended (1,087 cases).
  - JOML's `Vector3i`: the 4 methods the client calls (2,474 cases).
  - `java.lang.Float.floatToIntBits` and `hashCode`, `Double.doubleToLongBits` and `hashCode` (2,184 cases, other NaN payloads included), which `Matrix4f`'s `equals` and `hashCode` now use too.
  - `Util.getRandom`, `shuffledCopy` and `shuffle`, checked through `Direction`'s and `Plane`'s random cases.
- Golden harness:
  - `InstanceSuite.every(Class)` covers every public constructor, method and constant a class declares, and writes the ones without a generator or encoder as skipped, so the port's test has to mark them pending.
  - New `CoreValues`: enum constants by name, `EnumSet<Direction.Axis>`, `Vec3i`, `Vec3` (also for `Position`) and `Vec2`, generated like the game's values (TESTING.md). Collections, streams and iterators are encoded as arrays of their elements. When every argument is an enum, there are no random cases: the edges are every combination already.
  - Extra inputs where results flip: points exactly as far apart as a strict `<` distance check's distance (and an ulp either side), lengths at `normalize`'s threshold, ties for `getNearest` and `axisStepOrder`, subnormal and NaN dot products for `getApproximateNearest`, `fromYRot`'s multiples of 45 degrees, every degree of `isFacingAngle`, and `BlockModelRotation`'s 48 matrices for `Direction.rotate`.
  - `golden.rs`: `encode_names!` encodes an enum by its constants' names, a `Vec` decodes from an array, and a `LegacyRandomSource` argument decodes from its seed and encodes as its next `nextLong()`, so `check_mut` checks instance methods that draw from one (`Vec3.offsetRandom`, `Plane.getRandomDirection`).
- Found:
  - `Direction.getApproximateNearest` starts from `Float.MIN_VALUE`, the smallest positive float: a zero or NaN vector gives `NORTH`, and a subnormal dot product beats it.
  - `Vec3.normalize` compares with `1.0E-5F` widened to a double (`9.999999747378752E-6`), which the bytecode shows.
  - x86's own NaN (`0.0 / 0.0`) is negative, unlike Java's canonical one, so `Vec3` and `Vec2` edges include one: their `hashCode` must canonicalize it.
- Mutations (42, run once and not committed): 41 fail the goldens. The other one is equivalent: `>>` for `>>>` in `Double.hashCode`, whose cast to int keeps only the low half. `<=` for `<` in `normalize` survived the first inputs, since no length came out exactly at the threshold. Along an axis it does, and that mutation fails 6 cases now.
- With NaNs normalized, every line of the other golden files is identical to before, except where the new types made skipped entries checkable: `Mth`'s `getSeed(Vec3i)` and `lerp(double, Vec3, Vec3)`, `ARGB.color(Vec3)` (under ARGB's `pending_rest`), and `Mth.rayIntersectsAABB`'s skip reason, which names `AABB` now.
- Evidence: `cargo test --workspace` passes 68 tests, including 34 golden suites (58 and 24 before). fmt, clippy and the port check are clean.

### 2026-10-22: Transformation, GivensParameters, MatrixUtil's decompositions and fdlibm's hypot

- Ported and checked against vanilla, which finishes the client's math that doesn't need milestone 2's types:
  - `GivensParameters`, a record: its constructor, both factories, accessors, `inverse`, `cos`, `sin` and the six `around*` (11,277 cases). Records' `equals`, `hashCode` and `toString` aren't ported: nothing calls them.
  - `MatrixUtil.eigenvalueJacobi` (2,962 cases) and `svdDecompose` (3,585), with their private helpers and the `G` and `PI_4` constants. Java names four matrices in the SVD but they're two objects that the steps overwrite in turn; the port follows the objects. Its zero-column checks compare as doubles (the bytecode widens them).
  - `Transformation`: both constructors, `compose`, `inverse`, `getMatrix`, `getMatrixCopy`, the four decomposition getters, `equals`, `hashCode`, `slerp`, the static `compose(Matrix4fc, Optional)` and `IDENTITY` (11,088 cases). The codecs wait for DataFixerUpper. The decomposition is a `OnceLock`, so its getters take `&self`. `IDENTITY` carries a flag its clones keep, since the game compares with it by reference (`FaceBakery`, and `inverse` returns it as it is).
  - `PoseStack.mulPose(Transformation)`. Java hands `mulPose` the transformation's own matrix, and `MatrixUtil.checkProperty` adds properties to it, so the port takes `&mut Transformation` and goes through `get_matrix_mut()` (PORTING.md). 256 more programs (768) multiply by transformations: 1,536 such steps, 16 of which add properties to the transformation's matrix.
  - fdlibm's `Hypot` in `fd_libm.rs`, as `Math.hypot` (8,253 cases: every edge pair, 6,000 spread over its paths, and its thresholds). std's `f64::hypot` (glibc's) gets 249 of them wrong.
  - JOML: `Matrix3f.scale(F)`; `Matrix4f.invertAffine` (both overloads), `isFinite`, `translation(Vector3fc)`, `scale(Vector3fc)`, `getTranslation`, `equals` (as `PartialEq`, comparing `floatToIntBits` and not the properties) and `hashCode`; `Quaternionf.slerp` into a `dest`; `Vector3f.lerp` into a `dest` (11,734 cases between them). The client calls each of them, or `Transformation` does (`equals` and `hashCode`, through `Objects`).
- Golden harness:
  - `StaticSuite` covers `void` methods: the result is null, and `after` shows what they wrote. On the Rust side, `golden.check_mut` takes a function with every argument as `&mut` and compares all of them afterwards.
  - `InstanceSuite` takes static members too (a record's factories), and `equals(Object)` gets another object of the same class.
  - New `MathValues`: `GivensParameters`, `Transformation` (made the ways the game makes them: block entities', block model rotations, entities' parts, their matrices, any JOML matrix, and decomposed ones) and `Optional<Transformation>`. JOML's generator (`Joml`, now `Mutable`) passes a null spec as null, and `golden.rs` decodes null as `None`.
- Mutations (41, run once and not committed): 34 fail the goldens. The other 7 are equivalent, or hidden where the game rounds:
  - `hypot`'s scaling thresholds moved inside their safety margins (3), and its early return moved from a ratio of 2^60 to 2^61, where the full computation rounds to the same answer;
  - swapping the two terms of a commutative addition in `slerp`;
  - comparing the SVD's zero columns as floats, which differs on one float value only;
  - `hypot`'s `w > b` as `w >= b`, which agrees on all 500 inputs with a = 2b exactly, where the two formulas add the same exactly split terms;
  - glibc's `hypot` in `qrGivensQuat`, whose result is rounded to a float.

  2 survived the first version of the inputs: `G` one ulp off and `w >= b`. That led to the matrices on `approxGivensQuat`'s quarter-turn boundary, after which `G` one ulp lower fails 40 cases (one ulp higher gives the same float `G`), and to the a = 2b pairs.
- With NaNs normalized, every line of the other golden files is identical to before. The changed files only gained the new members, and `PoseStack` its 256 programs after the old 512.
- Evidence: `cargo test --workspace` passes 58 tests, including 24 golden suites (56 and 22 before). fmt, clippy and the port check are clean.

### 2026-10-22: Matrix3x2f, Matrix3x2fStack, Vector2f and FrustumIntersection

- Ported from JOML 1.10.9 the methods the client calls, found by scanning its bytecode for `org/joml/` calls (PORTING.md). Each is checked against vanilla:
  - `Vector2f`: 2 constructors and 6 methods (5,524 cases).
  - `Matrix3x2f`, the GUI's 2D pose: 2 constructors and 12 methods (8,831 cases). These are `set`, `translate`, both `scale`s, `rotate`, `transformPosition` and `invert` with a `dest`; each fluent one also has its `dest` overload.
  - `Matrix3x2fStack` (push and pop). New `Matrix3x2fStackSuite`: 256 programs of 48 ops, using the ops the client calls on `GuiGraphicsExtractor`'s stack (push, pop, translate, scale, rotate), 10,055 steps.
  - `Matrix4f.mul(Matrix3x2fc)` and its `dest` overload (1,350 cases). `GlyphRenderState` uses it to put a glyph's 2D pose into 3D.
  - `FrustumIntersection`: `<init>()`, both `set`s, `intersectAab`, `testPoint` and its 15 constants (20,239 cases). Minecraft's `Frustum` needs `AABB`, so it waits for milestone 2.
- Golden harness:
  - `InstanceSuite` takes extra inputs per member, and can write the class's constants (`withFields`).
  - `golden.rs` encodes fixed-size arrays.
  - New generators: GUI poses, and frustums. Most frustums are the camera's culling frustums (`Camera.createProjectionMatrixForCulling` times the view rotation) looking anywhere, with the walk scenario's two looks among the edges. The rest are frustums of any JOML matrix, plus one perspective that ignores x.
  - `intersectAab` and `testPoint` get extra inputs:
    - boxes like the game's: chunk sections, entities, the camera's 8-block cube, and points;
    - boxes and points on a frustum face, to rounding: solved from three planes, then nudged by up to two ulps;
    - every float edge in each coordinate of a box in front of each edge frustum.
  - Every `intersectAab` result occurs: `INSIDE` 1,128 times, `INTERSECT` 2,569, and each plane from 262 to 4,109.
- Found:
  - Only cases on a plane pin down the order a plane's terms are added in. Before they existed, reassociating `testPoint`'s or `intersectAab`'s sums still passed the goldens.
  - A camera frustum's side planes pass through the camera (w = 0), so adding w first is exact there. Only the near and far planes, and frustums that aren't the camera's, can tell the order apart.
  - Moving `w` across `intersectAab`'s `>=` can't change a comparison near equality: `s + w` is exact when `s` is close to `-w` (Sterbenz). Only infinities can tell, and the frustum of an infinite translation covers those.
- Mutations (37, run once and not committed): 35 fail the goldens. The other 2 are equivalent:
  - a stack size of 0 panics anyway, on its allocation of -1, as Java throws `NegativeArraySizeException`;
  - the far plane's `w` moved across the `>=` (Sterbenz, above).

  4 mutations survived the first version of the frustum inputs, which led to the on-plane cases, the perspective that ignores x (a zero normal component times an infinite coordinate) and the float-edge sweep.
- With NaNs normalized, every line of the other golden files is identical to before. `Matrix4f`'s only gained the 1,350 new cases.
- Evidence: `cargo test --workspace` passes 56 tests, including 22 golden suites (52 and 18 before). fmt, clippy and the port check are clean.

### 2026-10-22: Axis, MatrixUtil, PoseStack, Projection, Matrix4fStack and Vector4f

- Ported Mojang's math on top of JOML, each checked against vanilla:
  - `com.mojang.math.Axis`: all 6 methods (3,716 cases). It's an interface, but only its six constants implement it, so it's an enum. `Axis.of` makes another implementation, but nothing calls it, so it isn't ported.
  - `MatrixUtil`: `mulComponentWise`, `checkPropertyRaw`, `checkProperty`, `isIdentity` and `isPureTranslation` (4,747 cases). `eigenvalueJacobi` and `svdDecompose` are pending: only `Transformation` calls them, and they return void, so the static suite skips them.
  - `PoseStack` and its `Pose`, except `translate(Vec3)` and `mulPose(Transformation)`. New `PoseStackSuite`: 512 programs of 48 ops (push, pop, both translates, uniform, mirrored and other scales, the rotations, `mulPose`, `transformNormal`, `setIdentity`, `isEmpty`), recording `[output, pose, normal, trustedNormals]` after each op (23,396 steps). One program in four takes any float, NaN included.
  - `Projection` and `ProjectionType`'s constants. `getMatrix` asks `RenderSystem`'s device whether depth runs from 0 to 1, so the port takes that as a parameter (PORTING.md). New `ProjectionSuite`, which installs a stand-in device (a `GpuDevice` proxy that only answers `getDeviceInfo`): 256 programs, half with each depth range, recording `getMatrix`'s matrix and every getter after each op (6,144 steps).
- JOML:
  - `Vector4f`: 5 constructors and 15 methods (13,264 cases).
  - `Matrix4f`: `assume`, and `transform`/`transformTranspose` of a `Vector4f` with their `dest` overloads (3,925 cases), which the frustum needs.
  - `Matrix4fStack` (`pushMatrix`, `popMatrix`, `clear`), which holds its `Matrix4f` and derefs to it. New `Matrix4fStackSuite`: 256 programs over stack sizes 1 to 20 and three the constructor rejects, recording the matrix after each op (10,426 steps).
  - With `Vector4f` values, ARGB's 12 `Vector4f` methods have cases now (8,514), under its `pending_rest`.
- Golden harness:
  - A slot vanilla leaves empty in `after` means the argument didn't change, or is the returned object. The port's argument must then equal what it was or the result. Before, `check_method` couldn't check a static method writing to its argument (`checkProperty`), since vanilla only records a changed argument.
  - `Encode::matches` never panics now: a value of another shape doesn't match. The rule above needs that, since it compares an argument with the result, like a matrix with `checkProperty`'s boolean.
  - Rust: `Encode` for `Box<dyn Encode>` and tuples, and `Thrown` for an op that panicked, so a program's steps keep their types (any NaN still matches any NaN). Java: an `Op` record shared by the program suites.
- Found:
  - NaN bits in the golden files differ between two runs of the generator. Two runs in a row gave 39 differing lines in `Matrix3f` and a few in four other files, all of them NaN signs. With NaNs normalized, every line of the old golden files is identical to before.
  - `PoseStack.Pose.mulPose` changes the matrix it's given: `MatrixUtil.checkProperty` adds the properties it works out to the matrix's. So `mul_pose` takes `&mut Matrix4f`.
- Mutations (31, run once and not committed): 29 fail the goldens, and the other 2 are equivalent (an index past the stack's end panics anyway, and JOML's `fma` is `a * b + c`). One found a gap: no program changed only the FOV between two setups, so dropping the FOV comparison from `setupPerspective` passed. Setups now also repeat the previous one with one argument changed (578 times), and that mutation fails 40 of 256 programs.
- Evidence: `cargo test --workspace` passes 52 tests, including 18 golden suites (12 before). fmt, clippy and the port check are clean.

### 2026-10-22: JOML's Math, Vector3f, Quaternionf, Matrix3f and Matrix4f

- The golden harness covers instance methods now. `InstanceSuite` (oracle/golden/) calls the constructors and instance methods it lists by reflection. The receiver is `args[0]`, `after[0]` is its state afterwards, and any other argument the method wrote to gets a slot unless it's the returned object. On the Rust side, `golden.check_method` compares all of that and fails a case when vanilla recorded a slot the check can't see. `encode_fields!` turns a struct into its JSON. TESTING.md has the format.
  - JOML's receivers and arguments come from JOML itself (`JomlValues`): identities (one with -0.0 in it, which still counts as the identity), translations, rotations, scales, the camera's reversed-depth perspective, the GUI's ortho, products of those, matrices built from components, NaN and infinities, and random transforms composed of one to four steps.
- Ported from JOML 1.10.9's sources into `src/org/joml/` (PORTING.md has a new section on libraries). `tools/setup_vanilla.py` fetches the sources jar from Maven Central, checks that Maven's binary jar is the one Mojang ships (same SHA-1), and unpacks it next to the decompiled game.
  - `Math`: 74 of its 76 methods, and its constants. `exp` is pending (HotSpot's stub isn't ported, and the client never calls it); `random` is clock-seeded.
  - `Matrix4fc`'s 20 constants.
  - The methods `Camera`, `PoseStack`, `Axis`, `Projection` and `GameRenderer` call, with what they call: 37 on `Vector3f`, 25 on `Quaternionf`, 31 on `Matrix3f`, 59 on `Matrix4f` (104,118 cases between them).
- Found:
  - JOML's fast trig (`-Djoml.fastmath`) and fused `fma` (`-Djoml.useMathFma`) are off unless the JVM is told otherwise, and the game doesn't. So JOML's trig goes through HotSpot's stubs, and its `fma` is `a * b + c`, rounded twice.
  - `Matrix4f.rotateAround(q, x, y, z, dest)` on an identity matrix writes and returns `this`, and leaves `dest` alone. Ported as is.
  - `Matrix4f`'s property shortcuts change bits: `identity()` keeps an identity's -0.0. Dropping that shortcut fails `identity` and `invert`.
  - `Math.roundLongUsing`'s half modes round to an int, so they saturate at the int range.
- `java.lang.Math` gained `acos`, `atan2`, `rint`, `round(F)`, `round(D)`, `signum(F)`, `signum(D)`, `toRadians` and `toDegrees`, all checked. With the libm crate, `atan2` got 76 of its 2,181 cases wrong: the crate ports musl, which returns ±pi/2 for |y/x| > 2^64 where fdlibm adds half of pi's low part. Ported the JDK's `FdLibm` `Asin`, `Acos`, `Atan` and `Atan2` into `src/java/lang/fd_libm.rs` instead, and dropped the libm dependency.
- `Mth`: un-pended and checked `rotationAroundAxis` and `X_AXIS`/`Y_AXIS`/`Z_AXIS`.
- In the other golden files, only the JOML entries vanilla used to skip changed (in `Mth` and `ARGB`). Every other line is byte-identical to before.
- Extra evidence, run once and not committed:
  - 12.3 million random `asin`/`acos`/`atan2` inputs from vanilla's JDK (10M with C2, 2M C1 only, 300k `-Xint`), compared with a release build of the port: 0 mismatches. The libm crate's `atan2` gets 12.6% of them wrong.
  - Mutations that each fail the goldens: `identity()` without its shortcut, `translate` keeping `PROPERTY_PERSPECTIVE`, `rotateAround` writing `dest` on an identity, `mulProject` without its identity shortcut, and a `normalize()` that leaves the receiver alone.
- Evidence: `cargo test --workspace` passes 46 tests, including 12 golden suites (6 before). fmt, clippy and the port check are clean.

### 2026-10-22: HotSpot's tan and pow

- Ported HotSpot's x86-64 stubs for `Math.tan` and `Math.pow` (from the same OpenJDK 25 source as sin, cos and log) into `src/java/lang/math/hotspot_x86/`. `Math::tan` and `Math::pow` run them.
  - tan's jumps nest, so it's `if`s like sin. pow jumps back and forth between 56 labels, so it's a loop with one `match` arm per label, in the C++'s order, and a `jmp!` macro (PORTING.md).
  - The register model gained `andpd`, `cvttpd2dq`, `cvtdq2pd`, `mulss`, `paddq`, `sqrtsd`, `movapd`, `movw`, `notl` and `bsrl`.
- Added and checked in the `java.lang.Math` suite: `tan(D)D` (20,541 cases) and `pow(DD)D` (22,181: every pair of the 41 double edge values, 20,000 extra inputs, 500 random). Nothing was pending.
  - tan's extra inputs come from sin's generator, with tan's thresholds (2^-29, 0.1 and 12800).
  - pow's cover game-like inputs, small integer exponents of any base, bases near 1 with exponents up to 2^64, results across the whole range (subnormal, overflowing, underflowing), negative bases with integer and non-integer exponents, tiny exponents, the special exponents (±0, ±1, 0.5, 2, 3, ±inf, NaN) and random bits.
  - The input helpers were factored out without changing a draw: every other line of the golden files is byte-identical to before.
  - Against these inputs, glibc gets 96 tan cases wrong, and 21 pow cases: 13 finite results one ulp off and 8 special cases (`pow(±1, ±inf)` and `pow(1, NaN)` are NaN in Java). fdlibm (the libm crate) gets 462 and 714 wrong. The port matches every case.
- Extra evidence, run once and not committed:
  - 10 million random inputs per function, streamed from vanilla's JDK into a release build of the port: 0 mismatches. The same with `-Xint` (300k each) and C1 only (2M each): 0 mismatches.
  - The same with garbage in every register lane the stubs don't set (the goldens, plus 5M streamed each): 0 mismatches.
  - All 29 tables equal the C++ arrays.
- `tools/hotspot_port_check.py`: all of tan's 448 and pow's 753 instructions match, and so do all 22 and 91 straight-line runs, in order. It now also reads `cmp64`/`jccb`, 16-bit stack loads (`movw`), indexed addresses (`Address(r11, rdx, Address::times_8, 0)`) and `ExternalAddress(e_coeff + 0)`. A swapped pair in pow, a changed displacement in pow's `L_tbl` address and a changed constant in tan each make it fail.
- pow picks a table entry with `rcpps` like log does, so its last bit can differ between CPUs too (PORTING.md).
- Evidence: `cargo test --workspace` passes 40 tests, including 6 golden suites. fmt, clippy and the port check are clean.

### 2026-10-22: HotSpot's sin, cos and log

- Ported HotSpot's x86-64 stubs for `Math.sin`, `cos` and `log` from OpenJDK 25 (adoptium/jdk25u at 520406d8, the source of the Temurin 25.0.4.1 the oracle runs) into `src/java/lang/math/hotspot_x86/`. Each `stubGenerator_x86_64_*.cpp` becomes one `.rs` file, with one Rust call per assembler instruction (`__ mulsd(xmm1, xmm0);` → `xmm1.mulsd(xmm0);`) on a small register model in `assembler.rs`. `Math::sin/cos/log` run them now, so `Mth`'s tables and `nextGaussian` do too.
- Un-pended and checked: `cos(D)D` and `log(D)D` in `java.lang.Math`, `atan2(DD)D` in `Mth`, and the four `<source>+nextGaussian` RandomSource programs. No pending entry that depends on `Math` is left.
- More golden inputs:
  - `Math.sin/cos/log`: 20,000 extra each (541 → 20,541 cases each), spread over every path through the stubs.
  - `Mth.atan2`: a 4096-angle sweep that reads every table entry (2,181 → 6,277 cases).
  - Gaussian programs: 16 → 64 per source.
  - Against these inputs, glibc's versions get 17 sin, 30 cos, 2 log and 6 atan2 cases and one gaussian program wrong. The 541 old inputs had let glibc's sin pass. The port matches every case.
- Extra evidence, run once and not committed:
  - 10 million random inputs per function, streamed from vanilla's JDK into a release build of the port: 0 mismatches.
  - The same with `-Xint` (300k) and C1 only (2M): 0 mismatches. So every JVM tier uses these stubs.
  - The same with garbage in every register lane the stubs don't set (5M): 0 mismatches. What a caller leaves in the registers never reaches the result.
  - All 22 tables match the C++ values.
- New: `tools/hotspot_port_check.py` reads a port back into assembler calls and compares it with the C++ of the installed JDK. All of sin's 359, cos's 357 and log's 105 instructions match, and so do all 53 straight-line runs, in order. A swapped pair and a changed constant both make it fail. CI runs it.
- Found: HotSpot's `log` picks its table entry with `rcpps`, an approximate reciprocal that Intel and AMD implement differently. With a correctly rounded reciprocal instead, 17 in 10 million log results on this Ryzen 5 5600G come out one ulp off. So the port runs the real instruction, and vanilla's outputs only compare with the port's from the same kind of CPU (PORTING.md). Golden files are regenerated on whichever machine runs the tests, CI included, so they stay consistent.
- Evidence: `cargo test --workspace` passes 40 tests, including 6 golden suites. fmt and clippy are clean.

### 2026-10-22: setup

- Vanilla 26.3 is decompiled into `vanilla/` (gitignored). The oracle covers four layers: function goldens, tick traces, packet captures and screenshots. See TESTING.md.
- Two vanilla runs of `walk` through a vanilla server match exactly (`check-determinism walk --capture`).
- Ported, and matching vanilla bit for bit: `Mth`, `RandomSource` and its four implementations, `RandomSupport`, `Xoroshiro128PlusPlus`, `MarsagliaPolarGaussian`, plus the JDK pieces they use (`Math`, `String.hashCode`, `UUID`).
- Pending, because HotSpot's x86-64 `cos`/`log` aren't ported: `Math.cos`, `Math.log`, `Mth.atan2`, and gaussian draws.
- Decided: a multiplayer-only client first. Entities will live in an arena addressed by ID when they come.
