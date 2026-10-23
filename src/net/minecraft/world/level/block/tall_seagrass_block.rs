//! net/minecraft/world/level/block/TallSeagrassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct TallSeagrassBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(TallSeagrassBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TallSeagrassBlock",
        &double_plant_block::CLASS,
    )
};

impl TallSeagrassBlock {
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> = DoublePlantBlock::HALF;

    pub fn new(properties: Properties) -> BlockData {
        TallSeagrassBlock::new_with(
            properties,
            &CLASS,
            DoublePlantBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DoublePlantBlock::new_with(properties, class, definition)
    }
}

impl TallSeagrassBlock {
    /// `getFluidState`: always still water.
    fn get_fluid_state(_block: &BlockData, _state: BlockState) -> FluidState {
        Fluids::WATER.get_source(false)
    }
}
