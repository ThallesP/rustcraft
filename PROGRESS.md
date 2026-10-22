# Progress

The loop's memory: newest entries first. Each step adds an entry and rewrites "Next".

## Next

Milestone 1, exact math. Port HotSpot's x86-64 `Math.cos` and `Math.log` (and `sin`, since glibc only happens to agree on the tested inputs) into `src/java/lang/math.rs`. The routines are in OpenJDK 25's `src/hotspot/cpu/x86/macroAssembler_x86_{sin,cos,log}.cpp`. Then un-pend `cos(D)D` and `log(D)D` in the `java.lang.Math` golden test, `atan2(DD)D` in `Mth`, and the `+nextGaussian` programs in `RandomSource`. Add many more random inputs for these functions to the golden suite, since 1-ulp disagreements are rare. Those OpenJDK files are GPLv2, so keep the port in its own module (e.g. `src/java/lang/math/hotspot_x86.rs`) with their copyright notice.

## Needs Thalles

Nothing yet.

## Log

### 2026-10-22: setup

- Vanilla 26.3 is decompiled into `vanilla/` (gitignored). The oracle covers four layers: function goldens, tick traces, packet captures and screenshots. See TESTING.md.
- Two vanilla runs of `walk` through a vanilla server match exactly (`check-determinism walk --capture`).
- Ported, and matching vanilla bit for bit: `Mth`, `RandomSource` and its four implementations, `RandomSupport`, `Xoroshiro128PlusPlus`, `MarsagliaPolarGaussian`, plus the JDK pieces they use (`Math`, `String.hashCode`, `UUID`).
- Pending, because HotSpot's x86-64 `cos`/`log` aren't ported: `Math.cos`, `Math.log`, `Mth.atan2`, and gaussian draws.
- Decided: a multiplayer-only client first. Entities will live in an arena addressed by ID when they come.
