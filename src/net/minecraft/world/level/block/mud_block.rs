//! net/minecraft/world/level/block/MudBlock.java
//!
//! Its shapes: a collision shape two sixteenths short of a block, like soul sand's, and the whole
//! block as its support. Not ported yet: the rest of its behaviour (visual and shade, and
//! pathfinding).

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 14.0));

pub struct MudBlock;

pub static CLASS: BlockClass = BlockClass {
    get_visual_shape: Some(MudBlock::get_visual_shape),
    get_shade_brightness: Some(MudBlock::get_shade_brightness),
    get_collision_shape: Some(MudBlock::get_collision_shape),
    get_block_support_shape: Some(MudBlock::get_block_support_shape),
    ..BlockClass::new("net.minecraft.world.level.block.MudBlock", &block::CLASS)
};

impl MudBlock {
    pub fn new(properties: Properties) -> BlockData {
        MudBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl MudBlock {
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_block_support_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::block()
    }
}

impl MudBlock {
    fn get_visual_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::block()
    }

    fn get_shade_brightness(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> f32 {
        0.2
    }
}
