//! net/minecraft/world/level/block/StructureVoidBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (a cube 6 sixteenths across), its render shape (invisible) and its
//! shade brightness are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct StructureVoidBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(StructureVoidBlock::get_shape),
    get_render_shape: Some(StructureVoidBlock::get_render_shape),
    get_shade_brightness: Some(StructureVoidBlock::get_shade_brightness),
    ..BlockClass::new(
        "net.minecraft.world.level.block.StructureVoidBlock",
        &block::CLASS,
    )
};

impl StructureVoidBlock {
    pub fn new(properties: Properties) -> BlockData {
        StructureVoidBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::cube(6.0));

impl StructureVoidBlock {
    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }

    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_shade_brightness(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> f32 {
        1.0
    }
}
