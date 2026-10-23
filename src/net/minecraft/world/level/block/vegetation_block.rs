//! net/minecraft/world/level/block/VegetationBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct VegetationBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(VegetationBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.VegetationBlock",
        &block::CLASS,
    )
};

impl VegetationBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl VegetationBlock {
    /// `propagatesSkylightDown`: without a fluid.
    fn propagates_skylight_down(block: &BlockData, state: BlockState) -> bool {
        block.get_fluid_state_of(state).is_empty()
    }
}
