//! net/minecraft/world/level/block/EndPortalFrameBlock.java
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
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct EndPortalFrameBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(EndPortalFrameBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndPortalFrameBlock",
        &block::CLASS,
    )
};

impl EndPortalFrameBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HAS_EYE: &'static BooleanProperty = BlockStateProperties::EYE;

    pub fn new(properties: Properties) -> BlockData {
        EndPortalFrameBlock::new_with(
            properties,
            &CLASS,
            EndPortalFrameBlock::create_block_state_definition,
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
                .set_value(EndPortalFrameBlock::FACING, Direction::NORTH)
                .set_value(EndPortalFrameBlock::HAS_EYE, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[EndPortalFrameBlock::FACING, EndPortalFrameBlock::HAS_EYE]);
    }
}

impl EndPortalFrameBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
