//! net/minecraft/world/level/block/PlayerWallHeadBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::wall_skull_block::{self, WallSkullBlock};

pub struct PlayerWallHeadBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.PlayerWallHeadBlock",
    &wall_skull_block::CLASS,
);

impl PlayerWallHeadBlock {
    pub fn new(properties: Properties) -> BlockData {
        PlayerWallHeadBlock::new_with(
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
        WallSkullBlock::new_with(properties, class, definition)
    }
}
