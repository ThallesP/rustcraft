//! org/joml/Math.java (JOML 1.10.9)
//!
//! JOML's fast paths (`Options.FASTMATH`, `SIN_LOOKUP`) only run with `-Djoml.fastmath`, which
//! the game doesn't set, so its trig goes through `java.lang.Math`, and so through HotSpot's
//! stubs. Likewise `fma` is `a * b + c`, rounded twice: the fused `java.lang.Math.fma` needs
//! `-Djoml.useMathFma`. Not ported: `random()`, which reads a clock-seeded generator.

use crate::java;
use crate::org::joml::rounding_mode::RoundingMode;

pub struct Math;

impl Math {
    pub const PI: f64 = std::f64::consts::PI;
    pub const PI_TIMES_2: f64 = Math::PI * 2.0;
    pub const PI_F: f32 = std::f64::consts::PI as f32;
    pub const PI_TIMES_2_F: f32 = Math::PI_F * 2.0;
    pub const PI_OVER_2: f64 = Math::PI * 0.5;
    pub const PI_OVER_2_F: f32 = (Math::PI * 0.5) as f32;
    pub const PI_OVER_4: f64 = Math::PI * 0.25;
    pub const PI_OVER_4_F: f32 = (Math::PI * 0.25) as f32;
    pub const ONE_OVER_PI: f64 = 1.0 / Math::PI;
    pub const ONE_OVER_PI_F: f32 = (1.0 / Math::PI) as f32;

    pub fn sin_f32(rad: f32) -> f32 {
        java::lang::math::Math::sin(f64::from(rad)) as f32
    }

    pub fn sin_f64(rad: f64) -> f64 {
        java::lang::math::Math::sin(rad)
    }

    pub fn cos_f32(rad: f32) -> f32 {
        java::lang::math::Math::cos(f64::from(rad)) as f32
    }

    pub fn cos_f64(rad: f64) -> f64 {
        java::lang::math::Math::cos(rad)
    }

    pub fn cos_from_sin_f32(sin: f32, angle: f32) -> f32 {
        Math::cos_from_sin_internal(sin, angle)
    }

    // sin(x)^2 + cos(x)^2 = 1, with the sign from the angle's quadrant.
    fn cos_from_sin_internal(sin: f32, angle: f32) -> f32 {
        let cos = Math::sqrt_f32(1.0 - sin * sin);
        let a = angle + Math::PI_OVER_2_F;
        let mut b = a - (a / Math::PI_TIMES_2_F) as i32 as f32 * Math::PI_TIMES_2_F;
        if f64::from(b) < 0.0 {
            b += Math::PI_TIMES_2_F;
        }
        if b >= Math::PI_F {
            return -cos;
        }
        cos
    }

    pub fn cos_from_sin_f64(sin: f64, angle: f64) -> f64 {
        let cos = Math::sqrt_f64(1.0 - sin * sin);
        let a = angle + Math::PI_OVER_2;
        let mut b = a - f64::from((a / Math::PI_TIMES_2) as i32) * Math::PI_TIMES_2;
        if b < 0.0 {
            b += Math::PI_TIMES_2;
        }
        if b >= Math::PI {
            return -cos;
        }
        cos
    }

    pub fn sqrt_f32(r: f32) -> f32 {
        f64::from(r).sqrt() as f32
    }

    pub fn sqrt_f64(r: f64) -> f64 {
        r.sqrt()
    }

    pub fn invsqrt_f32(r: f32) -> f32 {
        1.0 / f64::from(r).sqrt() as f32
    }

    pub fn invsqrt_f64(r: f64) -> f64 {
        1.0 / r.sqrt()
    }

    pub fn tan_f32(r: f32) -> f32 {
        java::lang::math::Math::tan(f64::from(r)) as f32
    }

    pub fn tan_f64(r: f64) -> f64 {
        java::lang::math::Math::tan(r)
    }

    pub fn acos_f32(r: f32) -> f32 {
        java::lang::math::Math::acos(f64::from(r)) as f32
    }

    pub fn acos_f64(r: f64) -> f64 {
        java::lang::math::Math::acos(r)
    }

    pub fn safe_acos_f32(v: f32) -> f32 {
        if v < -1.0 {
            Math::PI_F
        } else if v > 1.0 {
            0.0
        } else {
            Math::acos_f32(v)
        }
    }

