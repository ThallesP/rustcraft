//! net/minecraft/util/ARGB.java
//!
//! Only what Mth, `ByteBufCodecs.RGB_COLOR`, clear colors (`vector4fFromARGB32`), vertex and
//! texture colors (`toABGR`), mipmaps (the sRGB tables, `meanLinear`) and the environment
//! attributes' colors (the `Vector3fc` and `Vector4fc` overloads, `colorFromFloat`) need so far.
//! The vector overloads that hand back one of their arguments in Java hand back a copy.

use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;
use std::sync::LazyLock;

/// Each sRGB channel's linear value, out of 1023.
static SRGB_TO_LINEAR: LazyLock<[i16; 256]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let channel = i as f32 / 255.0;
        Math::round_f32(ARGB::compute_srgb_to_linear(channel) * 1023.0) as i16
    })
});

/// Each linear value out of 1023's sRGB channel.
static LINEAR_TO_SRGB: LazyLock<[i8; 1024]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let channel = i as f32 / 1023.0;
        Math::round_f32(ARGB::compute_linear_to_srgb(channel) * 255.0) as i8
    })
});

pub struct ARGB;

impl ARGB {
    fn compute_srgb_to_linear(x: f32) -> f32 {
        if x >= 0.04045 {
            Math::pow((f64::from(x) + 0.055) / 1.055, 2.4) as f32
        } else {
            x / 12.92
        }
    }

    fn compute_linear_to_srgb(x: f32) -> f32 {
        if x >= 0.0031308 {
            (1.055 * Math::pow(f64::from(x), 0.4166666666666667) - 0.055) as f32
        } else {
            12.92 * x
        }
    }

    /// Panics where Java throws `ArrayIndexOutOfBoundsException`, for a channel outside 0..=255.
    pub fn srgb_to_linear_channel(srgb: i32) -> f32 {
        f32::from(SRGB_TO_LINEAR[srgb as usize]) / 1023.0
    }

    /// Panics where Java throws `ArrayIndexOutOfBoundsException`, for a value outside 0..=1.
    pub fn linear_to_srgb_channel(linear: f32) -> i32 {
        let index = Mth::floor_f32(linear * 1023.0);
        assert!(
            (0..1024).contains(&index),
            "Index {index} out of bounds for length 1024"
        );
        i32::from(LINEAR_TO_SRGB[index as usize]) & 0xFF
    }

    /// The four colors' mean: alpha's directly, the others' in linear light.
    pub fn mean_linear(srgb1: i32, srgb2: i32, srgb3: i32, srgb4: i32) -> i32 {
        ARGB::color(
            (ARGB::alpha(srgb1) + ARGB::alpha(srgb2) + ARGB::alpha(srgb3) + ARGB::alpha(srgb4)) / 4,
            ARGB::linear_channel_mean(
                ARGB::red(srgb1),
                ARGB::red(srgb2),
                ARGB::red(srgb3),
                ARGB::red(srgb4),
            ),
            ARGB::linear_channel_mean(
                ARGB::green(srgb1),
                ARGB::green(srgb2),
                ARGB::green(srgb3),
                ARGB::green(srgb4),
            ),
            ARGB::linear_channel_mean(
                ARGB::blue(srgb1),
                ARGB::blue(srgb2),
                ARGB::blue(srgb3),
                ARGB::blue(srgb4),
            ),
        )
    }

    fn linear_channel_mean(c1: i32, c2: i32, c3: i32, c4: i32) -> i32 {
        let linear = (i32::from(SRGB_TO_LINEAR[c1 as usize])
            + i32::from(SRGB_TO_LINEAR[c2 as usize])
            + i32::from(SRGB_TO_LINEAR[c3 as usize])
            + i32::from(SRGB_TO_LINEAR[c4 as usize]))
            / 4;
        i32::from(LINEAR_TO_SRGB[linear as usize]) & 0xFF
    }

    pub fn as8_bit_channel(value: f32) -> i32 {
        Mth::floor_f32(value * 255.0)
    }

    /// `color(float, int)`: the RGB with an alpha from 0 to 1.
    pub fn color_alpha_f32(alpha: f32, rgb: i32) -> i32 {
        ARGB::as8_bit_channel(alpha) << 24 | rgb & 16777215
    }

    /// `color(int, int)`: the RGB with an alpha.
    pub fn color_alpha(alpha: i32, rgb: i32) -> i32 {
        alpha << 24 | rgb & 16777215
    }

    /// The color with its alpha full.
    pub fn opaque(color: i32) -> i32 {
        color | 0xFF000000_u32 as i32
    }

