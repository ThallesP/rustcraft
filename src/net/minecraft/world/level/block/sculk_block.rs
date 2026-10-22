//! net/minecraft/world/level/block/SculkBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::drop_experience_block::{
    self, DropExperienceBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SculkBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SculkBlock",
    &drop_experience_block::CLASS,
);

impl SculkBlock {
    pub fn new(properties: Properties) -> BlockData {
        SculkBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DropExperienceBlock::new_with(properties, class, definition)
    }
}
