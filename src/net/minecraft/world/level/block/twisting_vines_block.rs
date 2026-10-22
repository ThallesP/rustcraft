//! net/minecraft/world/level/block/TwistingVinesBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::growing_plant_head_block::{
    self, GrowingPlantHeadBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TwistingVinesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TwistingVinesBlock",
    &growing_plant_head_block::CLASS,
);

impl TwistingVinesBlock {
    pub fn new(properties: Properties) -> BlockData {
        TwistingVinesBlock::new_with(
            properties,
            &CLASS,
            GrowingPlantHeadBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        GrowingPlantHeadBlock::new_with(properties, class, definition)
    }
}