    /// `color >>> 24`.
    pub fn alpha(color: i32) -> i32 {
        ((color as u32) >> 24) as i32
    }

    pub fn red(color: i32) -> i32 {
        (color >> 16) & 0xFF
    }

    pub fn green(color: i32) -> i32 {
        (color >> 8) & 0xFF
    }

    pub fn blue(color: i32) -> i32 {
        color & 0xFF
    }

    pub fn color(alpha: i32, red: i32, green: i32, blue: i32) -> i32 {
        ((alpha & 0xFF) << 24) | ((red & 0xFF) << 16) | ((green & 0xFF) << 8) | (blue & 0xFF)
    }

    /// Both colors' channels multiplied; white (-1) leaves the other as it is.
    pub fn multiply(lhs: i32, rhs: i32) -> i32 {
        if lhs == -1 {
            rhs
        } else if rhs == -1 {
            lhs
        } else {
            ARGB::color(
                ARGB::alpha(lhs) * ARGB::alpha(rhs) / 255,
                ARGB::red(lhs) * ARGB::red(rhs) / 255,
                ARGB::green(lhs) * ARGB::green(rhs) / 255,
                ARGB::blue(lhs) * ARGB::blue(rhs) / 255,
            )
        }
    }

    pub fn scale_rgb(color: i32, scale: f32) -> i32 {
        ARGB::scale_rgb3(color, scale, scale, scale)
    }

    /// Each channel scaled, cut (Java's `(int)`) and clamped to a byte (`Math.clamp(long, int,
    /// int)`).
    pub fn scale_rgb3(color: i32, scale_r: f32, scale_g: f32, scale_b: f32) -> i32 {
        let channel = |value: i32, scale: f32| ((value as f32 * scale) as i32).clamp(0, 255);
        ARGB::color(
            ARGB::alpha(color),
            channel(ARGB::red(color), scale_r),
            channel(ARGB::green(color), scale_g),
            channel(ARGB::blue(color), scale_b),
        )
    }

    /// An opaque gray of the brightness.
    pub fn gray(brightness: f32) -> i32 {
        let channel = ARGB::as8_bit_channel(brightness);
        ARGB::color_rgb(channel, channel, channel)
    }

    /// `color(int, int, int)`: opaque.
    /// `white(float)`: white with the alpha as an 8-bit channel.
    pub fn white_f32(alpha: f32) -> i32 {
        ARGB::as8_bit_channel(alpha) << 24 | 0xFFFFFF
    }

    /// `white(int)`.
    pub fn white(alpha: i32) -> i32 {
        alpha << 24 | 0xFFFFFF
    }

    pub fn color_rgb(red: i32, green: i32, blue: i32) -> i32 {
        ARGB::color(255, red, green, blue)
    }

    pub fn vector4f_from_argb32(color: i32) -> Vector4f {
        Vector4f::from_xyzw(
            ARGB::red_float(color),
            ARGB::green_float(color),
            ARGB::blue_float(color),
            ARGB::alpha_float(color),
        )
    }

    pub fn set_vector4f_from_argb32(dest: &mut Vector4f, color: i32) -> Vector4f {
        *dest.set_xyzw(
            ARGB::red_float(color),
            ARGB::green_float(color),
            ARGB::blue_float(color),
            ARGB::alpha_float(color),
        )
    }

    pub fn alpha_float(color: i32) -> f32 {
        ARGB::from8_bit_channel(ARGB::alpha(color))
    }

    pub fn red_float(color: i32) -> f32 {
        ARGB::from8_bit_channel(ARGB::red(color))
    }

    pub fn green_float(color: i32) -> f32 {
        ARGB::from8_bit_channel(ARGB::green(color))
    }

    pub fn blue_float(color: i32) -> f32 {
        ARGB::from8_bit_channel(ARGB::blue(color))
    }

    /// Swaps red and blue: an ARGB color as the bytes red, green, blue, alpha in a little-endian
    /// int, which is how textures and vertices hold colors.
    pub fn to_abgr(color: i32) -> i32 {
        color & -16711936 | (color & 0xFF0000) >> 16 | (color & 0xFF) << 16
    }

    pub fn from_abgr(color: i32) -> i32 {
        ARGB::to_abgr(color)
    }

    /// `colorFromFloat`: each channel from 0 to 1.
    pub fn color_from_float(alpha: f32, red: f32, green: f32, blue: f32) -> i32 {
        ARGB::color(
            ARGB::as8_bit_channel(alpha),
            ARGB::as8_bit_channel(red),
            ARGB::as8_bit_channel(green),
            ARGB::as8_bit_channel(blue),
        )
    }

