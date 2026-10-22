//! net/minecraft/world/level/block/ObserverBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ObserverBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ObserverBlock",
    &directional_block::CLASS,
);

impl ObserverBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        ObserverBlock::new_with(
            properties,
            &CLASS,
            ObserverBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::SOUTH)
                .set_value(ObserverBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING, ObserverBlock::POWERED]);
    }
}
