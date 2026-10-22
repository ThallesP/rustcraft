//! net/minecraft/world/level/block/TintedGlassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::transparent_block::{self, TransparentBlock};

pub struct TintedGlassBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TintedGlassBlock",
    &transparent_block::CLASS,
);

impl TintedGlassBlock {
    pub fn new(properties: Properties) -> BlockData {
        TintedGlassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        TransparentBlock::new_with(properties, class, definition)
    }
}
