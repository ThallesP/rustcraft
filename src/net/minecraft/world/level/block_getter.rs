//! net/minecraft/world/level/BlockGetter.java
//!
//! Only `getBlockState`, what collisions read. The rest (block entities, fluid states, `clip`,
//! the light and height accessors it extends) comes with the world track's `ClientLevel`, which
//! implements it.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// Something blocks can be read from: a level, or one of its chunks.
pub trait BlockGetter {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
}
