//! net/minecraft/world/level/block/SpreadingSnowyBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `baseBlock`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::snowy_block::{self, SnowyBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct SpreadingSnowyBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SpreadingSnowyBlock",
    &snowy_block::CLASS,
);

impl SpreadingSnowyBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SnowyBlock::new_with(properties, class, definition)
    }
}
