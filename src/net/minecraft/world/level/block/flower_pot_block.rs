//! net/minecraft/world/level/block/FlowerPotBlock.java
//!
//! Only what decides its blocks' states so far: which of them tick. Not ported yet: the rest,
//! including its `potted` field and `POTTED_BY_CONTENT`, which its constructor fills.
//!
//! Of its behaviour, its shape is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct FlowerPotBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(FlowerPotBlock::get_shape),
    is_randomly_ticking: Some(FlowerPotBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FlowerPotBlock",
        &block::CLASS,
    )
};

impl FlowerPotBlock {
    pub fn new(properties: Properties) -> BlockData {
        FlowerPotBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }

    /// The eyeblossoms in pots tick, to open and close.
    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.is(Blocks::POTTED_OPEN_EYEBLOSSOM) || state.is(Blocks::POTTED_CLOSED_EYEBLOSSOM)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, 0.0, 6.0));

impl FlowerPotBlock {
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
