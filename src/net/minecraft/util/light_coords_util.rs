//! net/minecraft/util/LightCoordsUtil.java
//!
//! Light packed the way vertices carry it: block light in bits 4 to 7, sky light in bits 20 to
//! 23, or the smooth form's bytes 0 and 2, which hold 16 times as much. `BrightnessGetter` is a
//! function of the level and the block, `DEFAULT` the level's two layers.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::light_layer::LightLayer;

pub struct LightCoordsUtil;

/// `LightCoordsUtil.BrightnessGetter`.
pub type BrightnessGetter<'a> = &'a dyn Fn(&dyn BlockAndLightGetter, &BlockPos) -> i32;

impl LightCoordsUtil {
    pub const FULL_BRIGHT: i32 = 15728880;
    pub const FULL_SKY: i32 = 15728640;

    pub fn pack(block: i32, sky: i32) -> i32 {
        block << 4 | sky << 20
    }

    pub fn block(packed: i32) -> i32 {
        packed >> 4 & 15
    }

    pub fn sky(packed: i32) -> i32 {
        packed >> 20 & 15
    }

    pub fn with_block(coords: i32, block: i32) -> i32 {
        coords & 0xFF0000 | block << 4
    }

    pub fn smooth_pack(block: i32, sky: i32) -> i32 {
        block & 0xFF | (sky & 0xFF) << 16
    }

    pub fn smooth_block(packed: i32) -> i32 {
        packed & 0xFF
    }

    pub fn smooth_sky(packed: i32) -> i32 {
        packed >> 16 & 0xFF
    }

    pub fn add_smooth_block_emission(light_coords: i32, block_light_emission: f32) -> i32 {
        let block_light_emission = Mth::clamp_f32(block_light_emission, 0.0, 1.0);
        let emitted_block = (Mth::clamp_f32(block_light_emission, 0.0, 1.0) * 240.0) as i32;
        let block = (LightCoordsUtil::smooth_block(light_coords) + emitted_block).min(240);
        LightCoordsUtil::smooth_pack(block, LightCoordsUtil::smooth_sky(light_coords))
    }

    pub fn max(coords1: i32, coords2: i32) -> i32 {
        let block1 = LightCoordsUtil::block(coords1);
        let block2 = LightCoordsUtil::block(coords2);
        let sky1 = LightCoordsUtil::sky(coords1);
        let sky2 = LightCoordsUtil::sky(coords2);
        LightCoordsUtil::pack(block1.max(block2), sky1.max(sky2))
    }

    pub fn light_coords_with_emission(light_coords: i32, emission: i32) -> i32 {
        if emission == 0 {
            return light_coords;
        }

        let sky = LightCoordsUtil::sky(light_coords).max(emission);
        let block = LightCoordsUtil::block(light_coords).max(emission);
        LightCoordsUtil::pack(block, sky)
    }

    /// The four corners' light averaged, a dark corner taking the center's light (and an unlit
    /// sky the center's sky) when the center has any to speak of.
    pub fn smooth_blend(
        mut neighbor1: i32,
        mut neighbor2: i32,
        mut neighbor3: i32,
        center: i32,
    ) -> i32 {
        if LightCoordsUtil::sky(center) > 2 || LightCoordsUtil::block(center) > 2 {
            for neighbor in [&mut neighbor1, &mut neighbor2, &mut neighbor3] {
                if *neighbor == 0 {
                    *neighbor = center;
                } else if LightCoordsUtil::sky(*neighbor) == 0 {
                    *neighbor |= center & 0xFF0000;
                }
            }
        }

        (neighbor1
            .wrapping_add(neighbor2)
            .wrapping_add(neighbor3)
            .wrapping_add(center)
            >> 2)
            & 16711935
    }

    #[allow(clippy::too_many_arguments)]
    pub fn smooth_weighted_blend(
        coords1: i32,
        coords2: i32,
        coords3: i32,
        coords4: i32,
        weight1: f32,
        weight2: f32,
        weight3: f32,
        weight4: f32,
    ) -> i32 {
        let sky = (LightCoordsUtil::smooth_sky(coords1) as f32 * weight1
            + LightCoordsUtil::smooth_sky(coords2) as f32 * weight2
            + LightCoordsUtil::smooth_sky(coords3) as f32 * weight3
            + LightCoordsUtil::smooth_sky(coords4) as f32 * weight4) as i32;
        let block = (LightCoordsUtil::smooth_block(coords1) as f32 * weight1
            + LightCoordsUtil::smooth_block(coords2) as f32 * weight2
            + LightCoordsUtil::smooth_block(coords3) as f32 * weight3
            + LightCoordsUtil::smooth_block(coords4) as f32 * weight4) as i32;
        LightCoordsUtil::smooth_pack(block, sky)
    }

    /// `BrightnessGetter.DEFAULT`.
    pub fn default_brightness(level: &dyn BlockAndLightGetter, pos: &BlockPos) -> i32 {
        let sky = level.get_brightness(LightLayer::SKY, pos);
        let block = level.get_brightness(LightLayer::BLOCK, pos);
        LightCoordsUtil::pack(block, sky)
    }

    pub fn get_light_coords_at(level: &dyn BlockAndLightGetter, pos: &BlockPos) -> i32 {
        LightCoordsUtil::get_light_coords(
            &LightCoordsUtil::default_brightness,
            level,
            level.get_block_state(pos),
            pos,
        )
    }

    /// Full bright for an emissive block, else the block's light with its own emission as the
    /// least block light.
    pub fn get_light_coords(
        brightness_getter: BrightnessGetter<'_>,
        level: &dyn BlockAndLightGetter,
        state: BlockState,
        pos: &BlockPos,
    ) -> i32 {
        if state.emissive_rendering() {
            return LightCoordsUtil::FULL_BRIGHT;
        }

        let packed_brightness = brightness_getter(level, pos);
        let block = LightCoordsUtil::block(packed_brightness);
        let block_self_emission = state.get_light_emission();
        if block < block_self_emission {
            LightCoordsUtil::with_block(packed_brightness, block_self_emission)
        } else {
            packed_brightness
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LightCoordsUtil;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.LightCoordsUtil");
        golden.field("FULL_BRIGHT", LightCoordsUtil::FULL_BRIGHT);
        golden.field("FULL_SKY", LightCoordsUtil::FULL_SKY);
        golden.check("pack(II)I", LightCoordsUtil::pack);
        golden.check("block(I)I", LightCoordsUtil::block);
        golden.check("sky(I)I", LightCoordsUtil::sky);
        golden.check("withBlock(II)I", LightCoordsUtil::with_block);
        golden.check("smoothPack(II)I", LightCoordsUtil::smooth_pack);
        golden.check("smoothBlock(I)I", LightCoordsUtil::smooth_block);
        golden.check("smoothSky(I)I", LightCoordsUtil::smooth_sky);
        golden.check(
            "addSmoothBlockEmission(IF)I",
            LightCoordsUtil::add_smooth_block_emission,
        );
        golden.check("max(II)I", LightCoordsUtil::max);
        golden.check(
            "lightCoordsWithEmission(II)I",
            LightCoordsUtil::light_coords_with_emission,
        );
        golden.check("smoothBlend(IIII)I", LightCoordsUtil::smooth_blend);
        golden.check(
            "smoothWeightedBlend(IIIIFFFF)I",
            LightCoordsUtil::smooth_weighted_blend,
        );
        golden.pending_rest(
            "getLightCoords takes a level: checked through ModelBlockRenderer's meshes",
        );
        golden.finish();
    }
}
