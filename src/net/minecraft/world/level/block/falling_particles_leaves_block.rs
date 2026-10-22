//! net/minecraft/world/level/block/FallingParticlesLeavesBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `leafParticleChance` and `ambientLeavesBlockSoundPlayer`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::leaves_block::{self, LeavesBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct FallingParticlesLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FallingParticlesLeavesBlock",
    &leaves_block::CLASS,
);

impl FallingParticlesLeavesBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        LeavesBlock::new_with(properties, class, definition)
    }
}
