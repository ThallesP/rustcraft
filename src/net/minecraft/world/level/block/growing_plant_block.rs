//! net/minecraft/world/level/block/GrowingPlantBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `growthDirection`, `shape` and `scheduleFluidTicks`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct GrowingPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.GrowingPlantBlock",
    &block::CLASS,
);

impl GrowingPlantBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
