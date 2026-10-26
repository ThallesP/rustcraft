//! net/minecraft/world/level/block/SeaPickleBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape (by how many pickles) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SeaPickleBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(SeaPickleBlock::may_place_on),
    get_shape: Some(SeaPickleBlock::get_shape),
    get_fluid_state: Some(SeaPickleBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SeaPickleBlock",
        &vegetation_block::CLASS,
    )
};

impl SeaPickleBlock {
    pub const PICKLES: &'static IntegerProperty = BlockStateProperties::PICKLES;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SeaPickleBlock::new_with(
            properties,
            &CLASS,
            SeaPickleBlock::create_block_state_definition,
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
                .set_value(SeaPickleBlock::PICKLES, 1)
                .set_value(SeaPickleBlock::WATERLOGGED, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SeaPickleBlock::PICKLES, SeaPickleBlock::WATERLOGGED]);
    }

    /// Out of water.
    pub fn is_dead(state: BlockState) -> bool {
        !state.get_value(SeaPickleBlock::WATERLOGGED)
    }
}

impl SeaPickleBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE_ONE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(4.0, 0.0, 6.0));
static SHAPE_TWO: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 6.0));
static SHAPE_THREE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 6.0));
static SHAPE_FOUR: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 7.0));

impl SeaPickleBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        match state.get_value(SeaPickleBlock::PICKLES) {
            2 => SHAPE_TWO.clone(),
            3 => SHAPE_THREE.clone(),
            4 => SHAPE_FOUR.clone(),
            _ => SHAPE_ONE.clone(),
        }
    }
}

impl SeaPickleBlock {
    /// `mayPlaceOn`: on anything with a top face to collide with, or a sturdy top.
    fn may_place_on(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        !state
            .get_collision_shape(level, pos)
            .get_face_shape(Direction::UP)
            .is_empty()
            || state.is_face_sturdy(level, pos, Direction::UP)
    }
}
