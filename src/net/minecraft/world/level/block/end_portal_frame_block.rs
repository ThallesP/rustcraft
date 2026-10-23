//! net/minecraft/world/level/block/EndPortalFrameBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shape (with the eye on top, when it has
//! one) are ported.

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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct EndPortalFrameBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(EndPortalFrameBlock::get_shape),
    use_shape_for_light_occlusion: Some(EndPortalFrameBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndPortalFrameBlock",
        &block::CLASS,
    )
};

impl EndPortalFrameBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HAS_EYE: &'static BooleanProperty = BlockStateProperties::EYE;

    pub fn new(properties: Properties) -> BlockData {
        EndPortalFrameBlock::new_with(
            properties,
            &CLASS,
            EndPortalFrameBlock::create_block_state_definition,
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
                .set_value(EndPortalFrameBlock::FACING, Direction::NORTH)
                .set_value(EndPortalFrameBlock::HAS_EYE, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[EndPortalFrameBlock::FACING, EndPortalFrameBlock::HAS_EYE]);
    }
}

impl EndPortalFrameBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_EMPTY: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 13.0));
static SHAPE_FULL: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::or(&SHAPE_EMPTY, &Block::column(8.0, 13.0, 16.0)));

impl EndPortalFrameBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(EndPortalFrameBlock::HAS_EYE) {
            SHAPE_FULL.clone()
        } else {
            SHAPE_EMPTY.clone()
        }
    }
}
