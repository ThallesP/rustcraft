//! net/minecraft/world/level/block/PlayerHeadBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::skull_block::{self, SkullBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct PlayerHeadBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.PlayerHeadBlock",
    &skull_block::CLASS,
);

impl PlayerHeadBlock {
    pub fn new(properties: Properties) -> BlockData {
        PlayerHeadBlock::new_with(
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
        SkullBlock::new_with(properties, class, definition)
    }
}
