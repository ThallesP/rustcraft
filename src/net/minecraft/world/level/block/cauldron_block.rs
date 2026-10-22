//! net/minecraft/world/level/block/CauldronBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::abstract_cauldron_block::{
    self, AbstractCauldronBlock,
};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct CauldronBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CauldronBlock",
    &abstract_cauldron_block::CLASS,
);

impl CauldronBlock {
    pub fn new(properties: Properties) -> BlockData {
        CauldronBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractCauldronBlock::new_with(properties, class, definition)
    }
}
