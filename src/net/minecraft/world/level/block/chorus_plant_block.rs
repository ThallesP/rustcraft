//! net/minecraft/world/level/block/ChorusPlantBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::{self, PipeBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ChorusPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ChorusPlantBlock",
    &pipe_block::CLASS,
);

impl ChorusPlantBlock {
    pub fn new(properties: Properties) -> BlockData {
        ChorusPlantBlock::new_with(
            properties,
            &CLASS,
            ChorusPlantBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = PipeBlock::new_with(10.0, properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(PipeBlock::NORTH, false)
                .set_value(PipeBlock::EAST, false)
                .set_value(PipeBlock::SOUTH, false)
                .set_value(PipeBlock::WEST, false)
                .set_value(PipeBlock::UP, false)
                .set_value(PipeBlock::DOWN, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            PipeBlock::NORTH,
            PipeBlock::EAST,
            PipeBlock::SOUTH,
            PipeBlock::WEST,
            PipeBlock::UP,
            PipeBlock::DOWN,
        ]);
    }
}
