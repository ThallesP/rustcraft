//! net/minecraft/world/level/block/EyeblossomBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `type`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::flower_block::{self, FlowerBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct EyeblossomBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.EyeblossomBlock",
    &flower_block::CLASS,
);

impl EyeblossomBlock {
    pub fn new(properties: Properties) -> BlockData {
        EyeblossomBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FlowerBlock::new_with(properties, class, definition)
    }
}
