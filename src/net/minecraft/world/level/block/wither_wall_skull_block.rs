//! net/minecraft/world/level/block/WitherWallSkullBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::skull_block::Types;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::wall_skull_block::{self, WallSkullBlock};

pub struct WitherWallSkullBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WitherWallSkullBlock",
    &wall_skull_block::CLASS,
);

impl WitherWallSkullBlock {
    pub fn new(properties: Properties) -> BlockData {
        WitherWallSkullBlock::new_with(
            properties,
            &CLASS,
            WallSkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        WallSkullBlock::new_with(Types::WITHER_SKELETON, properties, class, definition)
    }
}
