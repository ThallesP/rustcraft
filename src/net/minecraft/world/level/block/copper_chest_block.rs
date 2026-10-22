//! net/minecraft/world/level/block/CopperChestBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `weatherState`, `openSound` and `closeSound`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::chest_block::{self, ChestBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CopperChestBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CopperChestBlock",
    &chest_block::CLASS,
);

impl CopperChestBlock {
    pub fn new(properties: Properties) -> BlockData {
        CopperChestBlock::new_with(
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
