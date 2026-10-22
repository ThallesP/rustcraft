//! net/minecraft/util/ARGB.java
//!
//! Only what Mth and `ByteBufCodecs.RGB_COLOR` need so far.

pub struct ARGB;

impl ARGB {
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
}

#[cfg(test)]
mod tests {
    use super::ARGB;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ARGB");
        golden.check("red(I)I", ARGB::red);
        golden.check("green(I)I", ARGB::green);
        golden.check("blue(I)I", ARGB::blue);
        golden.check("color(IIII)I", ARGB::color);
        golden.check("color(III)I", ARGB::color_rgb);
        golden.pending_rest("not ported yet");
        golden.finish();
    }
}
