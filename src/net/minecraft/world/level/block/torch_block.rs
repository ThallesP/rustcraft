//! net/minecraft/world/level/block/TorchBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `flameParticle`.

use crate::net::minecraft::world::level::block::base_torch_block::{self, BaseTorchBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TorchBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TorchBlock",
    &base_torch_block::CLASS,
);

impl TorchBlock {
    pub fn new(properties: Properties) -> BlockData {
        TorchBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseTorchBlock::new_with(properties, class, definition)
    }
}
