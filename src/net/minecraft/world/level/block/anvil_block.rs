//! net/minecraft/world/level/block/AnvilBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (its foot, waist and top along its axis) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_block::{self, FallingBlock};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct AnvilBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(AnvilBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AnvilBlock",
        &falling_block::CLASS,
    )
};

impl AnvilBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        AnvilBlock::new_with(
            properties,
            &CLASS,
            AnvilBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = FallingBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(AnvilBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[AnvilBlock::FACING]);
    }
}

static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal_axis(&Shapes::or_tail(
        &Block::column(12.0, 0.0, 4.0),
        &[
            Block::column_xz(8.0, 10.0, 4.0, 5.0),
            Block::column_xz(4.0, 8.0, 5.0, 10.0),
            Block::column_xz(10.0, 16.0, 10.0, 16.0),
        ],
    ))
});

impl AnvilBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(AnvilBlock::FACING).get_axis()).clone()
    }
}
