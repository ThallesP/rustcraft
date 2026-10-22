//! net/minecraft/world/level/block/RepeaterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::diode_block::{self, DiodeBlock};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RepeaterBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RepeaterBlock",
    &diode_block::CLASS,
);

impl RepeaterBlock {
    pub const LOCKED: &'static BooleanProperty = BlockStateProperties::LOCKED;
    pub const DELAY: &'static IntegerProperty = BlockStateProperties::DELAY;

    pub fn new(properties: Properties) -> BlockData {
        RepeaterBlock::new_with(
            properties,
            &CLASS,
            RepeaterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DiodeBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(RepeaterBlock::DELAY, 1)
                .set_value(RepeaterBlock::LOCKED, false)
                .set_value(DiodeBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            RepeaterBlock::DELAY,
            RepeaterBlock::LOCKED,
            DiodeBlock::POWERED,
        ]);
    }
}
