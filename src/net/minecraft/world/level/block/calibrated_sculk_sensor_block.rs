//! net/minecraft/world/level/block/CalibratedSculkSensorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sculk_sensor_block::{self, SculkSensorBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CalibratedSculkSensorBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CalibratedSculkSensorBlock",
    &sculk_sensor_block::CLASS,
);

impl CalibratedSculkSensorBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        CalibratedSculkSensorBlock::new_with(
            properties,
            &CLASS,
            CalibratedSculkSensorBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SculkSensorBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(CalibratedSculkSensorBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        SculkSensorBlock::create_block_state_definition(builder);
        builder.add(&[CalibratedSculkSensorBlock::FACING]);
    }
}
