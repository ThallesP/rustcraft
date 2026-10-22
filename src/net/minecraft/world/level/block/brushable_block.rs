//! net/minecraft/world/level/block/BrushableBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `turnsInto`, `brushSound` and `brushCompletedSound`.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BrushableBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BrushableBlock",
    &base_entity_block::CLASS,
);

impl BrushableBlock {
    pub const DUSTED: &'static IntegerProperty = BlockStateProperties::DUSTED;

    pub fn new(properties: Properties) -> BlockData {
        BrushableBlock::new_with(
            properties,
            &CLASS,
            BrushableBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BrushableBlock::DUSTED, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BrushableBlock::DUSTED]);
    }
}
