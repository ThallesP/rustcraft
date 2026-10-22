//! net/minecraft/world/level/block/DaylightDetectorBlock.java
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
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct DaylightDetectorBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DaylightDetectorBlock",
    &base_entity_block::CLASS,
);

impl DaylightDetectorBlock {
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;
    pub const INVERTED: &'static BooleanProperty = BlockStateProperties::INVERTED;

    pub fn new(properties: Properties) -> BlockData {
        DaylightDetectorBlock::new_with(
            properties,
            &CLASS,
            DaylightDetectorBlock::create_block_state_definition,
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
                .set_value(DaylightDetectorBlock::POWER, 0)
                .set_value(DaylightDetectorBlock::INVERTED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DaylightDetectorBlock::POWER,
            DaylightDetectorBlock::INVERTED,
        ]);
    }
}
