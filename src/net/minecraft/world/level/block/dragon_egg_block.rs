//! net/minecraft/world/level/block/DragonEggBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_block::{self, FallingBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct DragonEggBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DragonEggBlock",
    &falling_block::CLASS,
);

impl DragonEggBlock {
    pub fn new(properties: Properties) -> BlockData {
        DragonEggBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FallingBlock::new_with(properties, class, definition)
    }
}
