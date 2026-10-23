//! net/minecraft/world/level/block/StairBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shapes: a
//! bottom half and a quarter on top (straight), an eighth (outer corners) or three quarters
//! (inner corners), turned to face each way, and upside down for the top half. Not ported yet: the
//! rest of its behaviour, and its constructor's `baseState`.

use std::sync::LazyLock;

use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::net::minecraft::core::block_pos::BlockPos;
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
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A stair's shapes facing north, then each way: `SHAPE_OUTER`, `SHAPE_STRAIGHT` and `SHAPE_INNER`
/// turned about the block's center, and the same upside down.
struct StairShapes {
    bottom_outer: Vec<(Direction, VoxelShape)>,
    bottom_straight: Vec<(Direction, VoxelShape)>,
    bottom_inner: Vec<(Direction, VoxelShape)>,
    top_outer: Vec<(Direction, VoxelShape)>,
    top_straight: Vec<(Direction, VoxelShape)>,
    top_inner: Vec<(Direction, VoxelShape)>,
}

static SHAPES: LazyLock<StairShapes> = LazyLock::new(|| {
    let shape_outer = Shapes::or(
        &Block::column(16.0, 0.0, 8.0),
        &Block::r#box(0.0, 8.0, 0.0, 8.0, 16.0, 8.0),
    );
    let shape_straight = Shapes::or(
        &shape_outer,
        &Shapes::rotate(&shape_outer, OctahedralGroup::BLOCK_ROT_Y_90),
    );
    let shape_inner = Shapes::or(
        &shape_straight,
        &Shapes::rotate(&shape_straight, OctahedralGroup::BLOCK_ROT_Y_90),
    );
    StairShapes {
        bottom_outer: Shapes::rotate_horizontal(&shape_outer),
        bottom_straight: Shapes::rotate_horizontal(&shape_straight),
        bottom_inner: Shapes::rotate_horizontal(&shape_inner),
        top_outer: Shapes::rotate_horizontal_from(&shape_outer, OctahedralGroup::INVERT_Y),
        top_straight: Shapes::rotate_horizontal_from(&shape_straight, OctahedralGroup::INVERT_Y),
        top_inner: Shapes::rotate_horizontal_from(&shape_inner, OctahedralGroup::INVERT_Y),
    }
});

pub struct StairBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(StairBlock::get_fluid_state),
    get_shape: Some(StairBlock::get_shape),
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
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let is_bottom = state.get_value(StairBlock::HALF) == Half::BOTTOM;
        let facing = state.get_value(StairBlock::FACING);
        let shapes = &*SHAPES;
        let map = match state.get_value(StairBlock::SHAPE) {
            StairsShape::STRAIGHT if is_bottom => &shapes.bottom_straight,
            StairsShape::STRAIGHT => &shapes.top_straight,
            StairsShape::OUTER_LEFT | StairsShape::OUTER_RIGHT if is_bottom => &shapes.bottom_outer,
            StairsShape::OUTER_LEFT | StairsShape::OUTER_RIGHT => &shapes.top_outer,
            StairsShape::INNER_RIGHT | StairsShape::INNER_LEFT if is_bottom => &shapes.bottom_inner,
            StairsShape::INNER_RIGHT | StairsShape::INNER_LEFT => &shapes.top_inner,
        };
        let direction = match state.get_value(StairBlock::SHAPE) {
            StairsShape::STRAIGHT | StairsShape::OUTER_LEFT | StairsShape::INNER_RIGHT => facing,
            StairsShape::INNER_LEFT => facing.get_counter_clock_wise(),
            StairsShape::OUTER_RIGHT => facing.get_clock_wise(),
        };
        Shapes::get(map, direction).clone()
    }

    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
