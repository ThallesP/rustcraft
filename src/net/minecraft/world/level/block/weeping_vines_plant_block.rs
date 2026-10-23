//! net/minecraft/world/level/block/WeepingVinesPlantBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::growing_plant_body_block::{
    self, GrowingPlantBodyBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 16.0));

pub struct WeepingVinesPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WeepingVinesPlantBlock",
    &growing_plant_body_block::CLASS,
);

impl WeepingVinesPlantBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeepingVinesPlantBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        GrowingPlantBodyBlock::new_with(properties, SHAPE.clone(), class, definition)
    }
}
