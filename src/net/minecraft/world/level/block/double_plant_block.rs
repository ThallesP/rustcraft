//! net/minecraft/world/level/block/DoublePlantBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct DoublePlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DoublePlantBlock",
    &vegetation_block::CLASS,
);

impl DoublePlantBlock {
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> =
        BlockStateProperties::DOUBLE_BLOCK_HALF;

    pub fn new(properties: Properties) -> BlockData {
        DoublePlantBlock::new_with(
            properties,
            &CLASS,
            DoublePlantBlock::create_block_state_definition,
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
                .set_value(DoublePlantBlock::HALF, DoubleBlockHalf::LOWER),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DoublePlantBlock::HALF]);
    }
}
