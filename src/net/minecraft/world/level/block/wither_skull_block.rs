//! net/minecraft/world/level/block/WitherSkullBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::skull_block::Types;
use crate::net::minecraft::world::level::block::skull_block::{self, SkullBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct WitherSkullBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WitherSkullBlock",
    &skull_block::CLASS,
);

impl WitherSkullBlock {
    pub fn new(properties: Properties) -> BlockData {
        WitherSkullBlock::new_with(
            properties,
            &CLASS,
            SkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SkullBlock::new_with(Types::WITHER_SKELETON, properties, class, definition)
    }
}
