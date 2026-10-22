//! net/minecraft/world/level/block/TransparentBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TransparentBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TransparentBlock",
    &half_transparent_block::CLASS,
);

impl TransparentBlock {
    pub fn new(properties: Properties) -> BlockData {
        TransparentBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }
}
