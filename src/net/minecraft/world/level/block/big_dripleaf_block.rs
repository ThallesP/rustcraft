//! net/minecraft/world/level/block/BigDripleafBlock.java
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
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::tilt::Tilt;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BigDripleafBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BigDripleafBlock",
    &horizontal_directional_block::CLASS,
);

impl BigDripleafBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const TILT: &'static EnumProperty<Tilt> = BlockStateProperties::TILT;

    pub fn new(properties: Properties) -> BlockData {
        BigDripleafBlock::new_with(
            properties,
            &CLASS,
            BigDripleafBlock::create_block_state_definition,
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
                .set_value(BigDripleafBlock::WATERLOGGED, false)
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(BigDripleafBlock::TILT, Tilt::NONE),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BigDripleafBlock::WATERLOGGED,
            HorizontalDirectionalBlock::FACING,
            BigDripleafBlock::TILT,
        ]);
    }
}