    pub fn vector3f_from_rgb24(color: i32) -> Vector3f {
        Vector3f::from_xyz(
            ARGB::red_float(color),
            ARGB::green_float(color),
            ARGB::blue_float(color),
        )
    }

    /// `multiply(Vector3fc, Vector3fc)`: white on either side leaves the other as it is.
    pub fn multiply_vector3f(lhs: &Vector3f, rhs: &Vector3f) -> Vector3f {
        if lhs.x == 1.0 && lhs.y == 1.0 && lhs.z == 1.0 {
            *rhs
        } else if rhs.x == 1.0 && rhs.y == 1.0 && rhs.z == 1.0 {
            *lhs
        } else {
            Vector3f::from_xyz(lhs.x * rhs.x, lhs.y * rhs.y, lhs.z * rhs.z)
        }
    }

    pub fn multiply_vector4f(lhs: &Vector4f, rhs: &Vector4f) -> Vector4f {
        if lhs.x == 1.0 && lhs.y == 1.0 && lhs.z == 1.0 && lhs.w == 1.0 {
            *rhs
        } else if rhs.x == 1.0 && rhs.y == 1.0 && rhs.z == 1.0 && rhs.w == 1.0 {
            *lhs
        } else {
            let mut dest = Vector4f::new();
            lhs.mul_into(rhs, &mut dest);
            dest
        }
    }

    pub fn add_rgb_vector3f(lhs: &Vector3f, rhs: &Vector3f) -> Vector3f {
        Vector3f::from_xyz(
            Math::min_f32(lhs.x + rhs.x, 1.0),
            Math::min_f32(lhs.y + rhs.y, 1.0),
            Math::min_f32(lhs.z + rhs.z, 1.0),
        )
    }

    pub fn add_rgb_vector4f(lhs: &Vector4f, rhs: &Vector3f) -> Vector4f {
        Vector4f::from_xyzw(
            Math::min_f32(lhs.x + rhs.x, 1.0),
            Math::min_f32(lhs.y + rhs.y, 1.0),
            Math::min_f32(lhs.z + rhs.z, 1.0),
            lhs.w,
        )
    }

    pub fn subtract_rgb_vector3f(lhs: &Vector3f, rhs: &Vector3f) -> Vector3f {
        Vector3f::from_xyz(
            Math::max_f32(lhs.x - rhs.x, 0.0),
            Math::max_f32(lhs.y - rhs.y, 0.0),
            Math::max_f32(lhs.z - rhs.z, 0.0),
        )
    }

    pub fn subtract_rgb_vector4f(lhs: &Vector4f, rhs: &Vector3f) -> Vector4f {
        Vector4f::from_xyzw(
            Math::max_f32(lhs.x - rhs.x, 0.0),
            Math::max_f32(lhs.y - rhs.y, 0.0),
            Math::max_f32(lhs.z - rhs.z, 0.0),
            lhs.w,
        )
    }

    pub fn scale_rgb_vector3f(color: &Vector3f, scale: f32) -> Vector3f {
        ARGB::scale_rgb3_vector3f(color, scale, scale, scale)
    }

    pub fn scale_rgb_vector4f(color: &Vector4f, scale: f32) -> Vector4f {
        ARGB::scale_rgb3_vector4f(color, scale, scale, scale)
    }

    /// Each channel scaled and clamped to 0..1 (`Math.clamp(float, float, float)`).
    pub fn scale_rgb3_vector3f(
        color: &Vector3f,
        scale_r: f32,
        scale_g: f32,
        scale_b: f32,
    ) -> Vector3f {
        Vector3f::from_xyz(
            Math::clamp_f32(color.x * scale_r, 0.0, 1.0),
            Math::clamp_f32(color.y * scale_g, 0.0, 1.0),
            Math::clamp_f32(color.z * scale_b, 0.0, 1.0),
        )
    }

    pub fn scale_rgb3_vector4f(
        color: &Vector4f,
        scale_r: f32,
        scale_g: f32,
        scale_b: f32,
    ) -> Vector4f {
        Vector4f::from_xyzw(
            Math::clamp_f32(color.x * scale_r, 0.0, 1.0),
            Math::clamp_f32(color.y * scale_g, 0.0, 1.0),
            Math::clamp_f32(color.z * scale_b, 0.0, 1.0),
            color.w,
        )
    }

