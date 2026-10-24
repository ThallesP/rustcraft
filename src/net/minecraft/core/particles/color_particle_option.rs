//! net/minecraft/core/particles/ColorParticleOption.java
//!
//! Not ported yet: its codecs.

use crate::net::minecraft::core::particles::particle_type::ParticleType;
use crate::net::minecraft::util::argb::ARGB;

/// A particle of a colour, ARGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorParticleOption {
    r#type: ParticleType,
    color: i32,
}

impl ColorParticleOption {
    /// `create(type, int)`.
    pub fn create(r#type: ParticleType, color: i32) -> ColorParticleOption {
        ColorParticleOption { r#type, color }
    }

    /// `create(type, float, float, float)`: opaque.
    pub fn create_rgb(
        r#type: ParticleType,
        red: f32,
        green: f32,
        blue: f32,
    ) -> ColorParticleOption {
        ColorParticleOption::create(r#type, ARGB::color_from_float(1.0, red, green, blue))
    }

    pub fn get_type(&self) -> ParticleType {
        self.r#type
    }

    /// The ARGB colour it was made with.
    pub fn color(&self) -> i32 {
        self.color
    }

    pub fn get_red(&self) -> f32 {
        ARGB::red(self.color) as f32 / 255.0
    }

    pub fn get_green(&self) -> f32 {
        ARGB::green(self.color) as f32 / 255.0
    }

    pub fn get_blue(&self) -> f32 {
        ARGB::blue(self.color) as f32 / 255.0
    }

    pub fn get_alpha(&self) -> f32 {
        ARGB::alpha(self.color) as f32 / 255.0
    }
}
