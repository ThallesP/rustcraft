//! net/minecraft/world/level/block/BarrierBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct BarrierBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(BarrierBlock::propagates_skylight_down),
    get_fluid_state: Some(BarrierBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BarrierBlock",
        &block::CLASS,
    )
};

impl BarrierBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        BarrierBlock::new_with(
            properties,
            &CLASS,
            BarrierBlock::create_block_state_definition,
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
                .default_block_state()
                .set_value(BarrierBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BarrierBlock::WATERLOGGED]);
    }
}

impl BarrierBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl BarrierBlock {
    /// `propagatesSkylightDown`: without a fluid.
    fn propagates_skylight_down(block: &BlockData, state: BlockState) -> bool {
        block.get_fluid_state_of(state).is_empty()
    }
}
