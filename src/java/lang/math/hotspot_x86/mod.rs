//! HotSpot's x86-64 intrinsics for `Math.sin`, `cos`, `tan`, `log` and `pow`, ported from
//! OpenJDK 25 (adoptium/jdk25u at 520406d8, the source of the Temurin 25.0.4.1 the oracle runs).
//!
//! On x86-64, HotSpot doesn't compute these with fdlibm (StrictMath). The interpreter and both
//! JIT compilers call stubs that `StubGenerator` emits from Intel's libm, whose results differ from
//! fdlibm's and glibc's in the last bit on some inputs. Each file here ports one of OpenJDK's
//! `src/hotspot/cpu/x86/stubGenerator_x86_64_*.cpp` instruction for instruction, on the register
//! model in `assembler`, and reads side by side with it. The stack frame and the stores the stubs
//! make to it become locals.
//!
//! Vanilla on other CPUs (aarch64) runs other stubs, which aren't ported.
//!
//! These files derive from OpenJDK, which is licensed under the GNU General Public License
//! version 2 only, and keep their copyright notices. The rest of the port doesn't depend on them
//! except through `java::lang::math::Math`.

mod assembler;
mod stub_generator_x86_64_constants;
mod stub_generator_x86_64_cos;
mod stub_generator_x86_64_log;
mod stub_generator_x86_64_pow;
mod stub_generator_x86_64_sin;
mod stub_generator_x86_64_tan;

pub use stub_generator_x86_64_cos::libm_cos;
pub use stub_generator_x86_64_log::libm_log;
pub use stub_generator_x86_64_pow::libm_pow;
pub use stub_generator_x86_64_sin::libm_sin;
pub use stub_generator_x86_64_tan::libm_tan;
