//! net/minecraft/core/particles/DustParticleOptions.java
//!
//! Not ported yet: its codecs.

use crate::net::minecraft::core::particles::scalable_particle_options_base::ScalableParticleOptionsBase;
use crate::net::minecraft::util::argb::ARGB;
use crate::org::joml::vector3f::Vector3f;

/// Redstone's dust: a colour (RGB) and a scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DustParticleOptions {
    color: i32,
    scale: f32,
}

impl DustParticleOptions {
    pub const REDSTONE_PARTICLE_COLOR: i32 = 16711680;
    pub const REDSTONE: DustParticleOptions = DustParticleOptions {
        color: DustParticleOptions::REDSTONE_PARTICLE_COLOR,
        scale: 1.0,
    };

    pub fn new(color: i32, scale: f32) -> DustParticleOptions {
        DustParticleOptions {
            color,
            scale: ScalableParticleOptionsBase::scale(scale),
        }
    }

    /// The RGB colour it was made with.
    pub fn color(&self) -> i32 {
        self.color
    }

    pub fn get_color(&self) -> Vector3f {
        ARGB::vector3f_from_rgb24(self.color)
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }
}
