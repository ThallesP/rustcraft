//! net/minecraft/util/ARGB.java
//!
//! Only what Mth, `ByteBufCodecs.RGB_COLOR` and clear colors (`vector4fFromARGB32`) need so far.

use crate::org::joml::vector4f::Vector4f;

pub struct ARGB;

impl ARGB {
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
