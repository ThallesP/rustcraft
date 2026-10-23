//! net/minecraft/world/level/block/SpeleothemBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `blockToGrowOn`.
//!
//! Of its behaviour, `getFluidState`, its shape (by its thickness, moved by its offset),
//! `isCollisionShapeFullBlock` (never) and its offset's reach (`getMaxHorizontalOffset`, its base's
//! inset) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::speleothem_thickness::SpeleothemThickness;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SpeleothemBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(SpeleothemBlock::get_shape),
    is_collision_shape_full_block: Some(SpeleothemBlock::is_collision_shape_full_block),
    get_max_horizontal_offset: Some(SpeleothemBlock::get_max_horizontal_offset),
    get_fluid_state: Some(SpeleothemBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SpeleothemBlock",
        &block::CLASS,
    )
};

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

impl SpeleothemBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE_TIP_MERGE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, 0.0, 16.0));
static SHAPE_TIP_UP: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, 0.0, 11.0));
/// `SHAPE_TIP_DOWN`.
pub static SHAPE_TIP_DOWN: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, 5.0, 16.0));
static SHAPE_FRUSTUM: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(8.0, 0.0, 16.0));
static SHAPE_MIDDLE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 16.0));
static SHAPE_BASE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 16.0));

impl SpeleothemBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let shape = match state.get_value(SpeleothemBlock::THICKNESS) {
            SpeleothemThickness::TIP_MERGE => &SHAPE_TIP_MERGE,
            SpeleothemThickness::TIP => {
                if state.get_value(SpeleothemBlock::TIP_DIRECTION) == Direction::DOWN {
                    &SHAPE_TIP_DOWN
                } else {
                    &SHAPE_TIP_UP
                }
            }
            SpeleothemThickness::FRUSTUM => &SHAPE_FRUSTUM,
            SpeleothemThickness::MIDDLE => &SHAPE_MIDDLE,
            SpeleothemThickness::BASE => &SHAPE_BASE,
        };
        shape.r#move(&state.get_offset(pos))
    }

    fn is_collision_shape_full_block(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> bool {
        false
    }

    /// `MAX_HORIZONTAL_OFFSET`: how far in its base is.
    fn get_max_horizontal_offset(_block: &BlockData) -> f32 {
        SHAPE_BASE.min(Axis::X) as f32
    }
}
