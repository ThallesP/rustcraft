//! net/minecraft/world/level/block/DropExperienceBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `xpRange`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct DropExperienceBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DropExperienceBlock",
    &block::CLASS,
);

impl DropExperienceBlock {
    pub fn new(properties: Properties) -> BlockData {
        DropExperienceBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
