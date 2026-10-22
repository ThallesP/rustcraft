//! java.lang.Math, where it differs from Rust's std.

mod hotspot_x86;

use std::cmp::Ordering;

use crate::java::lang::fd_libm;

pub struct Math;

const DEGREES_TO_RADIANS: f64 = 0.017453292519943295;
const RADIANS_TO_DEGREES: f64 = 57.29577951308232;

impl Math {
    /// Wraps on `i32::MIN` instead of panicking.
    pub fn abs_i32(a: i32) -> i32 {
        a.wrapping_abs()
    }

    pub fn abs_i64(a: i64) -> i64 {
        a.wrapping_abs()
    }

    /// Unlike `f32::min`, a NaN on either side wins and -0.0 is less than 0.0.
    pub fn min_f32(a: f32, b: f32) -> f32 {
        if a.is_nan() {
            return a;
        }
        if a == 0.0 && b == 0.0 && b.is_sign_negative() {
            return b;
        }
        if a <= b { a } else { b }
    }

    pub fn max_f32(a: f32, b: f32) -> f32 {
        if a.is_nan() {
            return a;
        }
        if a == 0.0 && b == 0.0 && a.is_sign_negative() {
            return b;
        }
        if a >= b { a } else { b }
    }

    pub fn min_f64(a: f64, b: f64) -> f64 {
        if a.is_nan() {
            return a;
        }
        if a == 0.0 && b == 0.0 && b.is_sign_negative() {
            return b;
        }
        if a <= b { a } else { b }
    }

    pub fn max_f64(a: f64, b: f64) -> f64 {
        if a.is_nan() {
            return a;
        }
        if a == 0.0 && b == 0.0 && a.is_sign_negative() {
            return b;
        }
        if a >= b { a } else { b }
    }

    /// Rounds toward negative infinity, which `div_euclid` doesn't for negative divisors, and
    /// wraps `i32::MIN / -1`. Panics on zero like Java's ArithmeticException.
    pub fn floor_div_i32(x: i32, y: i32) -> i32 {
        let quotient = x.wrapping_div(y);
        if (x ^ y) < 0 && quotient.wrapping_mul(y) != x {
            quotient - 1
        } else {
            quotient
        }
    }

    pub fn floor_div_i64(x: i64, y: i64) -> i64 {
        let quotient = x.wrapping_div(y);
        if (x ^ y) < 0 && quotient.wrapping_mul(y) != x {
            quotient - 1
        } else {
            quotient
        }
    }

    /// Takes the divisor's sign, which `rem_euclid` doesn't for negative divisors.
    pub fn floor_mod_i32(x: i32, y: i32) -> i32 {
        let modulo = x.wrapping_rem(y);
        if (modulo ^ y) < 0 && modulo != 0 {
            modulo + y
        } else {
            modulo
        }
    }

    pub fn floor_mod_i64(x: i64, y: i64) -> i64 {
        let modulo = x.wrapping_rem(y);
        if (modulo ^ y) < 0 && modulo != 0 {
            modulo + y
        } else {
            modulo
        }
    }

    // On x86-64, HotSpot computes sin, cos, tan, log and pow with its own assembly (from Intel's
    // libm), so these run a port of it. glibc's answers are one ulp off on some inputs. sqrt,
    // floor, ceil and abs are exact everywhere and need no shim.
    pub fn sin(a: f64) -> f64 {
        hotspot_x86::libm_sin(a)
    }

    pub fn cos(a: f64) -> f64 {
        hotspot_x86::libm_cos(a)
    }

    pub fn tan(a: f64) -> f64 {
        hotspot_x86::libm_tan(a)
    }

    pub fn log(a: f64) -> f64 {
        hotspot_x86::libm_log(a)
    }

    pub fn pow(a: f64, b: f64) -> f64 {
        hotspot_x86::libm_pow(a, b)
    }

    /// HotSpot has no assembly for asin, acos, atan2 or hypot, so they're StrictMath's, which runs
    /// the JDK's Java port of fdlibm.
    pub fn asin(a: f64) -> f64 {
        fd_libm::Asin::compute(a)
    }

    pub fn acos(a: f64) -> f64 {
        fd_libm::Acos::compute(a)
    }

    pub fn atan2(y: f64, x: f64) -> f64 {
        fd_libm::Atan2::compute(y, x)
    }

    pub fn hypot(x: f64, y: f64) -> f64 {
        fd_libm::Hypot::compute(x, y)
    }

    pub fn to_radians(angdeg: f64) -> f64 {
        angdeg * DEGREES_TO_RADIANS
    }

    pub fn to_degrees(angrad: f64) -> f64 {
        angrad * RADIANS_TO_DEGREES
    }

    /// Rounds half to even, like `round_ties_even`.
    pub fn rint(a: f64) -> f64 {
        a.round_ties_even()
    }

