//! net/minecraft/world/level/block/CoralBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `deadBlock`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CoralBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.CoralBlock", &block::CLASS);

impl CoralBlock {
    pub fn new(properties: Properties) -> BlockData {
        CoralBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
