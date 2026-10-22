//! net/minecraft/world/level/block/WoolCarpetBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `color`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::carpet_block::{self, CarpetBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct WoolCarpetBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WoolCarpetBlock",
    &carpet_block::CLASS,
);

impl WoolCarpetBlock {
    pub fn new(properties: Properties) -> BlockData {
        WoolCarpetBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CarpetBlock::new_with(properties, class, definition)
    }
}