    /// floor(a + 1/2) without rounding the sum, and saturating like a cast.
    pub fn round_f32(a: f32) -> i32 {
        let int_bits = a.to_bits() as i32;
        let biased_exp = (int_bits & 0x7F800000) >> 23;
        let shift = (24 - 2 + 127) - biased_exp;
        if (shift & -32) == 0 {
            let mut r = (int_bits & 0x007FFFFF) | (0x007FFFFF + 1);
            if int_bits < 0 {
                r = -r;
            }
            ((r >> shift) + 1) >> 1
        } else {
            a as i32
        }
    }

    pub fn round_f64(a: f64) -> i64 {
        let long_bits = a.to_bits() as i64;
        let biased_exp = (long_bits & 0x7FF0000000000000) >> 52;
        let shift = (53 - 2 + 1023) - biased_exp;
        if (shift & -64) == 0 {
            let mut r = (long_bits & 0x000FFFFFFFFFFFFF) | (0x000FFFFFFFFFFFFF + 1);
            if long_bits < 0 {
                r = -r;
            }
            ((r >> shift) + 1) >> 1
        } else {
            a as i64
        }
    }

    /// Unlike `f32::signum`, zeros and NaN come back as they are.
    pub fn signum_f32(f: f32) -> f32 {
        if f == 0.0 || f.is_nan() {
            f
        } else {
            1.0f32.copysign(f)
        }
    }

    pub fn signum_f64(d: f64) -> f64 {
        if d == 0.0 || d.is_nan() {
            d
        } else {
            1.0f64.copysign(d)
        }
    }

    /// `clamp(double, double, double)`. Panics where Java throws `IllegalArgumentException`: on a
    /// NaN bound, or `min` above `max`, which 0.0 is above -0.0.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
        if !(min < max) {
            assert!(!min.is_nan(), "min is NaN");
            assert!(!max.is_nan(), "max is NaN");
            assert!(min.total_cmp(&max) != Ordering::Greater, "{min} > {max}");
        }

        Math::min_f64(max, Math::max_f64(value, min))
    }

    /// The next double toward positive infinity. -0.0 becomes 0.0 first, so its next is
    /// `Double.MIN_VALUE`.
    pub fn next_up_f64(d: f64) -> f64 {
        if d < f64::INFINITY {
            let transducer = (d + 0.0).to_bits() as i64;
            f64::from_bits(transducer.wrapping_add(if transducer >= 0 { 1 } else { -1 }) as u64)
        } else {
            d
        }
    }

    pub fn next_down_f64(d: f64) -> f64 {
        if d.is_nan() || d == f64::NEG_INFINITY {
            d
        } else if d == 0.0 {
            -f64::from_bits(1)
        } else {
            f64::from_bits((d.to_bits() as i64).wrapping_add(if d > 0.0 { -1 } else { 1 }) as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Math;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.Math");
        golden.check("abs(I)I", Math::abs_i32);
        golden.check("abs(J)J", Math::abs_i64);
        golden.check("abs(F)F", f32::abs);
        golden.check("abs(D)D", f64::abs);
        golden.check("min(FF)F", Math::min_f32);
        golden.check("max(FF)F", Math::max_f32);
        golden.check("min(DD)D", Math::min_f64);
        golden.check("max(DD)D", Math::max_f64);
        golden.check("floorDiv(II)I", Math::floor_div_i32);
        golden.check("floorDiv(JJ)J", Math::floor_div_i64);
        golden.check("floorMod(II)I", Math::floor_mod_i32);
        golden.check("floorMod(JJ)J", Math::floor_mod_i64);

        golden.check("sin(D)D", Math::sin);
        golden.check("cos(D)D", Math::cos);
        golden.check("tan(D)D", Math::tan);
        golden.check("log(D)D", Math::log);
        golden.check("pow(DD)D", Math::pow);
        golden.check("asin(D)D", Math::asin);
        golden.check("acos(D)D", Math::acos);
        golden.check("atan2(DD)D", Math::atan2);
        golden.check("hypot(DD)D", Math::hypot);
        golden.check("sqrt(D)D", f64::sqrt);
        golden.check("toRadians(D)D", Math::to_radians);
        golden.check("toDegrees(D)D", Math::to_degrees);
        golden.check("rint(D)D", Math::rint);
        golden.check("round(F)I", Math::round_f32);
        golden.check("round(D)J", Math::round_f64);
        golden.check("signum(F)F", Math::signum_f32);
        golden.check("signum(D)D", Math::signum_f64);
        golden.check("nextUp(D)D", Math::next_up_f64);
        golden.check("nextDown(D)D", Math::next_down_f64);
        golden.check("clamp(DDD)D", Math::clamp_f64);
        golden.finish();
    }
}
