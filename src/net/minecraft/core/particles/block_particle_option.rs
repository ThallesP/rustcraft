//! net/minecraft/core/particles/BlockParticleOption.java
//!
//! Not ported yet: its codecs.

use crate::net::minecraft::core::particles::particle_type::ParticleType;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// A particle of a block's state: its dust falling, its marker, its pieces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockParticleOption {
    r#type: ParticleType,
    state: BlockState,
}

impl BlockParticleOption {
    pub fn new(r#type: ParticleType, state: BlockState) -> BlockParticleOption {
        BlockParticleOption { r#type, state }
    }

    pub fn get_type(&self) -> ParticleType {
        self.r#type
    }

    pub fn get_state(&self) -> BlockState {
        self.state
    }
}
