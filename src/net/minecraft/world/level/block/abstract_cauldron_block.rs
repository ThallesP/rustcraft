//! net/minecraft/world/level/block/AbstractCauldronBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `interactions`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct AbstractCauldronBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AbstractCauldronBlock",
    &block::CLASS,
);

impl AbstractCauldronBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
