# Porting rules

How Java becomes Rust in this repo. The port should read side by side with `vanilla/<version>/src/` and match vanilla bit for bit, which TESTING.md checks.

## Layout and names

- One Java file becomes one Rust file at the same path, in snake_case: `net/minecraft/util/Mth.java` → `src/net/minecraft/util/mth.rs`. Each file starts with `//! net/minecraft/util/Mth.java`.
- Class names stay as they are, acronyms included (`Mth`, `ARGB`, `UUID`). Methods and fields become snake_case (`wrapDegrees` → `wrap_degrees`).
- Static methods and constants become associated functions and consts (`Mth::sin`, `Mth::PI`). Static methods of an interface become free functions in its module (`random_source::create`), since a trait has to stay object safe.
- Nested classes live in their outer class's module: `RandomSupport.Seed128bit` → `random_support::Seed128bit`.
- Rust has no overloading, so overloads get a suffix that tells them apart:
  - The Rust type, when they differ by type: `clamp_i32`, `clamp_f32`.
  - What they add, when they differ by arity: `next_int_bounded`, `length3_f64`.
  - The golden test is the mapping table: each `golden.check` line pairs a JVM descriptor with its Rust function.
- Constructors are `new`. Overloaded constructors follow the same rule (`from_seed128bit`).
- JDK behaviour the game depends on lives under `src/java/`, mirroring the JDK's packages (`java::lang::math::Math`, `java::util::uuid::UUID`).
- Anything not ported yet is named in the file's header comment and marked pending in its golden test.

## Object model

- Interfaces become traits. An interface-typed value becomes `Box<dyn Trait>` when owned and `&mut dyn Trait` when borrowed.
- An interface that fills in another interface's methods with defaults (`BitRandomSource`) becomes functions that implementors forward to. A Rust trait can't provide another trait's methods.
- A back-reference, i.e. an object pointing at its owner, becomes a parameter. `MarsagliaPolarGaussian` holds its `RandomSource` in Java; in Rust, the owner passes itself in.
- Concurrency guards that `&mut self` already rules out, such as an AtomicLong checked by a ThreadingDetector, become plain fields.
- Deep class hierarchies (`Entity → LivingEntity → Mob → …`) get settled when the first entity is ported. The plan: entities live in an arena and are addressed by ID, the parent class is embedded as a field, and overridden methods become trait methods.

## Where Java and Rust disagree on numbers

The golden tests caught each of these, or exist to catch them.

- **Overflow.** Java ints and longs wrap; Rust panics in debug builds, so use `wrapping_*`. The port's tests run unoptimized with overflow checks, so a wrap you missed panics instead of passing.
- **Widen where Java widens.** In `Mth.getSeed`, `x * 3129871` is an int multiply that wraps *before* the result widens to long.
- **`>>>`** is a cast to unsigned, a shift and a cast back. `>>` is an arithmetic shift in both languages.
- **`Math.min` / `Math.max` on floats:** a NaN wins and -0.0 is less than 0.0. Rust's `f32::min` returns the operand that isn't NaN, so use `Math::min_f32`.
- **`Math.floorDiv` / `Math.floorMod`** round toward negative infinity, so use `Math::floor_div_i32`. `div_euclid` gives different answers for negative divisors.
- **`Math.abs(Integer.MIN_VALUE)`** wraps, so use `Math::abs_i32`.
- **`clamp`:** Rust's `i32::clamp` panics when min > max and Java's doesn't. Write `max(min).min(max)`.
- **NaN bits:** `Float.floatToIntBits` turns every NaN into the canonical one. `floatToRawIntBits` is `to_bits`.
- **Strings** are UTF-16. `String.hashCode` runs over UTF-16 units (`JavaString::hash_code`), and lengths and indices count UTF-16 units too.
- **Same in both:** `(int)` casts from floating point (they saturate, and NaN becomes 0, like `as`), and float `%` (fmod).
- **Transcendental functions.**
  - On x86-64, HotSpot computes `Math.sin/cos/tan/log/exp/pow` with its own assembly, based on Intel's libm. Functions without that (`asin`, `acos`, `atan`, …) fall back to `StrictMath`, which is fdlibm (the `libm` crate ports fdlibm).
  - glibc and fdlibm differ from the HotSpot routines in the last bit on some inputs. Vanilla itself gives different results on ARM.
  - Every call goes through `java::lang::Math`, so porting the HotSpot routines once fixes every call site. Until then, golden entries that depend on `cos` or `log` are pending.
- **Vineflower and float literals.** Vineflower prints a double constant that fits in a float with an `F` suffix, so `long * 1.110223E-16F` reads as float math when the bytecode multiplies doubles. When an expression's type matters, read the bytecode: `vanilla/toolchain/jdk-25/bin/javap -c -p -cp vanilla/26.3/client.jar net.minecraft.util.Mth`.
- **Methods that never return.** Some vanilla methods never return on extreme inputs (`Mth.rotLerpRad` with a huge difference). Port them as they are; the golden suite keeps its inputs in range with `where()`.

## Optimizing

The port starts literal. An optimization is welcome as long as every golden test, trace, packet capture and screenshot still matches. The comparisons are the contract, not the shape of the code. These look free but change results:

- Fused multiply-add (`mul_add`), or reordering float additions and multiplications. Java rounds every operation in source order.
- Narrowing `f64` math to `f32`, or the reverse.
- Replacing a lookup table with the real function (`Mth.sin` is a 65536-entry table, not `sin`).
- SIMD reductions that sum in a different order.
- Iterating a `HashMap` where Java iterated a `LinkedHashMap`, or relying on any hash order. Java's order is deterministic, Rust's isn't.

## Porting a class

1. Read the Java in `vanilla/<version>/src/`.
2. Add the class to `oracle/golden/src/rustcraft/golden/Golden.java`: a `StaticSuite` for static methods, or a hand-written suite for stateful classes like `RandomSourceSuite`. Then run `python3 tools/golden.py`.
3. Port it, and end the file with its golden test: `golden.check(descriptor, function)` for each method, `golden.pending(name, reason)` for the rest, then `golden.finish()`.
4. Run `cargo test`. A mismatch prints the arguments, vanilla's answer and yours, with floats shown as bits and decimals.
