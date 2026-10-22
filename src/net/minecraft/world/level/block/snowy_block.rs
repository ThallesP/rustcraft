//! net/minecraft/world/level/block/SnowyBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SnowyBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.SnowyBlock", &block::CLASS);

impl SnowyBlock {
    pub const SNOWY: &'static BooleanProperty = BlockStateProperties::SNOWY;

    pub fn new(properties: Properties) -> BlockData {
        SnowyBlock::new_with(
            properties,
            &CLASS,
            SnowyBlock::create_block_state_definition,
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
                .set_value(SnowyBlock::SNOWY, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SnowyBlock::SNOWY]);
    }
}
