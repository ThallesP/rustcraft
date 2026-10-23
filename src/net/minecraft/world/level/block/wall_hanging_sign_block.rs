//! net/minecraft/world/level/block/WallHangingSignBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, its shapes (the sign under its plank, along its axis; the plank to collide
//! with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::sign_block::{self, SignBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct WallHangingSignBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(WallHangingSignBlock::get_shape),
    get_block_support_shape: Some(WallHangingSignBlock::get_block_support_shape),
    get_collision_shape: Some(WallHangingSignBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WallHangingSignBlock",
        &sign_block::CLASS,
    )
};

impl WallHangingSignBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        WallHangingSignBlock::new_with(
            properties,
            &CLASS,
            WallHangingSignBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SignBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WallHangingSignBlock::FACING, Direction::NORTH)
                .set_value(SignBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WallHangingSignBlock::FACING, SignBlock::WATERLOGGED]);
    }
}

static SHAPES_PLANK: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::column_xz(16.0, 4.0, 14.0, 16.0)));
static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal_axis(&Shapes::or(
        Shapes::get(&SHAPES_PLANK, Axis::Z),
        &Block::column_xz(14.0, 2.0, 0.0, 10.0),
    ))
});

impl WallHangingSignBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES,
            state.get_value(WallHangingSignBlock::FACING).get_axis(),
        )
        .clone()
    }

    fn get_block_support_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        state.get_shape_in(level, pos, &collision_context::empty())
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES_PLANK,
            state.get_value(WallHangingSignBlock::FACING).get_axis(),
        )
        .clone()
    }
}
