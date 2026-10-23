//! net/minecraft/world/level/block/SeagrassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct SeagrassBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(SeagrassBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SeagrassBlock",
        &vegetation_block::CLASS,
    )
};

impl SeagrassBlock {
    pub fn new(properties: Properties) -> BlockData {
        SeagrassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}

impl SeagrassBlock {
    /// `getFluidState`: always still water.
    fn get_fluid_state(_block: &BlockData, _state: BlockState) -> FluidState {
        Fluids::WATER.get_source(false)
    }
}
