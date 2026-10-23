//! net/minecraft/world/level/block/WaterloggedTransparentBlock.java
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
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::transparent_block::{self, TransparentBlock};
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct WaterloggedTransparentBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(WaterloggedTransparentBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WaterloggedTransparentBlock",
        &transparent_block::CLASS,
    )
};

impl WaterloggedTransparentBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        WaterloggedTransparentBlock::new_with(
            properties,
            &CLASS,
            WaterloggedTransparentBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = TransparentBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(WaterloggedTransparentBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WaterloggedTransparentBlock::WATERLOGGED]);
    }
}

impl WaterloggedTransparentBlock {
    /// `getFluidState`: falling water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(true)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
