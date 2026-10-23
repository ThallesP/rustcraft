//! net/minecraft/world/level/block/StonecutterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct StonecutterBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(StonecutterBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.StonecutterBlock",
        &block::CLASS,
    )
};

impl StonecutterBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        StonecutterBlock::new_with(
            properties,
            &CLASS,
            StonecutterBlock::create_block_state_definition,
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
                .set_value(StonecutterBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[StonecutterBlock::FACING]);
    }
}

impl StonecutterBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
