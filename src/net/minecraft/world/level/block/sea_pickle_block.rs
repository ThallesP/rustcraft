//! net/minecraft/world/level/block/SeaPickleBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct SeaPickleBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(SeaPickleBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SeaPickleBlock",
        &vegetation_block::CLASS,
    )
};

impl SeaPickleBlock {
    pub const PICKLES: &'static IntegerProperty = BlockStateProperties::PICKLES;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SeaPickleBlock::new_with(
            properties,
            &CLASS,
            SeaPickleBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(SeaPickleBlock::PICKLES, 1)
                .set_value(SeaPickleBlock::WATERLOGGED, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SeaPickleBlock::PICKLES, SeaPickleBlock::WATERLOGGED]);
    }

    /// Out of water.
    pub fn is_dead(state: BlockState) -> bool {
        !state.get_value(SeaPickleBlock::WATERLOGGED)
    }
}

impl SeaPickleBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
