//! net/minecraft/world/level/block/BeetrootBlock.java
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

pub struct BeetrootBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BeetrootBlock",
    &crop_block::CLASS,
);

impl BeetrootBlock {
    pub const MAX_AGE: i32 = 3;
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_3;

    pub fn new(properties: Properties) -> BlockData {
        BeetrootBlock::new_with(
            properties,
            &CLASS,
            BeetrootBlock::create_block_state_definition,
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
            BeetrootBlock::AGE,
            BeetrootBlock::MAX_AGE,
        )
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BeetrootBlock::AGE]);
    }
}
