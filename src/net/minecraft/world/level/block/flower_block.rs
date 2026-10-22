//! net/minecraft/world/level/block/FlowerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `suspiciousStewEffect` and `effectSeconds`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct FlowerBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FlowerBlock",
    &vegetation_block::CLASS,
);

impl FlowerBlock {
    pub fn new(properties: Properties) -> BlockData {
        FlowerBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}
