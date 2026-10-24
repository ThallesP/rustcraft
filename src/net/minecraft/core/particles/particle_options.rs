//! net/minecraft/core/particles/ParticleOptions.java
//!
//! An interface its classes implement, an enum of those ported here (what blocks' `animateTick`s
//! make), which the particle engine tells apart like Java's providers by their type. A
//! `SimpleParticleType` is its own options. Not ported yet: `ItemParticleOption`,
//! `PowerParticleOption`, `SpellParticleOption`, `ShriekParticleOption`,
//! `SculkChargeParticleOptions`, `VibrationParticleOption`, `TrailParticleOption` and the geyser's.

use crate::net::minecraft::core::particles::block_particle_option::BlockParticleOption;
use crate::net::minecraft::core::particles::color_particle_option::ColorParticleOption;
use crate::net::minecraft::core::particles::dust_color_transition_options::DustColorTransitionOptions;
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::core::particles::particle_type::ParticleType;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;

/// What a particle is made with: its type and the values its class adds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleOptions {
    /// A `SimpleParticleType`.
    Simple(ParticleType),
    Block(BlockParticleOption),
    Color(ColorParticleOption),
    Dust(DustParticleOptions),
    DustColorTransition(DustColorTransitionOptions),
}

impl ParticleOptions {
    pub fn get_type(&self) -> ParticleType {
        match self {
            ParticleOptions::Simple(r#type) => *r#type,
            ParticleOptions::Block(options) => options.get_type(),
            ParticleOptions::Color(options) => options.get_type(),
            ParticleOptions::Dust(_) => ParticleTypes::DUST,
            ParticleOptions::DustColorTransition(_) => ParticleTypes::DUST_COLOR_TRANSITION,
        }
    }
}

impl From<ParticleType> for ParticleOptions {
    /// A `SimpleParticleType` as its options.
    fn from(r#type: ParticleType) -> ParticleOptions {
        debug_assert_eq!(r#type.options_class(), "Simple", "{type:?} isn't simple");
        ParticleOptions::Simple(r#type)
    }
}

impl From<BlockParticleOption> for ParticleOptions {
    fn from(options: BlockParticleOption) -> ParticleOptions {
        ParticleOptions::Block(options)
    }
}

impl From<ColorParticleOption> for ParticleOptions {
    fn from(options: ColorParticleOption) -> ParticleOptions {
        ParticleOptions::Color(options)
    }
}

impl From<DustParticleOptions> for ParticleOptions {
    fn from(options: DustParticleOptions) -> ParticleOptions {
        ParticleOptions::Dust(options)
    }
}

impl From<DustColorTransitionOptions> for ParticleOptions {
    fn from(options: DustColorTransitionOptions) -> ParticleOptions {
        ParticleOptions::DustColorTransition(options)
    }
}
