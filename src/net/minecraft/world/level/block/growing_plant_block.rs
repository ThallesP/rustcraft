//! net/minecraft/world/level/block/GrowingPlantBlock.java
//!
//! Only the class so far: its blocks' states are its parent's, and its constructor's `shape`, which
//! is its shape. Not ported yet: its behaviour, and its constructor's `growthDirection` and
//! `scheduleFluidTicks`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct GrowingPlantBlock {
    shape: VoxelShape,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(GrowingPlantBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.GrowingPlantBlock",
        &block::CLASS,
    )
};

impl GrowingPlantBlock {
    pub(crate) fn new_with(
        properties: Properties,
        shape: VoxelShape,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.set_fields(GrowingPlantBlock { shape });
        block
    }

    fn get_shape(
        block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<GrowingPlantBlock>().shape.clone()
    }
}
