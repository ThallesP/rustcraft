//! net/minecraft/world/level/block/SlimeBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SlimeBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SlimeBlock",
    &half_transparent_block::CLASS,
);

impl SlimeBlock {
    pub fn new(properties: Properties) -> BlockData {
        SlimeBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }
}
