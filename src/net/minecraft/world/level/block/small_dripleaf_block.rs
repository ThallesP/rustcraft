//! net/minecraft/world/level/block/SmallDripleafBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SmallDripleafBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SmallDripleafBlock",
    &double_plant_block::CLASS,
);

impl SmallDripleafBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        SmallDripleafBlock::new_with(
            properties,
            &CLASS,
            SmallDripleafBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DoublePlantBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DoublePlantBlock::HALF, DoubleBlockHalf::LOWER)
                .set_value(SmallDripleafBlock::WATERLOGGED, false)
                .set_value(SmallDripleafBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DoublePlantBlock::HALF,
            SmallDripleafBlock::WATERLOGGED,
            SmallDripleafBlock::FACING,
        ]);
    }
}
