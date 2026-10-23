//! net/minecraft/world/level/block/RodBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (along where it faces) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct RodBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(RodBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RodBlock",
        &directional_block::CLASS,
    )
};

impl RodBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DirectionalBlock::new_with(properties, class, definition)
    }
}

static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_all_axis(&Block::cube_xyz(4.0, 4.0, 16.0)));

impl RodBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES,
            state.get_value(DirectionalBlock::FACING).get_axis(),
        )
        .clone()
    }
}
