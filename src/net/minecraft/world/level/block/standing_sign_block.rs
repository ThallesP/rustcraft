//! net/minecraft/world/level/block/StandingSignBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sign_block::{self, SignBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct StandingSignBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.StandingSignBlock",
    &sign_block::CLASS,
);

impl StandingSignBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;

    pub fn new(properties: Properties) -> BlockData {
        StandingSignBlock::new_with(
            properties,
            &CLASS,
            StandingSignBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SignBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(StandingSignBlock::ROTATION, 8)
                .set_value(SignBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[StandingSignBlock::ROTATION, SignBlock::WATERLOGGED]);
    }
}
