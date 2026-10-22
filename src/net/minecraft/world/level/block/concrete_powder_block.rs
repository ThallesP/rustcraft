//! net/minecraft/world/level/block/ConcretePowderBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `concrete`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_block::{self, FallingBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct ConcretePowderBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ConcretePowderBlock",
    &falling_block::CLASS,
);

impl ConcretePowderBlock {
    pub fn new(properties: Properties) -> BlockData {
        ConcretePowderBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FallingBlock::new_with(properties, class, definition)
    }
}
