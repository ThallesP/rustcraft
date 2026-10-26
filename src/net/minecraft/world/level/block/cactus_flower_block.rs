//! net/minecraft/world/level/block/CactusFlowerBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::support_type::SupportType;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CactusFlowerBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(CactusFlowerBlock::may_place_on),
    get_shape: Some(CactusFlowerBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CactusFlowerBlock",
        &vegetation_block::CLASS,
    )
};

impl CactusFlowerBlock {
    pub fn new(properties: Properties) -> BlockData {
        CactusFlowerBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 12.0));

impl CactusFlowerBlock {
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

impl CactusFlowerBlock {
    /// `mayPlaceOn`: on what holds a cactus flower up, or a top sturdy at its center.
    fn may_place_on(
        _block: &BlockData,
        _state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let block_below = level.get_block_state(pos);
        block_below.is_tag(BlockTags::SUPPORT_OVERRIDE_CACTUS_FLOWER)
            || block_below.is_face_sturdy_for(level, pos, Direction::UP, SupportType::CENTER)
    }
}
