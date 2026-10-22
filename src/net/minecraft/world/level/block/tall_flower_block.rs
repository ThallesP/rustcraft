//! net/minecraft/world/level/block/TallFlowerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TallFlowerBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TallFlowerBlock",
    &double_plant_block::CLASS,
);

impl TallFlowerBlock {
    pub fn new(properties: Properties) -> BlockData {
        TallFlowerBlock::new_with(
            properties,
            &CLASS,
            DoublePlantBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DoublePlantBlock::new_with(properties, class, definition)
    }
}
