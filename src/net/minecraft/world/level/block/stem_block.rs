//! net/minecraft/world/level/block/StemBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `fruit`, `attachedStem`, `seed`, `stemSupportBlocks`
//! and `fruitSupportBlocks`.
//!
//! Of its behaviour, its shape (taller as it grows) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct StemBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(StemBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.StemBlock",
        &vegetation_block::CLASS,
    )
};

impl StemBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_7;

    pub fn new(properties: Properties) -> BlockData {
        StemBlock::new_with(properties, &CLASS, StemBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(StemBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[StemBlock::AGE]);
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> =
    LazyLock::new(|| Block::boxes(7, |age| Block::column(2.0, 0.0, f64::from(2 + age * 2))));

impl StemBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(StemBlock::AGE) as usize].clone()
    }
}
