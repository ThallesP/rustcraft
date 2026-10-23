//! net/minecraft/world/level/block/CeilingHangingSignBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, its shapes (wide along one of the four ways it can face, narrow at any other
//! rotation, and the same to hold things up) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sign_block::{self, SignBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::rotation_segment::RotationSegment;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CeilingHangingSignBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CeilingHangingSignBlock::get_shape),
    get_block_support_shape: Some(CeilingHangingSignBlock::get_block_support_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CeilingHangingSignBlock",
        &sign_block::CLASS,
    )
};

impl CeilingHangingSignBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;
    pub const ATTACHED: &'static BooleanProperty = BlockStateProperties::ATTACHED;

    pub fn new(properties: Properties) -> BlockData {
        CeilingHangingSignBlock::new_with(
            properties,
            &CLASS,
            CeilingHangingSignBlock::create_block_state_definition,
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
                .set_value(CeilingHangingSignBlock::ROTATION, 8)
                .set_value(CeilingHangingSignBlock::ATTACHED, false)
                .set_value(SignBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CeilingHangingSignBlock::ROTATION,
            CeilingHangingSignBlock::ATTACHED,
            SignBlock::WATERLOGGED,
        ]);
    }
}

static SHAPE_DEFAULT: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 16.0));
/// `SHAPES`: a `HashMap` by rotation, which only `getOrDefault` reads.
static SHAPES: LazyLock<Vec<(i32, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal(&Block::column_xz(14.0, 2.0, 0.0, 10.0))
        .into_iter()
        .map(|(direction, shape)| (RotationSegment::convert_to_segment(direction), shape))
        .collect()
});

impl CeilingHangingSignBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let rotation = state.get_value(CeilingHangingSignBlock::ROTATION);
        SHAPES
            .iter()
            .find(|(segment, _)| *segment == rotation)
            .map_or_else(|| SHAPE_DEFAULT.clone(), |(_, shape)| shape.clone())
    }

    fn get_block_support_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        state.get_shape_in(level, pos, &collision_context::empty())
    }
}
