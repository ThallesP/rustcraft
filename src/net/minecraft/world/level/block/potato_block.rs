//! net/minecraft/world/level/block/PotatoBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::crop_block::{self, CropBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct PotatoBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.PotatoBlock",
    &crop_block::CLASS,
);

impl PotatoBlock {
    pub fn new(properties: Properties) -> BlockData {
        PotatoBlock::new_with(properties, &CLASS, CropBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CropBlock::new_with(
            properties,
            class,
            definition,
            CropBlock::AGE,
            CropBlock::MAX_AGE,
        )
    }
}
