//! net/minecraft/world/level/block/WeatheringCopperGrateBlock.java
//!
//! Only what decides its blocks' states so far: which of its states tick. Not ported yet: its
//! behaviour, and its constructor's `weatherState`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::waterlogged_transparent_block::{
    self, WaterloggedTransparentBlock,
};
use crate::net::minecraft::world::level::block::weathering_copper::WeatheringCopper;

pub struct WeatheringCopperGrateBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(WeatheringCopper::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WeatheringCopperGrateBlock",
        &waterlogged_transparent_block::CLASS,
    )
};

impl WeatheringCopperGrateBlock {
    pub fn new(properties: Properties) -> BlockData {
        WeatheringCopperGrateBlock::new_with(
            properties,
            &CLASS,
            WaterloggedTransparentBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        WaterloggedTransparentBlock::new_with(properties, class, definition)
    }
}
