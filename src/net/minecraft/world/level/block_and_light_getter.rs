//! net/minecraft/world/level/BlockAndLightGetter.java
//!
//! Not ported yet: `getLightEngine`, `getRawBrightness` and `canSeeSky`, which read the level's
//! light engine; the client's keeps the server's light as its section states allow, which comes
//! with it. `getBrightness` is the renderer's way in until then: Java's default asks the light
//! engine's layer, which each level answers here.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::light_layer::LightLayer;

/// Blocks and their light.
pub trait BlockAndLightGetter: BlockGetter {
    /// The light of the layer at the block, 0 to 15.
    fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32;
}
