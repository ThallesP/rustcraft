//! net/minecraft/world/level/block/WeatheringCopperSlabBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::slab_block::{self, SlabBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperSlabBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperSlabBlock",
        &slab_block::CLASS,
    )
};

impl WeatheringCopperSlabBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperSlabBlock::new_with(
            properties,
            &CLASS,
            SlabBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SlabBlock::new_with(properties, class, definition)
    }
}
