//! net/minecraft/util/ARGB.java
//!
//! Only what Mth, `ByteBufCodecs.RGB_COLOR`, clear colors (`vector4fFromARGB32`), vertex and
//! texture colors (`toABGR`) and mipmaps (the sRGB tables, `meanLinear`) need so far.

use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;
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

    /// `color(int, int, int)`: opaque.
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

    fn from8_bit_channel(value: i32) -> f32 {
        value as f32 / 255.0
    }
}

#[cfg(test)]
mod tests {
    use super::ARGB;
    use crate::golden::Golden;
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
        golden.pending_rest("not ported yet");
        golden.finish();
    }
}
