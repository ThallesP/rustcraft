//! net/minecraft/world/level/block/BaseFireBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `fireDamage`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct BaseFireBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BaseFireBlock",
    &block::CLASS,
);

impl BaseFireBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
