//! net/minecraft/world/level/block/CandleBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::abstract_candle_block::{
    self, AbstractCandleBlock,
};
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
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct CandleBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(CandleBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CandleBlock",
        &abstract_candle_block::CLASS,
    )
};

impl CandleBlock {
    pub const CANDLES: &'static IntegerProperty = BlockStateProperties::CANDLES;
    pub const LIT: &'static BooleanProperty = AbstractCandleBlock::LIT;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        CandleBlock::new_with(
            properties,
            &CLASS,
            CandleBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCandleBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CandleBlock::CANDLES, 1)
                .set_value(CandleBlock::LIT, false)
                .set_value(CandleBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CandleBlock::CANDLES,
            CandleBlock::LIT,
            CandleBlock::WATERLOGGED,
        ]);
    }

    /// `LIGHT_EMISSION`: 3 for each lit candle.
    pub fn light_emission(state: BlockState) -> i32 {
        if state.get_value(CandleBlock::LIT) {
            3 * state.get_value(CandleBlock::CANDLES)
        } else {
            0
        }
    }
}

impl CandleBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
