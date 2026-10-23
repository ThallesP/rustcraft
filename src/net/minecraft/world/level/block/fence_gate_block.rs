//! net/minecraft/world/level/block/FenceGateBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, its shapes (lower in a wall; a fence's height to collide with and to hold
//! things up while it's closed, nothing while it's open; its posts to occlude) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct FenceGateBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(FenceGateBlock::get_shape),
    get_block_support_shape: Some(FenceGateBlock::get_block_support_shape),
    get_collision_shape: Some(FenceGateBlock::get_collision_shape),
    get_occlusion_shape: Some(FenceGateBlock::get_occlusion_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FenceGateBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl FenceGateBlock {
    pub const OPEN: &'static BooleanProperty = BlockStateProperties::OPEN;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const IN_WALL: &'static BooleanProperty = BlockStateProperties::IN_WALL;

    pub fn new(properties: Properties) -> BlockData {
        FenceGateBlock::new_with(
            properties,
            &CLASS,
            FenceGateBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FenceGateBlock::OPEN, false)
                .set_value(FenceGateBlock::POWERED, false)
                .set_value(FenceGateBlock::IN_WALL, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            FenceGateBlock::OPEN,
            FenceGateBlock::POWERED,
            FenceGateBlock::IN_WALL,
        ]);
    }
}

static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::cube_xyz(16.0, 16.0, 4.0)));
static SHAPES_WALL: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    SHAPES
        .iter()
        .map(|(axis, shape)| {
            (
                *axis,
                Shapes::join(
                    shape,
                    &Block::column(16.0, 13.0, 16.0),
                    BooleanOp::ONLY_FIRST,
                ),
            )
        })
        .collect()
});
static SHAPE_COLLISION: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::column_xz(16.0, 4.0, 0.0, 24.0)));
static SHAPE_SUPPORT: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::column_xz(16.0, 4.0, 5.0, 24.0)));
static SHAPE_OCCLUSION: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal_axis(&Shapes::or(
        &Block::r#box(0.0, 5.0, 7.0, 2.0, 16.0, 9.0),
        &Block::r#box(14.0, 5.0, 7.0, 16.0, 16.0, 9.0),
    ))
});
static SHAPE_OCCLUSION_WALL: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    SHAPE_OCCLUSION
        .iter()
        .map(|(axis, shape)| (*axis, shape.move_xyz(0.0, -0.1875, 0.0).optimize()))
        .collect()
});

impl FenceGateBlock {
    fn axis(state: BlockState) -> Axis {
        state
            .get_value(HorizontalDirectionalBlock::FACING)
            .get_axis()
    }

    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let shapes = if state.get_value(FenceGateBlock::IN_WALL) {
            &SHAPES_WALL
        } else {
            &SHAPES
        };
        Shapes::get(shapes, FenceGateBlock::axis(state)).clone()
    }

    fn get_block_support_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        if state.get_value(FenceGateBlock::OPEN) {
            Shapes::empty()
        } else {
            Shapes::get(&SHAPE_SUPPORT, FenceGateBlock::axis(state)).clone()
        }
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(FenceGateBlock::OPEN) {
            Shapes::empty()
        } else {
            Shapes::get(&SHAPE_COLLISION, FenceGateBlock::axis(state)).clone()
        }
    }

    fn get_occlusion_shape(_block: &BlockData, state: BlockState) -> VoxelShape {
        let shapes = if state.get_value(FenceGateBlock::IN_WALL) {
            &SHAPE_OCCLUSION_WALL
        } else {
            &SHAPE_OCCLUSION
        };
        Shapes::get(shapes, FenceGateBlock::axis(state)).clone()
    }
}
