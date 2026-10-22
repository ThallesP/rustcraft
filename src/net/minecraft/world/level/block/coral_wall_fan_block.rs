//! net/minecraft/world/level/block/CoralWallFanBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `deadBlock`.

use crate::net::minecraft::world::level::block::base_coral_wall_fan_block::{
    self, BaseCoralWallFanBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CoralWallFanBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CoralWallFanBlock",
    &base_coral_wall_fan_block::CLASS,
);

impl CoralWallFanBlock {
    pub fn new(properties: Properties) -> BlockData {
        CoralWallFanBlock::new_with(
            properties,
            &CLASS,
            BaseCoralWallFanBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseCoralWallFanBlock::new_with(properties, class, definition)
    }
}
