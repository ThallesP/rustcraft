//! net/minecraft/world/level/block/TrappedChestBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::chest_block::{self, ChestBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TrappedChestBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TrappedChestBlock",
    &chest_block::CLASS,
);

impl TrappedChestBlock {
    pub fn new(properties: Properties) -> BlockData {
        TrappedChestBlock::new_with(
            properties,
            &CLASS,
            ChestBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        ChestBlock::new_with(properties, class, definition)
    }
}
