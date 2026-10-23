//! net/minecraft/world/level/block/AbstractCauldronBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `interactions`.
//!
//! Of its behaviour, its shapes (a block with its legs' gaps and its inside taken out, and its
//! inside to interact with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct AbstractCauldronBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(AbstractCauldronBlock::get_shape),
    get_interaction_shape: Some(AbstractCauldronBlock::get_interaction_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AbstractCauldronBlock",
        &block::CLASS,
    )
};

impl AbstractCauldronBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE_INSIDE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 4.0, 16.0));

/// `SHAPE`: the block, less the gaps between its legs and its inside.
pub static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::join(
        &Shapes::block(),
        &Shapes::or_tail(
            &Block::column_xz(16.0, 8.0, 0.0, 3.0),
            &[
                Block::column_xz(8.0, 16.0, 0.0, 3.0),
                Block::column(12.0, 0.0, 3.0),
                SHAPE_INSIDE.clone(),
            ],
        ),
        BooleanOp::ONLY_FIRST,
    )
});

impl AbstractCauldronBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_interaction_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        SHAPE_INSIDE.clone()
    }
}
