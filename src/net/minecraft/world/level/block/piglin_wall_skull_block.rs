//! net/minecraft/world/level/block/PiglinWallSkullBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (against the wall it faces from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::skull_block::Types;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::wall_skull_block::{self, WallSkullBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PiglinWallSkullBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PiglinWallSkullBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PiglinWallSkullBlock",
        &wall_skull_block::CLASS,
    )
};

impl PiglinWallSkullBlock {
    pub fn new(properties: Properties) -> BlockData {
        PiglinWallSkullBlock::new_with(
            properties,
            &CLASS,
            WallSkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        WallSkullBlock::new_with(Types::PIGLIN, properties, class, definition)
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z_sized(10.0, 8.0, 8.0, 16.0)));

impl PiglinWallSkullBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(WallSkullBlock::FACING)).clone()
    }
}
