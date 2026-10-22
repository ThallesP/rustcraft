//! net/minecraft/world/level/block/LoomBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;

pub struct LoomBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.LoomBlock",
    &horizontal_directional_block::CLASS,
);

impl LoomBlock {
    pub fn new(properties: Properties) -> BlockData {
        LoomBlock::new_with(properties, &CLASS, LoomBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HorizontalDirectionalBlock::new_with(properties, class, definition)
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HorizontalDirectionalBlock::FACING]);
    }
}
