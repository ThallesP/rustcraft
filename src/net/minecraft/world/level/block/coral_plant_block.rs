//! net/minecraft/world/level/block/CoralPlantBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `deadBlock`.

use crate::net::minecraft::world::level::block::base_coral_plant_type_block::{
    self, BaseCoralPlantTypeBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CoralPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CoralPlantBlock",
    &base_coral_plant_type_block::CLASS,
);

impl CoralPlantBlock {
    pub fn new(properties: Properties) -> BlockData {
        CoralPlantBlock::new_with(
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
        BaseCoralPlantTypeBlock::new_with(properties, class, definition)
    }
}
