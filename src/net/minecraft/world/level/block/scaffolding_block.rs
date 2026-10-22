//! net/minecraft/world/level/block/ScaffoldingBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ScaffoldingBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ScaffoldingBlock",
    &block::CLASS,
);

impl ScaffoldingBlock {
    pub const DISTANCE: &'static IntegerProperty = BlockStateProperties::STABILITY_DISTANCE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const BOTTOM: &'static BooleanProperty = BlockStateProperties::BOTTOM;

    pub fn new(properties: Properties) -> BlockData {
        ScaffoldingBlock::new_with(
            properties,
            &CLASS,
            ScaffoldingBlock::create_block_state_definition,
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
                .set_value(ScaffoldingBlock::DISTANCE, 7)
                .set_value(ScaffoldingBlock::WATERLOGGED, false)
                .set_value(ScaffoldingBlock::BOTTOM, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            ScaffoldingBlock::DISTANCE,
            ScaffoldingBlock::WATERLOGGED,
            ScaffoldingBlock::BOTTOM,
        ]);
    }
}
