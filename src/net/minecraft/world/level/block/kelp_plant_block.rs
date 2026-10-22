//! net/minecraft/world/level/block/KelpPlantBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::growing_plant_body_block::{
    self, GrowingPlantBodyBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct KelpPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.KelpPlantBlock",
    &growing_plant_body_block::CLASS,
);

impl KelpPlantBlock {
    pub fn new(properties: Properties) -> BlockData {
        KelpPlantBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        GrowingPlantBodyBlock::new_with(properties, class, definition)
    }
}
