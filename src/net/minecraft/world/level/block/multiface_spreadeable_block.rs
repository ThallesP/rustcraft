//! net/minecraft/world/level/block/MultifaceSpreadeableBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::multiface_block::{self, MultifaceBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct MultifaceSpreadeableBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.MultifaceSpreadeableBlock",
    &multiface_block::CLASS,
);

impl MultifaceSpreadeableBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        MultifaceBlock::new_with(properties, class, definition)
    }
}
