//! net/minecraft/world/level/block/NetherFungusBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `feature`, `requiredBlock` and `supportBlocks`.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct NetherFungusBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.NetherFungusBlock",
    &vegetation_block::CLASS,
);

impl NetherFungusBlock {
    pub fn new(properties: Properties) -> BlockData {
        NetherFungusBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}
