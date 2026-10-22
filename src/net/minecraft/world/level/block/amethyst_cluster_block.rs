//! net/minecraft/world/level/block/AmethystClusterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `height` and `width`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::amethyst_block::{self, AmethystBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct AmethystClusterBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AmethystClusterBlock",
    &amethyst_block::CLASS,
);

impl AmethystClusterBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::FACING;

    pub fn new(properties: Properties) -> BlockData {
        AmethystClusterBlock::new_with(
            properties,
            &CLASS,
            AmethystClusterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AmethystBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(AmethystClusterBlock::WATERLOGGED, false)
                .set_value(AmethystClusterBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            AmethystClusterBlock::WATERLOGGED,
            AmethystClusterBlock::FACING,
        ]);
    }
}
