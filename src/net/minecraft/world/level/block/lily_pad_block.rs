//! net/minecraft/world/level/block/LilyPadBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct LilyPadBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(LilyPadBlock::may_place_on),
    get_shape: Some(LilyPadBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LilyPadBlock",
        &vegetation_block::CLASS,
    )
};

impl LilyPadBlock {
    pub fn new(properties: Properties) -> BlockData {
        LilyPadBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 1.5));

impl LilyPadBlock {
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

impl LilyPadBlock {
    /// `mayPlaceOn`: on a fluid or block that holds lily pads, with no fluid above it.
    fn may_place_on(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let fluid_state = level.get_fluid_state(pos);
        let fluid_above = level.get_fluid_state(&pos.above());
        (fluid_state.is_tag(FluidTags::SUPPORTS_LILY_PAD)
            || state.is_tag(BlockTags::SUPPORTS_LILY_PAD))
            && fluid_above.get_type() == Fluids::EMPTY
    }
}
