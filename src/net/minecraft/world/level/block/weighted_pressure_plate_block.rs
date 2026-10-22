//! net/minecraft/world/level/block/WeightedPressurePlateBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `maxWeight` and `type`.

use crate::net::minecraft::world::level::block::base_pressure_plate_block::{
    self, BasePressurePlateBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct WeightedPressurePlateBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WeightedPressurePlateBlock",
    &base_pressure_plate_block::CLASS,
);

impl WeightedPressurePlateBlock {
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;

    pub fn new(properties: Properties) -> BlockData {
        WeightedPressurePlateBlock::new_with(
            properties,
            &CLASS,
            WeightedPressurePlateBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BasePressurePlateBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WeightedPressurePlateBlock::POWER, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WeightedPressurePlateBlock::POWER]);
    }
}
