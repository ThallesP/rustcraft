//! net/minecraft/world/timeline/Timelines.java
//!
//! Only `NIGHT_SKY_LIGHT_COLOR`, which the weather's layers blend toward. The timelines themselves
//! (`bootstrap`, which the data generator writes the vanilla pack's from) come from the pack.

use crate::net::minecraft::util::argb::ARGB;

pub struct Timelines;

impl Timelines {
    /// `ARGB.colorFromFloat(1.0F, 0.48F, 0.48F, 1.0F)`.
    pub fn night_sky_light_color() -> i32 {
        ARGB::color_from_float(1.0, 0.48, 0.48, 1.0)
    }
}
