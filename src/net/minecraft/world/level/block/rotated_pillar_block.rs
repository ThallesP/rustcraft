//! net/minecraft/world/level/block/RotatedPillarBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RotatedPillarBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RotatedPillarBlock",
    &block::CLASS,
);

impl RotatedPillarBlock {
    pub const AXIS: &'static EnumProperty<Axis> = BlockStateProperties::AXIS;

    pub fn new(properties: Properties) -> BlockData {
        RotatedPillarBlock::new_with(
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
        let mut block = Block::new_with(properties, class, definition);
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
