//! net/minecraft/world/level/block/FireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_fire_block::{self, BaseFireBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct FireBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FireBlock",
    &base_fire_block::CLASS,
);

impl FireBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_15;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const UP: &'static BooleanProperty = PipeBlock::UP;

    pub fn new(properties: Properties) -> BlockData {
        FireBlock::new_with(properties, &CLASS, FireBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseFireBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FireBlock::AGE, 0)
                .set_value(FireBlock::NORTH, false)
                .set_value(FireBlock::EAST, false)
                .set_value(FireBlock::SOUTH, false)
                .set_value(FireBlock::WEST, false)
                .set_value(FireBlock::UP, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FireBlock::AGE,
            FireBlock::NORTH,
            FireBlock::EAST,
            FireBlock::SOUTH,
            FireBlock::WEST,
            FireBlock::UP,
        ]);
    }
}
