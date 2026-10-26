//! net/minecraft/world/level/block/SpawnerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::spawner_block_entity::SpawnerBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct SpawnerBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(SpawnerBlock::get_ticker),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SpawnerBlock",
        &base_entity_block::CLASS,
    )
};

impl SpawnerBlock {
    pub fn new(properties: Properties) -> BlockData {
        SpawnerBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl SpawnerBlock {
    /// `getTicker`: on the client, `clientTick`.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::MOB_SPAWNER,
            SpawnerBlockEntity::client_tick,
        )
    }
}
