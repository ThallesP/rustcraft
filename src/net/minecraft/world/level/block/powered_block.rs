//! net/minecraft/world/level/block/PoweredBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct PoweredBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(PoweredBlock::is_signal_source),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PoweredBlock",
        &block::CLASS,
    )
};

impl PoweredBlock {
    pub fn new(properties: Properties) -> BlockData {
        PoweredBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl PoweredBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
