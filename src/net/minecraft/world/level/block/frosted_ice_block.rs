//! net/minecraft/world/level/block/FrostedIceBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::ice_block::{self, IceBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct FrostedIceBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FrostedIceBlock",
    &ice_block::CLASS,
);

impl FrostedIceBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_3;

    pub fn new(properties: Properties) -> BlockData {
        FrostedIceBlock::new_with(
            properties,
            &CLASS,
            FrostedIceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = IceBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FrostedIceBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[FrostedIceBlock::AGE]);
    }
}
