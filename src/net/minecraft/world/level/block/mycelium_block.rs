//! net/minecraft/world/level/block/MyceliumBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::snowy_block::SnowyBlock;
use crate::net::minecraft::world::level::block::spreading_snowy_block::{
    self, SpreadingSnowyBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct MyceliumBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.MyceliumBlock",
    &spreading_snowy_block::CLASS,
);

impl MyceliumBlock {
    pub fn new(properties: Properties) -> BlockData {
        MyceliumBlock::new_with(
            properties,
            &CLASS,
            SnowyBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SpreadingSnowyBlock::new_with(properties, class, definition)
    }
}
