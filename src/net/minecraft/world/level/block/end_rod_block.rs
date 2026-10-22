//! net/minecraft/world/level/block/EndRodBlock.java
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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct EndRodBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.EndRodBlock",
    &rod_block::CLASS,
);

impl EndRodBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndRodBlock::new_with(
            properties,
            &CLASS,
            EndRodBlock::create_block_state_definition,
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
                .set_value(DirectionalBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING]);
    }
}