    pub fn greyscale_vector3f(color: &Vector3f) -> Vector3f {
        let greyscale = color.x * 0.3 + color.y * 0.59 + color.z * 0.11;
        Vector3f::from_xyz(greyscale, greyscale, greyscale)
    }

    pub fn greyscale_vector4f(color: &Vector4f) -> Vector4f {
        let greyscale = color.x * 0.3 + color.y * 0.59 + color.z * 0.11;
        Vector4f::from_xyzw(greyscale, greyscale, greyscale, color.w)
    }

    /// `alphaBlend(Vector3fc, Vector4fc)`: the source's RGB over the destination by its alpha.
    pub fn alpha_blend_vector3f(destination: &Vector3f, source: &Vector4f) -> Vector3f {
        let source_alpha = source.w;
        if source_alpha == 0.0 {
            return *destination;
        }

        let mut source_rgb = Vector3f::from_xyz(source.x, source.y, source.z);
        let other = source_rgb;
        destination.lerp_into(&other, source_alpha, &mut source_rgb);
        source_rgb
    }

    pub fn alpha_blend_vector4f(destination: &Vector4f, source: &Vector4f) -> Vector4f {
        let destination_alpha = destination.w;
        let source_alpha = source.w;
        if source_alpha == 1.0 {
            return *source;
        }

        if source_alpha == 0.0 {
            return *destination;
        }

        let alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
        let channel = |destination: f32, source: f32| {
            (source * source_alpha + destination * (alpha - source_alpha)) / alpha
        };
        Vector4f::from_xyzw(
            channel(destination.x, source.x),
            channel(destination.y, source.y),
            channel(destination.z, source.z),
            alpha,
        )
    }

    /// `srgbLerp(float, Vector3fc, Vector3fc)`: JOML's `lerp`, despite the name.
    pub fn srgb_lerp_vector3f(alpha: f32, p0: &Vector3f, p1: &Vector3f) -> Vector3f {
        let mut dest = Vector3f::new();
        p0.lerp_into(p1, alpha, &mut dest);
        dest
    }

    pub fn srgb_lerp_vector4f(alpha: f32, p0: &Vector4f, p1: &Vector4f) -> Vector4f {
        let mut dest = Vector4f::new();
        p0.lerp_into(p1, alpha, &mut dest);
        dest
    }

    fn from8_bit_channel(value: i32) -> f32 {
        value as f32 / 255.0
    }
}

