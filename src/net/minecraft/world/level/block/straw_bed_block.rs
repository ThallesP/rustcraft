//! net/minecraft/world/level/block/StrawBedBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::abstract_bed_block::{self, AbstractBedBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct StrawBedBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.StrawBedBlock",
    &abstract_bed_block::CLASS,
);

impl StrawBedBlock {
    pub fn new(properties: Properties) -> BlockData {
        StrawBedBlock::new_with(
            properties,
            &CLASS,
            AbstractBedBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractBedBlock::new_with(properties, class, definition)
    }
}
