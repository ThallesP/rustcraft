//! net/minecraft/world/level/block/AzaleaBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct AzaleaBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AzaleaBlock",
    &vegetation_block::CLASS,
);

impl AzaleaBlock {
    pub fn new(properties: Properties) -> BlockData {
        AzaleaBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}
