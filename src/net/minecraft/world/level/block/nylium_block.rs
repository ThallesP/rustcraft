//! net/minecraft/world/level/block/NyliumBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct NyliumBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.NyliumBlock", &block::CLASS);

impl NyliumBlock {
    pub fn new(properties: Properties) -> BlockData {
        NyliumBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
