//! net/minecraft/world/level/block/DryVegetationBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct DryVegetationBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DryVegetationBlock",
    &vegetation_block::CLASS,
);

impl DryVegetationBlock {
    pub fn new(properties: Properties) -> BlockData {
        DryVegetationBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}
