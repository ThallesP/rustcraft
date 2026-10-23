//! net/minecraft/world/level/block/ShulkerBoxBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `color`.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ShulkerBoxBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(ShulkerBoxBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShulkerBoxBlock",
        &base_entity_block::CLASS,
    )
};

impl ShulkerBoxBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        ShulkerBoxBlock::new_with(
            properties,
            &CLASS,
            ShulkerBoxBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(ShulkerBoxBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ShulkerBoxBlock::FACING]);
    }
}

impl ShulkerBoxBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}
