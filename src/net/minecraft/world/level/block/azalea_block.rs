//! net/minecraft/world/level/block/AzaleaBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (a bush on a stem) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct AzaleaBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(AzaleaBlock::may_place_on),
    get_shape: Some(AzaleaBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AzaleaBlock",
        &vegetation_block::CLASS,
    )
};

impl AzaleaBlock {
    pub fn new(properties: Properties) -> BlockData {
        AzaleaBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(16.0, 8.0, 16.0),
        &Block::column(4.0, 0.0, 8.0),
    )
});

impl AzaleaBlock {
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

impl AzaleaBlock {
    /// `mayPlaceOn`: on what supports azalea.
    fn may_place_on(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> bool {
        state.is_tag(BlockTags::SUPPORTS_AZALEA)
    }
}
