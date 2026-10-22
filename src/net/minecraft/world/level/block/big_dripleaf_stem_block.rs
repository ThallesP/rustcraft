//! net/minecraft/world/level/block/BigDripleafStemBlock.java
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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BigDripleafStemBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BigDripleafStemBlock",
    &horizontal_directional_block::CLASS,
);

impl BigDripleafStemBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        BigDripleafStemBlock::new_with(
            properties,
            &CLASS,
            BigDripleafStemBlock::create_block_state_definition,
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
                .set_value(BigDripleafStemBlock::WATERLOGGED, false)
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BigDripleafStemBlock::WATERLOGGED,
            HorizontalDirectionalBlock::FACING,
        ]);
    }
}
