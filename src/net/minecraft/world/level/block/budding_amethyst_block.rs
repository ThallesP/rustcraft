//! net/minecraft/world/level/block/BuddingAmethystBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::amethyst_block::{self, AmethystBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct BuddingAmethystBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BuddingAmethystBlock",
    &amethyst_block::CLASS,
);

impl BuddingAmethystBlock {
    pub fn new(properties: Properties) -> BlockData {
        BuddingAmethystBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AmethystBlock::new_with(properties, class, definition)
    }
}
