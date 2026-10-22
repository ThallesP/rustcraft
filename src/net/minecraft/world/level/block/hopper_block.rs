//! net/minecraft/world/level/block/HopperBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct HopperBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.HopperBlock",
    &base_entity_block::CLASS,
);

impl HopperBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::FACING_HOPPER;
    pub const ENABLED: &'static BooleanProperty = BlockStateProperties::ENABLED;

    pub fn new(properties: Properties) -> BlockData {
        HopperBlock::new_with(
            properties,
            &CLASS,
            HopperBlock::create_block_state_definition,
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
                .set_value(HopperBlock::FACING, Direction::DOWN)
                .set_value(HopperBlock::ENABLED, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HopperBlock::FACING, HopperBlock::ENABLED]);
    }
}
