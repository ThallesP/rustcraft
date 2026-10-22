//! net/minecraft/world/level/block/WeatheringCopperFullBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperFullBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperFullBlock",
        &block::CLASS,
    )
};

impl WeatheringCopperFullBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperFullBlock::new_with(
            properties,
            &CLASS,
            Block::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
