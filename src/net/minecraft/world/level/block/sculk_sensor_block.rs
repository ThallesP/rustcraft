//! net/minecraft/world/level/block/SculkSensorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::sculk_sensor_phase::SculkSensorPhase;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SculkSensorBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SculkSensorBlock",
    &base_entity_block::CLASS,
);

impl SculkSensorBlock {
    pub const PHASE: &'static EnumProperty<SculkSensorPhase> =
        BlockStateProperties::SCULK_SENSOR_PHASE;
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SculkSensorBlock::new_with(
            properties,
            &CLASS,
            SculkSensorBlock::create_block_state_definition,
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
                .set_value(SculkSensorBlock::PHASE, SculkSensorPhase::INACTIVE)
                .set_value(SculkSensorBlock::POWER, 0)
                .set_value(SculkSensorBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            SculkSensorBlock::PHASE,
            SculkSensorBlock::POWER,
            SculkSensorBlock::WATERLOGGED,
        ]);
    }

    pub fn get_phase(state: BlockState) -> SculkSensorPhase {
        state.get_value(SculkSensorBlock::PHASE)
    }
}
