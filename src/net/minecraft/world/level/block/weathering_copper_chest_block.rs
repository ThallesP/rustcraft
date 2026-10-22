//! net/minecraft/world/level/block/WeatheringCopperChestBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`, `openSound` and `closeSound`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::chest_block::ChestBlock;
use crate::net::minecraft::world::level::block::copper_chest_block::{self, CopperChestBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperChestBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperChestBlock",
        &copper_chest_block::CLASS,
    )
};

impl WeatheringCopperChestBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperChestBlock::new_with(
            properties,
            &CLASS,
            ChestBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CopperChestBlock::new_with(properties, class, definition)
    }
}
