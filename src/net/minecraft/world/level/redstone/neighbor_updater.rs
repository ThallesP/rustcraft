//! net/minecraft/world/level/redstone/NeighborUpdater.java
//!
//! Only `executeShapeUpdate`, what a shape update does: the client's level only sends those
//! (`Level.neighborChanged` and `updateNeighborsAt` do nothing there), so `executeUpdate`,
//! `UPDATE_ORDER` and `updateNeighborsAtExceptFromFacing` aren't ported. Java turns what a shape
//! update throws into a crash report; here a panic ends the client as it is.

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct NeighborUpdater;

impl NeighborUpdater {
    /// `executeShapeUpdate(level, direction, pos, neighborPos, neighborState, updateFlags,
    /// updateLimit)`: the block at `pos` updates its shape for its neighbour that way, and takes
    /// the new state, unless it's redstone wire and the flags skip wire
    /// (`UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE`).
    #[allow(clippy::too_many_arguments)]
    pub fn execute_shape_update(
        level: &mut ClientLevel,
        direction: Direction,
        pos: &BlockPos,
        neighbor_pos: &BlockPos,
        neighbor_state: BlockState,
        update_flags: i32,
        update_limit: i32,
    ) {
        let current_state = level.get_block_state(pos);
        if (update_flags & 128) == 0 || !current_state.is(Blocks::REDSTONE_WIRE) {
            let new_state = current_state.update_shape(
                &*level,
                pos,
                direction,
                neighbor_pos,
                neighbor_state,
                &mut *level.get_random(),
            );
            Block::update_or_destroy(
                current_state,
                new_state,
                level,
                pos,
                update_flags,
                update_limit,
            );
        }
    }
}
