//! net/minecraft/world/level/block/WeatheringCopperGolemStatueBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::copper_golem_statue_block::{
    self, CopperGolemStatueBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperGolemStatueBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperGolemStatueBlock",
        &copper_golem_statue_block::CLASS,
    )
};

impl WeatheringCopperGolemStatueBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperGolemStatueBlock::new_with(
            properties,
            &CLASS,
            CopperGolemStatueBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CopperGolemStatueBlock::new_with(properties, class, definition)
    }
}
