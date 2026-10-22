//! net/minecraft/world/level/block/RodBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct RodBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RodBlock",
    &directional_block::CLASS,
);

impl RodBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DirectionalBlock::new_with(properties, class, definition)
    }
}
