//! net/minecraft/world/level/block/SandBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `dustColor`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::colored_falling_block::{
    self, ColoredFallingBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SandBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SandBlock",
    &colored_falling_block::CLASS,
);

impl SandBlock {
    pub fn new(properties: Properties) -> BlockData {
        SandBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        ColoredFallingBlock::new_with(properties, class, definition)
    }
}
