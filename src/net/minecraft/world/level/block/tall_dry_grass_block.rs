//! net/minecraft/world/level/block/TallDryGrassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::dry_vegetation_block::{self, DryVegetationBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TallDryGrassBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TallDryGrassBlock",
    &dry_vegetation_block::CLASS,
);

impl TallDryGrassBlock {
    pub fn new(properties: Properties) -> BlockData {
        TallDryGrassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DryVegetationBlock::new_with(properties, class, definition)
    }
}
