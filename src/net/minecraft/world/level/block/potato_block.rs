//! net/minecraft/world/level/block/PotatoBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (taller as it grows) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::crop_block::CropBlockFields;
use crate::net::minecraft::world::level::block::crop_block::{self, CropBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PotatoBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PotatoBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PotatoBlock",
        &crop_block::CLASS,
    )
};

impl PotatoBlock {
    pub fn new(properties: Properties) -> BlockData {
        PotatoBlock::new_with(properties, &CLASS, CropBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        CropBlock::new_with(
            properties,
            class,
            definition,
            CropBlock::AGE,
            CropBlock::MAX_AGE,
        )
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> =
    LazyLock::new(|| Block::boxes(7, |age| Block::column(16.0, 0.0, f64::from(2 + age))));

impl PotatoBlock {
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(block.fields::<CropBlockFields>().age_property) as usize].clone()
    }
}
