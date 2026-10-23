//! net/minecraft/world/level/block/MangroveRootsBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct MangroveRootsBlock;

pub static CLASS: BlockClass = BlockClass {
    skip_rendering: Some(MangroveRootsBlock::skip_rendering),
    get_fluid_state: Some(MangroveRootsBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MangroveRootsBlock",
        &block::CLASS,
    )
};

impl MangroveRootsBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        MangroveRootsBlock::new_with(
            properties,
            &CLASS,
            MangroveRootsBlock::create_block_state_definition,
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
                .set_value(MangroveRootsBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[MangroveRootsBlock::WATERLOGGED]);
    }
}

impl MangroveRootsBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl MangroveRootsBlock {
    /// `skipRendering`: the faces between two blocks of mangrove roots stacked up.
    fn skip_rendering(
        _block: &BlockData,
        _state: BlockState,
        neighbor_state: BlockState,
        direction: Direction,
    ) -> bool {
        neighbor_state.is(Blocks::MANGROVE_ROOTS) && direction.get_axis() == Axis::Y
    }
}
