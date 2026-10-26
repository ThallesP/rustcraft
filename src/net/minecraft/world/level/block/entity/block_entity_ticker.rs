//! net/minecraft/world/level/block/entity/BlockEntityTicker.java
//!
//! What a block entity's class does each tick, Java's method reference (`ChestBlockEntity::
//! lidAnimateTick`), handed the level, the block entity's position and the state there. The block
//! entity is the level's at that position, which the ticker borrows from it
//! (`LevelTicking::block_entity`) for as long as it changes it, where Java hands the object over.
//! Not ported yet: `andThen`, which only the potent sulfur's tickers chain.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// `BlockEntityTicker.tick(level, pos, state, entity)`.
pub type BlockEntityTicker = fn(&mut LevelTicking<'_>, &BlockPos, BlockState);
