//! net/minecraft/world/level/block/TorchflowerCropBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
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
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TorchflowerCropBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(TorchflowerCropBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TorchflowerCropBlock",
        &crop_block::CLASS,
    )
};

impl TorchflowerCropBlock {
    pub const MAX_AGE: i32 = 1;
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_1;

    pub fn new(properties: Properties) -> BlockData {
        TorchflowerCropBlock::new_with(
            properties,
            &CLASS,
            TorchflowerCropBlock::create_block_state_definition,
        )
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
            TorchflowerCropBlock::AGE,
            TorchflowerCropBlock::get_max_age(),
        )
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[TorchflowerCropBlock::AGE]);
    }

    /// `getMaxAge()`: 2, past `AGE`'s last value (`MAX_AGE` is 1), so a torchflower crop never
    /// counts as grown, and always ticks.
    pub fn get_max_age() -> i32 {
        2
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> =
    LazyLock::new(|| Block::boxes(1, |age| Block::column(6.0, 0.0, f64::from(6 + age * 4))));

impl TorchflowerCropBlock {
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
