//! net/minecraft/world/level/block/SnowLayerBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SnowLayerBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SnowLayerBlock",
    &block::CLASS,
);

impl SnowLayerBlock {
    pub const LAYERS: &'static IntegerProperty = BlockStateProperties::LAYERS;

    pub fn new(properties: Properties) -> BlockData {
        SnowLayerBlock::new_with(
            properties,
            &CLASS,
            SnowLayerBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(SnowLayerBlock::LAYERS, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SnowLayerBlock::LAYERS]);
    }
}
