//! net/minecraft/world/level/block/CoralFanBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `deadBlock`.

use crate::net::minecraft::world::level::block::base_coral_fan_block::{self, BaseCoralFanBlock};
use crate::net::minecraft::world::level::block::base_coral_plant_type_block::BaseCoralPlantTypeBlock;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CoralFanBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CoralFanBlock",
    &base_coral_fan_block::CLASS,
);

impl CoralFanBlock {
    pub fn new(properties: Properties) -> BlockData {
        CoralFanBlock::new_with(
            properties,
            &CLASS,
            BaseCoralPlantTypeBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseCoralFanBlock::new_with(properties, class, definition)
    }
}
