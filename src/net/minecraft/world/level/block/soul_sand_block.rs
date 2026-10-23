//! net/minecraft/world/level/block/SoulSandBlock.java
//!
//! Its collision shape, two sixteenths short of a block, which sinks what stands on it. Not
//! ported yet: the rest of its behaviour (its support, visual and shade, and pathfinding).

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 14.0));

pub struct SoulSandBlock;

pub static CLASS: BlockClass = BlockClass {
    get_collision_shape: Some(SoulSandBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SoulSandBlock",
        &block::CLASS,
    )
};

impl SoulSandBlock {
    pub fn new(properties: Properties) -> BlockData {
        SoulSandBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl SoulSandBlock {
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}
