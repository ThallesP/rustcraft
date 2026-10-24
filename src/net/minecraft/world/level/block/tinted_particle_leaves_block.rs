//! net/minecraft/world/level/block/TintedParticleLeavesBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! For its parent's `animateTick`, its leaves' particle (`spawnFallingLeavesParticle`, the block's
//! tint) and its constructor are ported.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_particles_leaves_block::{
    self, FallingParticlesLeavesBlock,
};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::sounds::ambient_leaves_block_sound_player::AmbientLeavesBlockSoundPlayer;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct TintedParticleLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TintedParticleLeavesBlock",
    &falling_particles_leaves_block::CLASS,
);

impl TintedParticleLeavesBlock {
    /// `TintedParticleLeavesBlock(leafParticleChance, properties)`: no ambient sound.
    pub fn new(leaf_particle_chance: f32, properties: Properties) -> BlockData {
        TintedParticleLeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
            leaf_particle_chance,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        leaf_particle_chance: f32,
    ) -> BlockData {
        FallingParticlesLeavesBlock::new_with(
            properties,
            class,
            definition,
            leaf_particle_chance,
            AmbientLeavesBlockSoundPlayer::no_ambient_sound(),
        )
    }
}