    pub fn safe_acos_f64(v: f64) -> f64 {
        if v < -1.0 {
            Math::PI
        } else if v > 1.0 {
            0.0
        } else {
            Math::acos_f64(v)
        }
    }

    pub fn atan2_f32(y: f32, x: f32) -> f32 {
        java::lang::math::Math::atan2(f64::from(y), f64::from(x)) as f32
    }

    pub fn atan2_f64(y: f64, x: f64) -> f64 {
        java::lang::math::Math::atan2(y, x)
    }

    pub fn asin_f32(r: f32) -> f32 {
        java::lang::math::Math::asin(f64::from(r)) as f32
    }

    pub fn asin_f64(r: f64) -> f64 {
        java::lang::math::Math::asin(r)
    }

    pub fn safe_asin_f32(r: f32) -> f32 {
        if r <= -1.0 {
            -Math::PI_OVER_2_F
        } else if r >= 1.0 {
            Math::PI_OVER_2_F
        } else {
            Math::asin_f32(r)
        }
    }

    pub fn safe_asin_f64(r: f64) -> f64 {
        if r <= -1.0 {
            -Math::PI_OVER_2
        } else if r >= 1.0 {
            Math::PI_OVER_2
        } else {
            Math::asin_f64(r)
        }
    }

    pub fn abs_f32(r: f32) -> f32 {
        r.abs()
    }

    pub fn abs_f64(r: f64) -> f64 {
        r.abs()
    }

    /// Whether `r` is 1 or -1, by its bits.
    pub(crate) fn abs_equals_one_f32(r: f32) -> bool {
        (r.to_bits() & 0x7FFFFFFF) == 0x3F800000
    }

    pub fn abs_i32(r: i32) -> i32 {
        java::lang::math::Math::abs_i32(r)
    }

    pub fn abs_i64(r: i64) -> i64 {
        java::lang::math::Math::abs_i64(r)
    }

    pub fn max_i32(x: i32, y: i32) -> i32 {
        x.max(y)
    }

    pub fn min_i32(x: i32, y: i32) -> i32 {
        x.min(y)
    }

    pub fn max_i64(x: i64, y: i64) -> i64 {
        x.max(y)
    }

    pub fn min_i64(x: i64, y: i64) -> i64 {
        x.min(y)
    }

    // Unlike java.lang.Math's, these return b when either side is NaN, and pick by `<` and `>`
    // alone, so a signed zero comes back from the second argument.
    pub fn min_f64(a: f64, b: f64) -> f64 {
        if a < b { a } else { b }
    }

    pub fn min_f32(a: f32, b: f32) -> f32 {
        if a < b { a } else { b }
    }

    pub fn max_f32(a: f32, b: f32) -> f32 {
        if a > b { a } else { b }
    }

    pub fn max_f64(a: f64, b: f64) -> f64 {
        if a > b { a } else { b }
    }

    pub fn clamp_f32(a: f32, b: f32, val: f32) -> f32 {
        Math::max_f32(a, Math::min_f32(b, val))
    }

    pub fn clamp_f64(a: f64, b: f64, val: f64) -> f64 {
        Math::max_f64(a, Math::min_f64(b, val))
    }

    pub fn clamp_i32(a: i32, b: i32, val: i32) -> i32 {
        Math::max_i32(a, Math::min_i32(b, val))
    }

    pub fn clamp_i64(a: i64, b: i64, val: i64) -> i64 {
        Math::max_i64(a, Math::min_i64(b, val))
    }

    pub fn to_radians_f32(angles: f32) -> f32 {
        java::lang::math::Math::to_radians(f64::from(angles)) as f32
    }

    pub fn to_radians_f64(angles: f64) -> f64 {
        java::lang::math::Math::to_radians(angles)
    }

    pub fn to_degrees_f32(angles: f32) -> f32 {
        java::lang::math::Math::to_degrees(f64::from(angles)) as f32
    }

    pub fn to_degrees_f64(angles: f64) -> f64 {
        java::lang::math::Math::to_degrees(angles)
    }

    pub fn floor_f64(v: f64) -> f64 {
        v.floor()
    }

    pub fn floor_f32(v: f32) -> f32 {
        f64::from(v).floor() as f32
    }

    pub fn ceil_f64(v: f64) -> f64 {
        v.ceil()
    }

    pub fn ceil_f32(v: f32) -> f32 {
        f64::from(v).ceil() as f32
    }

