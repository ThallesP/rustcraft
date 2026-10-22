//! net/minecraft/world/level/block/SmithingTableBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::crafting_table_block::{self, CraftingTableBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SmithingTableBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SmithingTableBlock",
    &crafting_table_block::CLASS,
);

impl SmithingTableBlock {
    pub fn new(properties: Properties) -> BlockData {
        SmithingTableBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CraftingTableBlock::new_with(properties, class, definition)
    }
}
