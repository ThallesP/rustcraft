//! net/minecraft/world/level/block/WeatheringCopperDoorBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `type` and `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::door_block::{self, DoorBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperDoorBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperDoorBlock",
        &door_block::CLASS,
    )
};

impl WeatheringCopperDoorBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperDoorBlock::new_with(
            properties,
            &CLASS,
            DoorBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DoorBlock::new_with(properties, class, definition)
    }
}
