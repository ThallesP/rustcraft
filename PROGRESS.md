# Progress

The loop's memory: newest entries first. Each step adds an entry and rewrites "Next".

## Next

Milestone 1, JOML, part 2: Mojang's math on top of what's ported. Port `com.mojang.math.Axis`, `MatrixUtil` (`isPureTranslation`, `checkPropertyRaw`…), `com.mojang.blaze3d.vertex.PoseStack` with its `Pose`, and `Projection.getMatrix` (which swaps near and far), plus the JOML they still need: `Matrix4fStack` (`pushMatrix`/`popMatrix`) and `Vector4f`. `PoseStack` is stateful, so give it a hand-written suite of push/pop/translate/scale/rotate/mulPose programs like `RandomSourceSuite`, recording `last().pose()`, `normal()` and `trustedNormals` after each op. Add each JOML method to its `InstanceSuite` list as it's ported. After that: `Matrix3x2f`/`Matrix3x2fStack` for the GUI, and `FrustumIntersection` with Minecraft's `Frustum` for chunk culling.

`log10` (one call site, `Options`) and `cbrt` (one, `ClientExplosionTracker`) are small HotSpot stubs; port them when those classes come. The client never calls `Math.exp`. ARGB's `Vector3f` methods now have golden cases (its `pending_rest` covers them) and can be ported whenever ARGB is next touched.

## Needs Thalles

- **Licence of the code translated from OpenJDK and JOML.** `src/java/lang/math/hotspot_x86/` is translated from OpenJDK's HotSpot, which is GPLv2 only (no Classpath Exception), and keeps its notices. `src/java/lang/fd_libm.rs` comes from the JDK's `FdLibm.java`, GPLv2 with the Classpath Exception. `src/org/joml/` comes from JOML, MIT, whose notice a distribution has to carry. That's fine while the repo stays private. Shipping a build of rustcraft would put it under GPLv2 too. Not blocking. Decide before the repo or any build goes public.

## Log

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
