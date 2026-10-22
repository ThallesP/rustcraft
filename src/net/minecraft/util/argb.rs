//! net/minecraft/util/ARGB.java
//!
//! Only what Mth needs so far.

pub struct ARGB;

impl ARGB {
    pub fn color(alpha: i32, red: i32, green: i32, blue: i32) -> i32 {
        ((alpha & 0xFF) << 24) | ((red & 0xFF) << 16) | ((green & 0xFF) << 8) | (blue & 0xFF)
    }
}

#[cfg(test)]
mod tests {
    use super::ARGB;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ARGB");
        golden.check("color(IIII)I", ARGB::color);
        golden.pending_rest("not ported yet");
        golden.finish();
    }
}
