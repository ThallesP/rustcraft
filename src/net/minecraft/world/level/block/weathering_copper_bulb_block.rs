//! net/minecraft/world/level/block/WeatheringCopperBulbBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::copper_bulb_block::{self, CopperBulbBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperBulbBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperBulbBlock",
        &copper_bulb_block::CLASS,
    )
};

impl WeatheringCopperBulbBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperBulbBlock::new_with(
            properties,
            &CLASS,
            CopperBulbBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CopperBulbBlock::new_with(properties, class, definition)
    }
}
