//! net/minecraft/client/resources/palette/PaletteMapping.java

use std::collections::HashMap;

use crate::net::minecraft::client::resources::palette::palette::Palette;
use crate::net::minecraft::util::argb::ARGB;

/// Each of a base palette's colors to the target palette's at the same index, by RGB.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteMapping {
    palette: HashMap<i32, i32>,
}

impl PaletteMapping {
    /// Java throws `IllegalArgumentException` for palettes of different sizes, which is the
    /// `Err`. A later base color replaces an earlier one with the same RGB.
    pub fn create(base: &Palette, target: &Palette) -> Result<PaletteMapping, String> {
        if target.size() != base.size() {
            return Err(format!(
                "PaletteMapping has different sizes: {} != {}",
                base.size(),
                target.size()
            ));
        }

        let mut palette = HashMap::with_capacity(base.size());
        for i in 0..base.size() {
            let key = base.get(i);
            if ARGB::alpha(key) != 0 {
                palette.insert(ARGB::opaque(key), target.get(i));
            }
        }
        Ok(PaletteMapping { palette })
    }

    /// A color the base palette has, as the target's, keeping its own alpha times the target's.
    pub fn apply(&self, base_color: i32) -> i32 {
        let base_alpha = ARGB::alpha(base_color);
        if base_alpha == 0 {
            return base_color;
        }

        let base_rgb = ARGB::opaque(base_color);
        let target_rgb = *self.palette.get(&base_rgb).unwrap_or(&base_rgb);
        let value_alpha = ARGB::alpha(target_rgb);
        ARGB::color_alpha(base_alpha * value_alpha / 255, target_rgb)
    }
}
