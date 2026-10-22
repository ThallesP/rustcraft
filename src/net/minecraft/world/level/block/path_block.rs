//! net/minecraft/world/level/block/PathBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `baseBlock`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct PathBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.PathBlock", &block::CLASS);

impl PathBlock {
    pub fn new(properties: Properties) -> BlockData {
        PathBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
