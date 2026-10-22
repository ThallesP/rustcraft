//! net/minecraft/world/level/block/WeatheringCopperStairBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState` and `baseState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::stair_block::{self, StairBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperStairBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperStairBlock",
        &stair_block::CLASS,
    )
};

impl WeatheringCopperStairBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperStairBlock::new_with(
            properties,
            &CLASS,
            StairBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        StairBlock::new_with(properties, class, definition)
    }
}
