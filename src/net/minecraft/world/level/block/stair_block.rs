//! net/minecraft/world/level/block/StairBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `baseState`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::half::Half;
use crate::net::minecraft::world::level::block::state::properties::stairs_shape::StairsShape;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct StairBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(StairBlock::get_fluid_state),
    ..BlockClass::new("net.minecraft.world.level.block.StairBlock", &block::CLASS)
};

impl StairBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HALF: &'static EnumProperty<Half> = BlockStateProperties::HALF;
    pub const SHAPE: &'static EnumProperty<StairsShape> = BlockStateProperties::STAIRS_SHAPE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        StairBlock::new_with(
            properties,
            &CLASS,
            StairBlock::create_block_state_definition,
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
                .get_state_definition()
                .any()
                .set_value(StairBlock::FACING, Direction::NORTH)
                .set_value(StairBlock::HALF, Half::BOTTOM)
                .set_value(StairBlock::SHAPE, StairsShape::STRAIGHT)
                .set_value(StairBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            StairBlock::FACING,
            StairBlock::HALF,
            StairBlock::SHAPE,
            StairBlock::WATERLOGGED,
        ]);
    }
}

impl StairBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
