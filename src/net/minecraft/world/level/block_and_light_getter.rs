//! net/minecraft/world/level/BlockAndLightGetter.java

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;

/// Blocks and their light.
pub trait BlockAndLightGetter: BlockGetter {
    fn get_light_engine(&self) -> &LevelLightEngine;

    /// The layer's light at the block.
    fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
        self.get_light_engine()
            .get_layer_listener(layer)
            .get_light_value(pos)
    }

    /// The brighter of the block's light and the sky's, the sky's dimmed by `darkening`.
    fn get_raw_brightness(&self, pos: &BlockPos, darkening: i32) -> i32 {
        self.get_light_engine().get_raw_brightness(pos, darkening)
    }

    fn can_see_sky(&self, pos: &BlockPos) -> bool {
        self.get_brightness(LightLayer::SKY, pos) >= 15
    }
}
