//! net/minecraft/world/level/block/piston/PistonBaseBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `isSticky`.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct PistonBaseBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(PistonBaseBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.piston.PistonBaseBlock",
        &directional_block::CLASS,
    )
};

impl PistonBaseBlock {
    pub const EXTENDED: &'static BooleanProperty = BlockStateProperties::EXTENDED;

    pub fn new(properties: Properties) -> BlockData {
        PistonBaseBlock::new_with(
            properties,
            &CLASS,
            PistonBaseBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::NORTH)
                .set_value(PistonBaseBlock::EXTENDED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING, PistonBaseBlock::EXTENDED]);
    }
}

impl PistonBaseBlock {
    /// `useShapeForLightOcclusion`: when it's extended.
    fn use_shape_for_light_occlusion(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(PistonBaseBlock::EXTENDED)
    }
}
