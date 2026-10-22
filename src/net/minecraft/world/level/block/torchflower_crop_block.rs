//! net/minecraft/world/level/block/TorchflowerCropBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::crop_block::{self, CropBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;

pub struct TorchflowerCropBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TorchflowerCropBlock",
    &crop_block::CLASS,
);

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
