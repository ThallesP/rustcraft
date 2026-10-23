//! net/minecraft/world/level/block/piston/PistonBaseBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `isSticky`.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shape (the whole block, or its base when
//! it's extended) are ported.

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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PistonBaseBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PistonBaseBlock::get_shape),
    use_shape_for_light_occlusion: Some(PistonBaseBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.piston.PistonBaseBlock",
        &directional_block::CLASS,
    )
};

impl PistonBaseBlock {
    pub const EXTENDED: &'static BooleanProperty = BlockStateProperties::EXTENDED;

    pub fn new(properties: Properties) -> BlockData {
        PistonBaseBlock::new_with(
            properties,
            &CLASS,
            PistonBaseBlock::create_block_state_definition,
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
                .set_value(PistonBaseBlock::EXTENDED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING, PistonBaseBlock::EXTENDED]);
    }
}

impl PistonBaseBlock {
    /// `useShapeForLightOcclusion`: when it's extended.
    fn use_shape_for_light_occlusion(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(PistonBaseBlock::EXTENDED)
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_all(&Block::box_z_square(16.0, 4.0, 16.0)));

impl PistonBaseBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(PistonBaseBlock::EXTENDED) {
            Shapes::get(&SHAPES, state.get_value(DirectionalBlock::FACING)).clone()
        } else {
            Shapes::block()
        }
    }
}
