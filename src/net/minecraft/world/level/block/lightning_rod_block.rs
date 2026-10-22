//! net/minecraft/world/level/block/LightningRodBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::rod_block::{self, RodBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct LightningRodBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.LightningRodBlock",
    &rod_block::CLASS,
);

impl LightningRodBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        LightningRodBlock::new_with(
            properties,
            &CLASS,
            LightningRodBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = RodBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::UP)
                .set_value(LightningRodBlock::WATERLOGGED, false)
                .set_value(LightningRodBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DirectionalBlock::FACING,
            LightningRodBlock::POWERED,
            LightningRodBlock::WATERLOGGED,
        ]);
    }
}
