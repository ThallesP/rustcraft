//! net/minecraft/world/level/block/BaseEntityBlock.java
//!
//! The class, a block event handed to its block entity (`triggerEvent`), and `createTickerHelper`.
//! Not ported yet: the rest of its behaviour.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct BaseEntityBlock;

pub static CLASS: BlockClass = BlockClass {
    entity_block: true,
    trigger_event: Some(BaseEntityBlock::trigger_event),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BaseEntityBlock",
        &block::CLASS,
    )
};

impl BaseEntityBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl BaseEntityBlock {
    /// `triggerEvent`: the block entity there's, made if the block makes one (`getBlockEntity`,
    /// `IMMEDIATE`); none handles nothing.
    fn trigger_event(
        _block: &BlockData,
        _state: BlockState,
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        b0: i32,
        b1: i32,
    ) -> bool {
        block_entity_trigger_event(level, pos, b0, b1)
    }
}

/// `level.getBlockEntity(pos)`'s `triggerEvent(b0, b1)`, false without one: what
/// `BaseEntityBlock.triggerEvent` and `ComparatorBlock.triggerEvent` do after `super`'s.
pub fn block_entity_trigger_event(
    level: &mut LevelTicking<'_>,
    pos: &BlockPos,
    b0: i32,
    b1: i32,
) -> bool {
    if level.level.get_block_entity(pos).is_none() {
        return false;
    }
    let mut block_entity = level.block_entity(pos);
    block_entity.trigger_event(level, b0, b1)
}

/// `createTickerHelper`: the ticker, if the block entity's type is the one it's for.
pub fn create_ticker_helper(
    actual: BlockEntityType,
    expected: BlockEntityType,
    ticker: BlockEntityTicker,
) -> Option<BlockEntityTicker> {
    (expected == actual).then_some(ticker)
}
