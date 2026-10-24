//! net/minecraft/world/level/block/MangroveLeavesBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! For its parent's `animateTick`, its constructor (`leafParticleChance`) is ported.

use crate::net::minecraft::world::level::block::block::{BlockClass, BlockData};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::tinted_particle_leaves_block::{
    self, TintedParticleLeavesBlock,
};

pub struct MangroveLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.MangroveLeavesBlock",
    &tinted_particle_leaves_block::CLASS,
);

impl MangroveLeavesBlock {
    /// `MangroveLeavesBlock(leafParticleChance, properties)`.
    pub fn new(leaf_particle_chance: f32, properties: Properties) -> BlockData {
        TintedParticleLeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
            leaf_particle_chance,
        )
    }
}
