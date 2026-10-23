//! net/minecraft/client/color/block/BlockTintSource.java
//!
//! Not ported yet: `relevantProperties`, which `ModelGroupCollector` reads.

use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// The colour a model's tinted faces of one tint index are multiplied by.
pub trait BlockTintSource: Send + Sync {
    fn color(&self, state: BlockState) -> i32;

    fn color_in_world(
        &self,
        state: BlockState,
        _level: &dyn BlockAndTintGetter,
        _pos: &BlockPos,
    ) -> i32 {
        self.color(state)
    }

    fn color_as_terrain_particle(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        self.color_in_world(state, level, pos)
    }
}
