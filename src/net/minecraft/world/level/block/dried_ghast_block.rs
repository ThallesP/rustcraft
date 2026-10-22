//! net/minecraft/world/level/block/DriedGhastBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct DriedGhastBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DriedGhastBlock",
    &horizontal_directional_block::CLASS,
);

impl DriedGhastBlock {
    pub const HYDRATION_LEVEL: &'static IntegerProperty =
        BlockStateProperties::DRIED_GHAST_HYDRATION_LEVELS;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        DriedGhastBlock::new_with(
            properties,
            &CLASS,
            DriedGhastBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(DriedGhastBlock::HYDRATION_LEVEL, 0)
                .set_value(DriedGhastBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            DriedGhastBlock::HYDRATION_LEVEL,
            DriedGhastBlock::WATERLOGGED,
        ]);
    }
}
