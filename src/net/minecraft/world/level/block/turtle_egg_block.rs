//! net/minecraft/world/level/block/TurtleEggBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct TurtleEggBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TurtleEggBlock",
    &block::CLASS,
);

impl TurtleEggBlock {
    pub const HATCH: &'static IntegerProperty = BlockStateProperties::HATCH;
    pub const EGGS: &'static IntegerProperty = BlockStateProperties::EGGS;

    pub fn new(properties: Properties) -> BlockData {
        TurtleEggBlock::new_with(
            properties,
            &CLASS,
            TurtleEggBlock::create_block_state_definition,
        )
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
                .set_value(TurtleEggBlock::HATCH, 0)
                .set_value(TurtleEggBlock::EGGS, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[TurtleEggBlock::HATCH, TurtleEggBlock::EGGS]);
    }
}
