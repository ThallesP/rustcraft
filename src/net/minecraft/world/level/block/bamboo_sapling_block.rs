//! net/minecraft/world/level/block/BambooSaplingBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (moved by its offset) is ported.

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

pub struct BambooSaplingBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BambooSaplingBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BambooSaplingBlock",
        &block::CLASS,
    )
};

impl BambooSaplingBlock {
    pub fn new(properties: Properties) -> BlockData {
        BambooSaplingBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(8.0, 0.0, 12.0));

impl BambooSaplingBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.r#move(&state.get_offset(pos))
    }
}
