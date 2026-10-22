//! net/minecraft/world/level/block/WeatheringCopperTrapDoorBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `type` and `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::trap_door_block::{self, TrapDoorBlock};
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperTrapDoorBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperTrapDoorBlock",
        &trap_door_block::CLASS,
    )
};

impl WeatheringCopperTrapDoorBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperTrapDoorBlock::new_with(
            properties,
            &CLASS,
            TrapDoorBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        TrapDoorBlock::new_with(properties, class, definition)
    }
}
