//! net/minecraft/world/level/block/GrowingPlantBodyBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `growthDirection`, `shape` and `scheduleFluidTicks`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::growing_plant_block::{self, GrowingPlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct GrowingPlantBodyBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.GrowingPlantBodyBlock",
    &growing_plant_block::CLASS,
);

impl GrowingPlantBodyBlock {
    pub(crate) fn new_with(
        properties: Properties,
        shape: VoxelShape,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        GrowingPlantBlock::new_with(properties, shape, class, definition)
    }
}
