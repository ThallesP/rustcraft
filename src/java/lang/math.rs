//! java.lang.Math, where it differs from Rust's std.

mod hotspot_x86;

pub struct Math;

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

    /// HotSpot has no assembly for asin, so it's StrictMath's fdlibm, which the libm crate ports too.
    pub fn asin(a: f64) -> f64 {
        libm::asin(a)
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
        golden.check("sqrt(D)D", f64::sqrt);
        golden.finish();
    }
}
