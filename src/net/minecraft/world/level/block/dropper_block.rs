//! net/minecraft/world/level/block/DropperBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::dispenser_block::{self, DispenserBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct DropperBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DropperBlock",
    &dispenser_block::CLASS,
);

impl DropperBlock {
    pub fn new(properties: Properties) -> BlockData {
        DropperBlock::new_with(
            properties,
            &CLASS,
            DispenserBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DispenserBlock::new_with(properties, class, definition)
    }
}
