//! net/minecraft/world/level/block/FlowerBedBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `shapeHeight`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct FlowerBedBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FlowerBedBlock",
    &vegetation_block::CLASS,
);

impl FlowerBedBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;
    pub const AMOUNT: &'static IntegerProperty = BlockStateProperties::FLOWER_AMOUNT;

    pub fn new(properties: Properties) -> BlockData {
        FlowerBedBlock::new_with(
            properties,
            &CLASS,
            FlowerBedBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FlowerBedBlock::FACING, Direction::NORTH)
                .set_value(FlowerBedBlock::AMOUNT, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[FlowerBedBlock::FACING, FlowerBedBlock::AMOUNT]);
    }
}
