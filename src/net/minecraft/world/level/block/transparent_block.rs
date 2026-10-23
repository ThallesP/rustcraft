//! net/minecraft/world/level/block/TransparentBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown`, its visual shape (nothing) and its shade brightness
//! are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct TransparentBlock;

pub static CLASS: BlockClass = BlockClass {
    get_visual_shape: Some(TransparentBlock::get_visual_shape),
    get_shade_brightness: Some(TransparentBlock::get_shade_brightness),
    propagates_skylight_down: Some(TransparentBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TransparentBlock",
        &half_transparent_block::CLASS,
    )
};

impl TransparentBlock {
    pub fn new(properties: Properties) -> BlockData {
        TransparentBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }
}

impl TransparentBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

impl TransparentBlock {
    fn get_visual_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
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
