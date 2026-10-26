//! net/minecraft/world/level/block/BeaconBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::beacon_block_entity::BeaconBlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct BeaconBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(BeaconBlock::get_ticker),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BeaconBlock",
        &base_entity_block::CLASS,
    )
};

impl BeaconBlock {
    pub fn new(properties: Properties) -> BlockData {
        BeaconBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl BeaconBlock {
    /// `getTicker`: `tick`, on both sides.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::BEACON,
            BeaconBlockEntity::tick,
        )
    }
}
