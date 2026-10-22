//! net/minecraft/world/level/block/ColoredFallingBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `dustColor`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_block::{self, FallingBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct ColoredFallingBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ColoredFallingBlock",
    &falling_block::CLASS,
);

impl ColoredFallingBlock {
    pub fn new(properties: Properties) -> BlockData {
        ColoredFallingBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FallingBlock::new_with(properties, class, definition)
    }
}
