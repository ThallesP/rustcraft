//! net/minecraft/world/level/block/WeatheringCopperBarsBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::iron_bars_block::{self, IronBarsBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperBarsBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperBarsBlock",
        &iron_bars_block::CLASS,
    )
};

impl WeatheringCopperBarsBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperBarsBlock::new_with(
            properties,
            &CLASS,
            IronBarsBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        IronBarsBlock::new_with(properties, class, definition)
    }
}
