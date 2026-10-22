//! net/minecraft/world/level/block/SculkVeinBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::multiface_block::MultifaceBlock;
use crate::net::minecraft::world::level::block::multiface_spreadeable_block::{
    self, MultifaceSpreadeableBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SculkVeinBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SculkVeinBlock",
    &multiface_spreadeable_block::CLASS,
);

impl SculkVeinBlock {
    pub fn new(properties: Properties) -> BlockData {
        SculkVeinBlock::new_with(
            properties,
            &CLASS,
            MultifaceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        MultifaceSpreadeableBlock::new_with(properties, class, definition)
    }
}
