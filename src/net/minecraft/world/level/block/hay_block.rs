//! net/minecraft/world/level/block/HayBlock.java
//!
//! Only what decides its blocks' states so far: its default state. Not ported yet: its behaviour.

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::rotated_pillar_block::{self, RotatedPillarBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct HayBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.HayBlock",
    &rotated_pillar_block::CLASS,
);

impl HayBlock {
    pub fn new(properties: Properties) -> BlockData {
        HayBlock::new_with(
            properties,
            &CLASS,
            RotatedPillarBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = RotatedPillarBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(RotatedPillarBlock::AXIS, Axis::Y),
        );
        block
    }
}
