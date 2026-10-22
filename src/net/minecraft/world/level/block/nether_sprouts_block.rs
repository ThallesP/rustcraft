//! net/minecraft/world/level/block/NetherSproutsBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct NetherSproutsBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.NetherSproutsBlock",
    &vegetation_block::CLASS,
);

impl NetherSproutsBlock {
    pub fn new(properties: Properties) -> BlockData {
        NetherSproutsBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}
