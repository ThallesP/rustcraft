//! net/minecraft/world/level/BlockGetter.java
//!
//! What collisions, the renderer and physics read blocks through: a level, one of its chunks, or
//! a copy of its sections. Not ported yet: `getBlockEntity` (block entities), `getBlockStates(AABB)`
//! and the ray casts (`clip`, `isBlockInLine`, `clipWithInteractionOverride`). Java's extends
//! `LevelHeightAccessor`, which is a trait of its own here that the levels and chunks implement,
//! so that what only reads blocks (the shapes' `EmptyBlockGetter`, the collisions' tests) needs
//! no height.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

/// Something blocks can be read from: a level, or one of its chunks.
pub trait BlockGetter {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState;

    fn get_light_emission(&self, pos: &BlockPos) -> i32 {
        self.get_block_state(pos).get_light_emission()
    }
}
