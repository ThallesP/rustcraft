//! net/minecraft/world/level/block/SulfurSpikeBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `blockToGrowOn`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::speleothem_block::{self, SpeleothemBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SulfurSpikeBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SulfurSpikeBlock",
    &speleothem_block::CLASS,
);

impl SulfurSpikeBlock {
    pub fn new(properties: Properties) -> BlockData {
        SulfurSpikeBlock::new_with(
            properties,
            &CLASS,
            SpeleothemBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SpeleothemBlock::new_with(properties, class, definition)
    }
}
