//! net/minecraft/world/level/block/piston/PistonHeadBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shape (the platform and its arm, shorter
//! while it moves) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PistonHeadBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PistonHeadBlock::get_shape),
    use_shape_for_light_occlusion: Some(PistonHeadBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.piston.PistonHeadBlock",
        &directional_block::CLASS,
    )
};

impl PistonHeadBlock {
    pub const TYPE: &'static EnumProperty<PistonType> = BlockStateProperties::PISTON_TYPE;
    pub const SHORT: &'static BooleanProperty = BlockStateProperties::SHORT;

    pub fn new(properties: Properties) -> BlockData {
        PistonHeadBlock::new_with(
            properties,
            &CLASS,
            PistonHeadBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::NORTH)
                .set_value(PistonHeadBlock::TYPE, PistonType::DEFAULT)
                .set_value(PistonHeadBlock::SHORT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DirectionalBlock::FACING,
            PistonHeadBlock::TYPE,
            PistonHeadBlock::SHORT,
        ]);
    }
}

impl PistonHeadBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_PLATFORM: LazyLock<VoxelShape> = LazyLock::new(|| Block::box_z_square(16.0, 0.0, 4.0));
static SHAPES_SHORT: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_all(&Shapes::or(
        &SHAPE_PLATFORM,
        &Block::box_z_square(4.0, 4.0, 16.0),
    ))
});
static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_all(&Shapes::or(
        &SHAPE_PLATFORM,
        &Block::box_z_square(4.0, 4.0, 20.0),
    ))
});

impl PistonHeadBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let shapes = if state.get_value(PistonHeadBlock::SHORT) {
            &SHAPES_SHORT
        } else {
            &SHAPES
        };
        Shapes::get(shapes, state.get_value(DirectionalBlock::FACING)).clone()
    }
}
