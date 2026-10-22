//! net/minecraft/world/level/block/SmokerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::abstract_furnace_block::{
    self, AbstractFurnaceBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SmokerBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SmokerBlock",
    &abstract_furnace_block::CLASS,
);

impl SmokerBlock {
    pub fn new(properties: Properties) -> BlockData {
        SmokerBlock::new_with(
            properties,
            &CLASS,
            AbstractFurnaceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractFurnaceBlock::new_with(properties, class, definition)
    }
}
