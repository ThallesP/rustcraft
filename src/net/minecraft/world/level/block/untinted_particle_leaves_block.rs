//! net/minecraft/world/level/block/UntintedParticleLeavesBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `leafParticleChance`, `leafParticle` and `ambientLeavesBlockSoundPlayer`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_particles_leaves_block::{
    self, FallingParticlesLeavesBlock,
};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct UntintedParticleLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.UntintedParticleLeavesBlock",
    &falling_particles_leaves_block::CLASS,
);

impl UntintedParticleLeavesBlock {
    pub fn new(properties: Properties) -> BlockData {
        UntintedParticleLeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FallingParticlesLeavesBlock::new_with(properties, class, definition)
    }
}
