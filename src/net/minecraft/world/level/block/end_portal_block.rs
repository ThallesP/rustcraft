//! net/minecraft/world/level/block/EndPortalBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (a slab from 6 to 12 sixteenths up) and its render shape (invisible:
//! its block entity draws it) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct EndPortalBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(EndPortalBlock::get_shape),
    get_render_shape: Some(EndPortalBlock::get_render_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndPortalBlock",
        &base_entity_block::CLASS,
    )
};

impl EndPortalBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndPortalBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 6.0, 12.0));

impl EndPortalBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}
