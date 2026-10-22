//! net/minecraft/world/level/block/LeafLitterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::segmentable_block::SegmentableBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct LeafLitterBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.LeafLitterBlock",
    &vegetation_block::CLASS,
);

impl LeafLitterBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        LeafLitterBlock::new_with(
            properties,
            &CLASS,
            LeafLitterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LeafLitterBlock::FACING, Direction::NORTH)
                .set_value(LeafLitterBlock::get_segment_amount_property(), 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            LeafLitterBlock::FACING,
            LeafLitterBlock::get_segment_amount_property(),
        ]);
    }

    /// `SegmentableBlock`'s, which it doesn't override.
    pub fn get_segment_amount_property() -> &'static IntegerProperty {
        SegmentableBlock::get_segment_amount_property()
    }
}
