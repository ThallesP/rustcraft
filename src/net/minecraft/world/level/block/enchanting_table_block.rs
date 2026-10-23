//! net/minecraft/world/level/block/EnchantingTableBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shape are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct EnchantingTableBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(EnchantingTableBlock::get_shape),
    use_shape_for_light_occlusion: Some(EnchantingTableBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EnchantingTableBlock",
        &base_entity_block::CLASS,
    )
};

impl EnchantingTableBlock {
    pub fn new(properties: Properties) -> BlockData {
        EnchantingTableBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl EnchantingTableBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 12.0));

impl EnchantingTableBlock {
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
