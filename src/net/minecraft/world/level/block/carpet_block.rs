//! net/minecraft/world/level/block/CarpetBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (a sixteenth) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CarpetBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CarpetBlock::get_shape),
    ..BlockClass::new("net.minecraft.world.level.block.CarpetBlock", &block::CLASS)
};

impl CarpetBlock {
    pub fn new(properties: Properties) -> BlockData {
        CarpetBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 1.0));

impl CarpetBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}