#[cfg(test)]
mod tests {
    use super::ARGB;
    use crate::golden::Golden;
    use crate::org::joml::vector3f::Vector3f;
    use crate::org::joml::vector4f::Vector4f;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ARGB");
        golden.check("red(I)I", ARGB::red);
        golden.check("green(I)I", ARGB::green);
        golden.check("blue(I)I", ARGB::blue);
        golden.check("color(IIII)I", ARGB::color);
        golden.check("color(III)I", ARGB::color_rgb);
        golden.check("alpha(I)I", ARGB::alpha);
        golden.check("opaque(I)I", ARGB::opaque);
        golden.check("toABGR(I)I", ARGB::to_abgr);
        golden.check("srgbToLinearChannel(I)F", ARGB::srgb_to_linear_channel);
        golden.check("linearToSrgbChannel(F)I", ARGB::linear_to_srgb_channel);
        golden.check("meanLinear(IIII)I", ARGB::mean_linear);
        golden.check("as8BitChannel(F)I", ARGB::as8_bit_channel);
        golden.check("color(FI)I", ARGB::color_alpha_f32);
        golden.check("color(II)I", ARGB::color_alpha);
        golden.check("multiply(II)I", ARGB::multiply);
        golden.check("scaleRGB(IF)I", ARGB::scale_rgb);
        golden.check("scaleRGB(IFFF)I", ARGB::scale_rgb3);
        golden.check("gray(F)I", ARGB::gray);
        golden.check("fromABGR(I)I", ARGB::from_abgr);
        golden.check("alphaFloat(I)F", ARGB::alpha_float);
        golden.check("redFloat(I)F", ARGB::red_float);
        golden.check("greenFloat(I)F", ARGB::green_float);
        golden.check("blueFloat(I)F", ARGB::blue_float);
        golden.check(
            "vector4fFromARGB32(I)Lorg/joml/Vector4f;",
            ARGB::vector4f_from_argb32,
        );
        golden.check_mut(
            "setVector4fFromARGB32(Lorg/joml/Vector4f;I)Lorg/joml/Vector4f;",
            |dest: &mut Vector4f, color: &mut i32| ARGB::set_vector4f_from_argb32(dest, *color),
        );
        golden.check("colorFromFloat(FFFF)I", ARGB::color_from_float);
        golden.check("white(F)I", ARGB::white_f32);
        golden.check("white(I)I", ARGB::white);
        golden.check(
            "vector3fFromRGB24(I)Lorg/joml/Vector3f;",
            ARGB::vector3f_from_rgb24,
        );
        golden.check(
            "multiply(Lorg/joml/Vector3fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector3fc;",
            |lhs: Vector3f, rhs: Vector3f| ARGB::multiply_vector3f(&lhs, &rhs),
        );
        golden.check(
            "multiply(Lorg/joml/Vector4fc;Lorg/joml/Vector4fc;)Lorg/joml/Vector4fc;",
            |lhs: Vector4f, rhs: Vector4f| ARGB::multiply_vector4f(&lhs, &rhs),
        );
        golden.check(
            "addRgb(Lorg/joml/Vector3fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector3fc;",
            |lhs: Vector3f, rhs: Vector3f| ARGB::add_rgb_vector3f(&lhs, &rhs),
        );
        golden.check(
            "addRgb(Lorg/joml/Vector4fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector4fc;",
            |lhs: Vector4f, rhs: Vector3f| ARGB::add_rgb_vector4f(&lhs, &rhs),
        );
        golden.check(
            "subtractRgb(Lorg/joml/Vector3fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector3fc;",
            |lhs: Vector3f, rhs: Vector3f| ARGB::subtract_rgb_vector3f(&lhs, &rhs),
        );
        golden.check(
            "subtractRgb(Lorg/joml/Vector4fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector4fc;",
            |lhs: Vector4f, rhs: Vector3f| ARGB::subtract_rgb_vector4f(&lhs, &rhs),
        );
        golden.check(
            "scaleRGB(Lorg/joml/Vector3fc;F)Lorg/joml/Vector3fc;",
            |color: Vector3f, scale: f32| ARGB::scale_rgb_vector3f(&color, scale),
        );
        golden.check(
            "scaleRGB(Lorg/joml/Vector4fc;F)Lorg/joml/Vector4fc;",
            |color: Vector4f, scale: f32| ARGB::scale_rgb_vector4f(&color, scale),
        );
        golden.check(
            "scaleRGB(Lorg/joml/Vector3fc;FFF)Lorg/joml/Vector3fc;",
            |color: Vector3f, r: f32, g: f32, b: f32| ARGB::scale_rgb3_vector3f(&color, r, g, b),
        );
        golden.check(
            "scaleRGB(Lorg/joml/Vector4fc;FFF)Lorg/joml/Vector4fc;",
            |color: Vector4f, r: f32, g: f32, b: f32| ARGB::scale_rgb3_vector4f(&color, r, g, b),
        );
        golden.check(
            "greyscale(Lorg/joml/Vector3fc;)Lorg/joml/Vector3fc;",
            |color: Vector3f| ARGB::greyscale_vector3f(&color),
        );
        golden.check(
            "greyscale(Lorg/joml/Vector4fc;)Lorg/joml/Vector4fc;",
            |color: Vector4f| ARGB::greyscale_vector4f(&color),
        );
        golden.check(
            "alphaBlend(Lorg/joml/Vector3fc;Lorg/joml/Vector4fc;)Lorg/joml/Vector3fc;",
            |destination: Vector3f, source: Vector4f| {
                ARGB::alpha_blend_vector3f(&destination, &source)
            },
        );
        golden.check(
            "alphaBlend(Lorg/joml/Vector4fc;Lorg/joml/Vector4fc;)Lorg/joml/Vector4fc;",
            |destination: Vector4f, source: Vector4f| {
                ARGB::alpha_blend_vector4f(&destination, &source)
            },
        );
        golden.check(
            "srgbLerp(FLorg/joml/Vector3fc;Lorg/joml/Vector3fc;)Lorg/joml/Vector3fc;",
            |alpha: f32, p0: Vector3f, p1: Vector3f| ARGB::srgb_lerp_vector3f(alpha, &p0, &p1),
        );
        golden.check(
            "srgbLerp(FLorg/joml/Vector4fc;Lorg/joml/Vector4fc;)Lorg/joml/Vector4fc;",
            |alpha: f32, p0: Vector4f, p1: Vector4f| ARGB::srgb_lerp_vector4f(alpha, &p0, &p1),
        );
        golden.pending_rest("not ported yet");
        golden.finish();
    }
}