    pub fn round_f64(v: f64) -> i64 {
        java::lang::math::Math::round_f64(v)
    }

    pub fn round_f32(v: f32) -> i32 {
        java::lang::math::Math::round_f32(v)
    }

    pub fn is_finite_f64(d: f64) -> bool {
        Math::abs_f64(d) <= f64::MAX
    }

    pub fn is_finite_f32(f: f32) -> bool {
        Math::abs_f32(f) <= f32::MAX
    }

    /// Not fused: see the header.
    pub fn fma_f32(a: f32, b: f32, c: f32) -> f32 {
        a * b + c
    }

    pub fn fma_f64(a: f64, b: f64, c: f64) -> f64 {
        a * b + c
    }

    /// Panics on an unknown mode, like Java's UnsupportedOperationException.
    pub fn round_using_f32(v: f32, mode: i32) -> i32 {
        match mode {
            RoundingMode::TRUNCATE => v as i32,
            RoundingMode::CEILING => f64::from(v).ceil() as i32,
            RoundingMode::FLOOR => f64::from(v).floor() as i32,
            RoundingMode::HALF_DOWN => Math::round_half_down_f32(v),
            RoundingMode::HALF_UP => Math::round_half_up_f32(v),
            RoundingMode::HALF_EVEN => Math::round_half_even_f32(v),
            _ => panic!("UnsupportedOperationException: rounding mode {mode}"),
        }
    }

    pub fn round_using_f64(v: f64, mode: i32) -> i32 {
        match mode {
            RoundingMode::TRUNCATE => v as i32,
            RoundingMode::CEILING => v.ceil() as i32,
            RoundingMode::FLOOR => v.floor() as i32,
            RoundingMode::HALF_DOWN => Math::round_half_down_f64(v),
            RoundingMode::HALF_UP => Math::round_half_up_f64(v),
            RoundingMode::HALF_EVEN => Math::round_half_even_f64(v),
            _ => panic!("UnsupportedOperationException: rounding mode {mode}"),
        }
    }

    /// The half modes round to an int, not a long, so they saturate at the int range.
    pub fn round_long_using(v: f64, mode: i32) -> i64 {
        match mode {
            RoundingMode::TRUNCATE => v as i64,
            RoundingMode::CEILING => v.ceil() as i64,
            RoundingMode::FLOOR => v.floor() as i64,
            RoundingMode::HALF_DOWN => i64::from(Math::round_half_down_f64(v)),
            RoundingMode::HALF_UP => i64::from(Math::round_half_up_f64(v)),
            RoundingMode::HALF_EVEN => i64::from(Math::round_half_even_f64(v)),
            _ => panic!("UnsupportedOperationException: rounding mode {mode}"),
        }
    }

    pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
        Math::fma_f32(b - a, t, a)
    }

    pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
        Math::fma_f64(b - a, t, a)
    }

    pub fn bi_lerp_f32(q00: f32, q10: f32, q01: f32, q11: f32, tx: f32, ty: f32) -> f32 {
        let lerp_x1 = Math::lerp_f32(q00, q10, tx);
        let lerp_x2 = Math::lerp_f32(q01, q11, tx);
        Math::lerp_f32(lerp_x1, lerp_x2, ty)
    }

    pub fn bi_lerp_f64(q00: f64, q10: f64, q01: f64, q11: f64, tx: f64, ty: f64) -> f64 {
        let lerp_x1 = Math::lerp_f64(q00, q10, tx);
        let lerp_x2 = Math::lerp_f64(q01, q11, tx);
        Math::lerp_f64(lerp_x1, lerp_x2, ty)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tri_lerp_f32(
        q000: f32,
        q100: f32,
        q010: f32,
        q110: f32,
        q001: f32,
        q101: f32,
        q011: f32,
        q111: f32,
        tx: f32,
        ty: f32,
        tz: f32,
    ) -> f32 {
        let x00 = Math::lerp_f32(q000, q100, tx);
        let x10 = Math::lerp_f32(q010, q110, tx);
        let x01 = Math::lerp_f32(q001, q101, tx);
        let x11 = Math::lerp_f32(q011, q111, tx);
        let y0 = Math::lerp_f32(x00, x10, ty);
        let y1 = Math::lerp_f32(x01, x11, ty);
        Math::lerp_f32(y0, y1, tz)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tri_lerp_f64(
        q000: f64,
        q100: f64,
        q010: f64,
        q110: f64,
        q001: f64,
        q101: f64,
        q011: f64,
        q111: f64,
        tx: f64,
        ty: f64,
        tz: f64,
    ) -> f64 {
        let x00 = Math::lerp_f64(q000, q100, tx);
        let x10 = Math::lerp_f64(q010, q110, tx);
        let x01 = Math::lerp_f64(q001, q101, tx);
        let x11 = Math::lerp_f64(q011, q111, tx);
        let y0 = Math::lerp_f64(x00, x10, ty);
        let y1 = Math::lerp_f64(x01, x11, ty);
        Math::lerp_f64(y0, y1, tz)
    }

    pub fn round_half_even_f32(v: f32) -> i32 {
        java::lang::math::Math::rint(f64::from(v)) as i32
    }

    pub fn round_half_down_f32(v: f32) -> i32 {
        if v > 0.0 {
            (f64::from(v) - 0.5).ceil() as i32
        } else {
            (f64::from(v) + 0.5).floor() as i32
        }
    }

    pub fn round_half_up_f32(v: f32) -> i32 {
        if v > 0.0 {
            (f64::from(v) + 0.5).floor() as i32
        } else {
            (f64::from(v) - 0.5).ceil() as i32
        }
    }

    pub fn round_half_even_f64(v: f64) -> i32 {
        java::lang::math::Math::rint(v) as i32
    }

    pub fn round_half_down_f64(v: f64) -> i32 {
        if v > 0.0 {
            (v - 0.5).ceil() as i32
        } else {
            (v + 0.5).floor() as i32
        }
    }

    pub fn round_half_up_f64(v: f64) -> i32 {
        if v > 0.0 {
            (v + 0.5).floor() as i32
        } else {
            (v - 0.5).ceil() as i32
        }
    }

    pub fn round_long_half_even(v: f64) -> i64 {
        java::lang::math::Math::rint(v) as i64
    }

    pub fn round_long_half_down(v: f64) -> i64 {
        if v > 0.0 {
            (v - 0.5).ceil() as i64
        } else {
            (v + 0.5).floor() as i64
        }
    }

    pub fn round_long_half_up(v: f64) -> i64 {
        if v > 0.0 {
            (v + 0.5).floor() as i64
        } else {
            (v - 0.5).ceil() as i64
        }
    }

    pub fn signum_f64(v: f64) -> f64 {
        java::lang::math::Math::signum_f64(v)
    }

    pub fn signum_f32(v: f32) -> f32 {
        java::lang::math::Math::signum_f32(v)
    }

    pub fn signum_i32(v: i32) -> i32 {
        (v >> 31) | ((v.wrapping_neg() as u32) >> 31) as i32
    }

    pub fn signum_i64(v: i64) -> i32 {
        ((v >> 63) | ((v.wrapping_neg() as u64) >> 63) as i64) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::Math;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Math");
        golden.field("PI", Math::PI);
        golden.field("PI_TIMES_2", Math::PI_TIMES_2);
        golden.field("PI_f", Math::PI_F);
        golden.field("PI_TIMES_2_f", Math::PI_TIMES_2_F);
        golden.field("PI_OVER_2", Math::PI_OVER_2);
        golden.field("PI_OVER_2_f", Math::PI_OVER_2_F);
        golden.field("PI_OVER_4", Math::PI_OVER_4);
        golden.field("PI_OVER_4_f", Math::PI_OVER_4_F);
        golden.field("ONE_OVER_PI", Math::ONE_OVER_PI);
        golden.field("ONE_OVER_PI_f", Math::ONE_OVER_PI_F);

        golden.check("sin(F)F", Math::sin_f32);
        golden.check("sin(D)D", Math::sin_f64);
        golden.check("cos(F)F", Math::cos_f32);
        golden.check("cos(D)D", Math::cos_f64);
        golden.check("cosFromSin(FF)F", Math::cos_from_sin_f32);
        golden.check("cosFromSin(DD)D", Math::cos_from_sin_f64);
        golden.check("sqrt(F)F", Math::sqrt_f32);
        golden.check("sqrt(D)D", Math::sqrt_f64);
        golden.check("invsqrt(F)F", Math::invsqrt_f32);
        golden.check("invsqrt(D)D", Math::invsqrt_f64);
        golden.check("tan(F)F", Math::tan_f32);
        golden.check("tan(D)D", Math::tan_f64);
        golden.check("acos(F)F", Math::acos_f32);
        golden.check("acos(D)D", Math::acos_f64);
        golden.check("safeAcos(F)F", Math::safe_acos_f32);
        golden.check("safeAcos(D)D", Math::safe_acos_f64);
        golden.check("atan2(FF)F", Math::atan2_f32);
        golden.check("atan2(DD)D", Math::atan2_f64);
        golden.check("asin(F)F", Math::asin_f32);
        golden.check("asin(D)D", Math::asin_f64);
        golden.check("safeAsin(F)F", Math::safe_asin_f32);
        golden.check("safeAsin(D)D", Math::safe_asin_f64);
        golden.check("abs(F)F", Math::abs_f32);
        golden.check("abs(D)D", Math::abs_f64);
        golden.check("abs(I)I", Math::abs_i32);
        golden.check("abs(J)J", Math::abs_i64);
        golden.check("max(II)I", Math::max_i32);
        golden.check("min(II)I", Math::min_i32);
        golden.check("max(JJ)J", Math::max_i64);
        golden.check("min(JJ)J", Math::min_i64);
        golden.check("min(DD)D", Math::min_f64);
        golden.check("min(FF)F", Math::min_f32);
        golden.check("max(FF)F", Math::max_f32);
        golden.check("max(DD)D", Math::max_f64);
        golden.check("clamp(FFF)F", Math::clamp_f32);
        golden.check("clamp(DDD)D", Math::clamp_f64);
        golden.check("clamp(III)I", Math::clamp_i32);
        golden.check("clamp(JJJ)J", Math::clamp_i64);
        golden.check("toRadians(F)F", Math::to_radians_f32);
        golden.check("toRadians(D)D", Math::to_radians_f64);
        golden.check("toDegrees(F)F", Math::to_degrees_f32);
        golden.check("toDegrees(D)D", Math::to_degrees_f64);
        golden.check("floor(D)D", Math::floor_f64);
        golden.check("floor(F)F", Math::floor_f32);
        golden.check("ceil(D)D", Math::ceil_f64);
        golden.check("ceil(F)F", Math::ceil_f32);
        golden.check("round(D)J", Math::round_f64);
        golden.check("round(F)I", Math::round_f32);
        golden.check("isFinite(D)Z", Math::is_finite_f64);
        golden.check("isFinite(F)Z", Math::is_finite_f32);
        golden.check("fma(FFF)F", Math::fma_f32);
        golden.check("fma(DDD)D", Math::fma_f64);
        golden.check("roundUsing(FI)I", Math::round_using_f32);
        golden.check("roundUsing(DI)I", Math::round_using_f64);
        golden.check("roundLongUsing(DI)J", Math::round_long_using);
        golden.check("lerp(FFF)F", Math::lerp_f32);
        golden.check("lerp(DDD)D", Math::lerp_f64);
        golden.check("biLerp(FFFFFF)F", Math::bi_lerp_f32);
        golden.check("biLerp(DDDDDD)D", Math::bi_lerp_f64);
        golden.check("triLerp(FFFFFFFFFFF)F", Math::tri_lerp_f32);
        golden.check("triLerp(DDDDDDDDDDD)D", Math::tri_lerp_f64);
        golden.check("roundHalfEven(F)I", Math::round_half_even_f32);
        golden.check("roundHalfDown(F)I", Math::round_half_down_f32);
        golden.check("roundHalfUp(F)I", Math::round_half_up_f32);
        golden.check("roundHalfEven(D)I", Math::round_half_even_f64);
        golden.check("roundHalfDown(D)I", Math::round_half_down_f64);
        golden.check("roundHalfUp(D)I", Math::round_half_up_f64);
        golden.check("roundLongHalfEven(D)J", Math::round_long_half_even);
        golden.check("roundLongHalfDown(D)J", Math::round_long_half_down);
        golden.check("roundLongHalfUp(D)J", Math::round_long_half_up);
        golden.check("signum(D)D", Math::signum_f64);
        golden.check("signum(F)F", Math::signum_f32);
        golden.check("signum(I)I", Math::signum_i32);
        golden.check("signum(J)I", Math::signum_i64);

        golden.pending(
            "exp(D)D",
            "HotSpot's exp stub isn't ported, and the client never calls it",
        );
        golden.pending("random()D", "vanilla skips it: seeded from the clock");
        golden.finish();
    }
}
