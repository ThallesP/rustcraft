//! net/minecraft/world/level/block/BaseEntityBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct BaseEntityBlock;

pub static CLASS: BlockClass = BlockClass {
    entity_block: true,
    ..BlockClass::new(
        "net.minecraft.world.level.block.BaseEntityBlock",
        &block::CLASS,
    )
};

impl BaseEntityBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
