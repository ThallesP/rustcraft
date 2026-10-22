//! net/minecraft/world/level/block/SoulFireBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::base_fire_block::{self, BaseFireBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SoulFireBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SoulFireBlock",
    &base_fire_block::CLASS,
);

impl SoulFireBlock {
    pub fn new(properties: Properties) -> BlockData {
        SoulFireBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseFireBlock::new_with(properties, class, definition)
    }
}
