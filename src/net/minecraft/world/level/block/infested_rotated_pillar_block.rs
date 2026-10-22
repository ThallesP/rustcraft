//! net/minecraft/world/level/block/InfestedRotatedPillarBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `hostBlock`.

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::infested_block::{self, InfestedBlock};
use crate::net::minecraft::world::level::block::rotated_pillar_block::RotatedPillarBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct InfestedRotatedPillarBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.InfestedRotatedPillarBlock",
    &infested_block::CLASS,
);

impl InfestedRotatedPillarBlock {
    pub fn new(host_block: &BlockData, properties: Properties) -> BlockData {
        InfestedRotatedPillarBlock::new_with(
            host_block,
            properties,
            &CLASS,
            InfestedRotatedPillarBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        host_block: &BlockData,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = InfestedBlock::new_with(host_block, properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(RotatedPillarBlock::AXIS, Axis::Y),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RotatedPillarBlock::AXIS]);
    }
}
