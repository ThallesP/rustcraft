//! net/minecraft/world/level/block/VineBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct VineBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.VineBlock", &block::CLASS);

impl VineBlock {
    pub const UP: &'static BooleanProperty = PipeBlock::UP;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;

    pub fn new(properties: Properties) -> BlockData {
        VineBlock::new_with(properties, &CLASS, VineBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(VineBlock::UP, false)
                .set_value(VineBlock::NORTH, false)
                .set_value(VineBlock::EAST, false)
                .set_value(VineBlock::SOUTH, false)
                .set_value(VineBlock::WEST, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            VineBlock::UP,
            VineBlock::NORTH,
            VineBlock::EAST,
            VineBlock::SOUTH,
            VineBlock::WEST,
        ]);
    }
}
