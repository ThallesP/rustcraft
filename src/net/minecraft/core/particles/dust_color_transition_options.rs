//! net/minecraft/core/particles/DustColorTransitionOptions.java
//!
//! Not ported yet: its codecs.

use crate::net::minecraft::core::particles::scalable_particle_options_base::ScalableParticleOptionsBase;
use crate::net::minecraft::util::argb::ARGB;
use crate::org::joml::vector3f::Vector3f;

/// Dust going from one colour to another (RGB), of a scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DustColorTransitionOptions {
    from_color: i32,
    to_color: i32,
    scale: f32,
}

impl DustColorTransitionOptions {
    pub const SCULK_PARTICLE_COLOR: i32 = 3790560;
    pub const SCULK_TO_REDSTONE: DustColorTransitionOptions = DustColorTransitionOptions {
        from_color: DustColorTransitionOptions::SCULK_PARTICLE_COLOR,
        to_color: 16711680,
        scale: 1.0,
    };

    pub fn new(from_color: i32, to_color: i32, scale: f32) -> DustColorTransitionOptions {
        DustColorTransitionOptions {
            from_color,
            to_color,
            scale: ScalableParticleOptionsBase::scale(scale),
        }
    }

    /// The RGB colours it was made with.
    pub fn colors(&self) -> (i32, i32) {
        (self.from_color, self.to_color)
    }

    pub fn get_from_color(&self) -> Vector3f {
        ARGB::vector3f_from_rgb24(self.from_color)
    }

    pub fn get_to_color(&self) -> Vector3f {
        ARGB::vector3f_from_rgb24(self.to_color)
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }
}
