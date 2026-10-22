//! net/minecraft/world/level/block/HugeMushroomBlock.java
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

pub struct HugeMushroomBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.HugeMushroomBlock",
    &block::CLASS,
);

impl HugeMushroomBlock {
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const UP: &'static BooleanProperty = PipeBlock::UP;
    pub const DOWN: &'static BooleanProperty = PipeBlock::DOWN;

    pub fn new(properties: Properties) -> BlockData {
        HugeMushroomBlock::new_with(
            properties,
            &CLASS,
            HugeMushroomBlock::create_block_state_definition,
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
                .set_value(HugeMushroomBlock::NORTH, true)
                .set_value(HugeMushroomBlock::EAST, true)
                .set_value(HugeMushroomBlock::SOUTH, true)
                .set_value(HugeMushroomBlock::WEST, true)
                .set_value(HugeMushroomBlock::UP, true)
                .set_value(HugeMushroomBlock::DOWN, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HugeMushroomBlock::UP,
            HugeMushroomBlock::DOWN,
            HugeMushroomBlock::NORTH,
            HugeMushroomBlock::EAST,
            HugeMushroomBlock::SOUTH,
            HugeMushroomBlock::WEST,
        ]);
    }
}
