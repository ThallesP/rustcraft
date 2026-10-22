//! net/minecraft/world/level/block/SpeleothemBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `blockToGrowOn`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::speleothem_thickness::SpeleothemThickness;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SpeleothemBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SpeleothemBlock",
    &block::CLASS,
);

impl SpeleothemBlock {
    pub const TIP_DIRECTION: &'static EnumProperty<Direction> =
        BlockStateProperties::VERTICAL_DIRECTION;
    pub const THICKNESS: &'static EnumProperty<SpeleothemThickness> =
        BlockStateProperties::SPELEOTHEM_THICKNESS;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(SpeleothemBlock::TIP_DIRECTION, Direction::UP)
                .set_value(SpeleothemBlock::THICKNESS, SpeleothemThickness::TIP)
                .set_value(SpeleothemBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            SpeleothemBlock::TIP_DIRECTION,
            SpeleothemBlock::THICKNESS,
            SpeleothemBlock::WATERLOGGED,
        ]);
    }
}
