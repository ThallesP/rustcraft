//! net/minecraft/world/level/block/WeatheringLightningRodBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::lightning_rod_block::{self, LightningRodBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringLightningRodBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringLightningRodBlock",
        &lightning_rod_block::CLASS,
    )
};

impl WeatheringLightningRodBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringLightningRodBlock::new_with(
            properties,
            &CLASS,
            LightningRodBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        LightningRodBlock::new_with(properties, class, definition)
    }
}
