//! net/minecraft/world/level/block/WitherRoseBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `mobEffect` and `effectSeconds`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::flower_block::{self, FlowerBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct WitherRoseBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WitherRoseBlock",
    &flower_block::CLASS,
);

impl WitherRoseBlock {
    pub fn new(properties: Properties) -> BlockData {
        WitherRoseBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FlowerBlock::new_with(properties, class, definition)
    }
}
