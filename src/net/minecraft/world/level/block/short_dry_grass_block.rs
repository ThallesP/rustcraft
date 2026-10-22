//! net/minecraft/world/level/block/ShortDryGrassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::dry_vegetation_block::{self, DryVegetationBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct ShortDryGrassBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ShortDryGrassBlock",
    &dry_vegetation_block::CLASS,
);

impl ShortDryGrassBlock {
    pub fn new(properties: Properties) -> BlockData {
        ShortDryGrassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DryVegetationBlock::new_with(properties, class, definition)
    }
}
