//! net/minecraft/world/level/block/BaseCoralWallFanBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_coral_fan_block::{self, BaseCoralFanBlock};
use crate::net::minecraft::world::level::block::base_coral_plant_type_block::BaseCoralPlantTypeBlock;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BaseCoralWallFanBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BaseCoralWallFanBlock",
    &base_coral_fan_block::CLASS,
);

impl BaseCoralWallFanBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        BaseCoralWallFanBlock::new_with(
            properties,
            &CLASS,
            BaseCoralWallFanBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseCoralFanBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BaseCoralWallFanBlock::FACING, Direction::NORTH)
                .set_value(BaseCoralPlantTypeBlock::WATERLOGGED, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BaseCoralWallFanBlock::FACING,
            BaseCoralPlantTypeBlock::WATERLOGGED,
        ]);
    }
}
