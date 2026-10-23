//! net/minecraft/world/level/block/SculkShriekerBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct SculkShriekerBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(SculkShriekerBlock::use_shape_for_light_occlusion),
    get_fluid_state: Some(SculkShriekerBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SculkShriekerBlock",
        &base_entity_block::CLASS,
    )
};

impl SculkShriekerBlock {
    pub const SHRIEKING: &'static BooleanProperty = BlockStateProperties::SHRIEKING;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const CAN_SUMMON: &'static BooleanProperty = BlockStateProperties::CAN_SUMMON;

    pub fn new(properties: Properties) -> BlockData {
        SculkShriekerBlock::new_with(
            properties,
            &CLASS,
            SculkShriekerBlock::create_block_state_definition,
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
                .set_value(SculkShriekerBlock::SHRIEKING, false)
                .set_value(SculkShriekerBlock::WATERLOGGED, false)
                .set_value(SculkShriekerBlock::CAN_SUMMON, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SculkShriekerBlock::SHRIEKING]);
        builder.add(&[SculkShriekerBlock::WATERLOGGED]);
        builder.add(&[SculkShriekerBlock::CAN_SUMMON]);
    }
}

impl SculkShriekerBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl SculkShriekerBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
