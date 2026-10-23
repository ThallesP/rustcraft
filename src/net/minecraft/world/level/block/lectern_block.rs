//! net/minecraft/world/level/block/LecternBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct LecternBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(LecternBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LecternBlock",
        &base_entity_block::CLASS,
    )
};

impl LecternBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const HAS_BOOK: &'static BooleanProperty = BlockStateProperties::HAS_BOOK;

    pub fn new(properties: Properties) -> BlockData {
        LecternBlock::new_with(
            properties,
            &CLASS,
            LecternBlock::create_block_state_definition,
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
                .set_value(LecternBlock::FACING, Direction::NORTH)
                .set_value(LecternBlock::POWERED, false)
                .set_value(LecternBlock::HAS_BOOK, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            LecternBlock::FACING,
            LecternBlock::POWERED,
            LecternBlock::HAS_BOOK,
        ]);
    }
}

impl LecternBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
