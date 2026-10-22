//! net/minecraft/world/level/block/SculkCatalystBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SculkCatalystBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SculkCatalystBlock",
    &base_entity_block::CLASS,
);

impl SculkCatalystBlock {
    pub const PULSE: &'static BooleanProperty = BlockStateProperties::BLOOM;

    pub fn new(properties: Properties) -> BlockData {
        SculkCatalystBlock::new_with(
            properties,
            &CLASS,
            SculkCatalystBlock::create_block_state_definition,
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
                .set_value(SculkCatalystBlock::PULSE, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SculkCatalystBlock::PULSE]);
    }
}
