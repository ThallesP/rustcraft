//! net/minecraft/world/level/block/SpawnerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SpawnerBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SpawnerBlock",
    &base_entity_block::CLASS,
);

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
