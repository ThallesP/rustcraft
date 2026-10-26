//! net/minecraft/world/level/block/LecternBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shapes (its base and post, with the sloped
//! top toward where it faces) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct LecternBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(LecternBlock::is_signal_source),
    get_shape: Some(LecternBlock::get_shape),
    get_collision_shape: Some(LecternBlock::get_collision_shape),
    get_occlusion_shape: Some(LecternBlock::get_occlusion_shape),
    use_shape_for_light_occlusion: Some(LecternBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LecternBlock",
        &base_entity_block::CLASS,
    )
};

impl LecternBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const HAS_BOOK: &'static BooleanProperty = BlockStateProperties::HAS_BOOK;

    pub fn new(properties: Properties) -> BlockData {
        LecternBlock::new_with(
            properties,
            &CLASS,
            LecternBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LecternBlock::FACING, Direction::NORTH)
                .set_value(LecternBlock::POWERED, false)
                .set_value(LecternBlock::HAS_BOOK, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            LecternBlock::FACING,
            LecternBlock::POWERED,
            LecternBlock::HAS_BOOK,
        ]);
    }
}

impl LecternBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_COLLISION: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(16.0, 0.0, 2.0),
        &Block::column(8.0, 2.0, 14.0),
    )
});
static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal(&Shapes::or_tail(
        &Block::box_z(16.0, 10.0, 14.0, 1.0, 5.333333),
        &[
            Block::box_z(16.0, 12.0, 16.0, 5.333333, 9.666667),
            Block::box_z(16.0, 14.0, 18.0, 9.666667, 14.0),
            SHAPE_COLLISION.clone(),
        ],
    ))
});

impl LecternBlock {
    fn get_occlusion_shape(_block: &BlockData, _state: BlockState) -> VoxelShape {
        SHAPE_COLLISION.clone()
    }

    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE_COLLISION.clone()
    }

    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(HorizontalDirectionalBlock::FACING)).clone()
    }
}

impl LecternBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
