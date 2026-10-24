//! net/minecraft/world/level/block/UntintedParticleLeavesBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! For its parent's `animateTick`, its leaves' particle (`spawnFallingLeavesParticle`) and its
//! constructor are ported.

use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::world::level::block::block::{BlockClass, BlockData};
use crate::net::minecraft::world::level::block::falling_particles_leaves_block::{
    self, FallingParticlesLeavesBlock,
};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlockFields;
use crate::net::minecraft::world::level::block::sounds::ambient_leaves_block_sound_player::AmbientLeavesBlockSoundPlayer;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct UntintedParticleLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.UntintedParticleLeavesBlock",
    &falling_particles_leaves_block::CLASS,
);

impl UntintedParticleLeavesBlock {
    /// `UntintedParticleLeavesBlock(leafParticleChance, leafParticle, ambientLeavesBlockSoundPlayer,
    /// properties)`.
    pub fn new(
        leaf_particle_chance: f32,
        leaf_particle: ParticleOptions,
        ambient_leaves_block_sound_player: AmbientLeavesBlockSoundPlayer,
        properties: Properties,
    ) -> BlockData {
        let mut block = FallingParticlesLeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
            leaf_particle_chance,
            ambient_leaves_block_sound_player,
        );
        let mut fields = *block.fields::<LeavesBlockFields>();
        fields.leaf_particle = Some(leaf_particle);
        block.set_fields(fields);
        block
    }
}
