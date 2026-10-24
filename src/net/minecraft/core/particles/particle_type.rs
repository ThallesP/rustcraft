//! net/minecraft/core/particles/ParticleType.java
//!
//! Java's types are objects the game compares by reference, numbered by
//! `BuiltInRegistries.PARTICLE_TYPE`, so here a type is its id there, `Copy`, like a `Block`, and
//! `SimpleParticleType` is a type whose options are the type itself (`ParticleOptions::Simple`).
//! Not ported yet: `codec` and `streamCodec`.

use std::fmt;

use crate::net::minecraft::core::particles::particle_types::{KEYS, OPTIONS, OVERRIDE_LIMITER};

/// A kind of particle, by its id in `BuiltInRegistries.PARTICLE_TYPE`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleType(pub(crate) u16);

impl ParticleType {
    pub fn id(&self) -> i32 {
        i32::from(self.0)
    }

    /// Its name in the registry, `minecraft:flame`.
    pub fn key(&self) -> &'static str {
        KEYS[usize::from(self.0)]
    }

    /// `BuiltInRegistries.PARTICLE_TYPE.byId(id)`.
    pub fn by_id(id: i32) -> Option<ParticleType> {
        usize::try_from(id)
            .ok()
            .filter(|&id| id < KEYS.len())
            .map(|id| ParticleType(id as u16))
    }

    pub fn registry_size() -> i32 {
        KEYS.len() as i32
    }

    /// `getOverrideLimiter()`: whether its particles show however far from the camera, and
    /// whatever the options' particles say.
    pub fn get_override_limiter(&self) -> bool {
        OVERRIDE_LIMITER[usize::from(self.0)]
    }

    /// The simple name of the class of its options (`Simple` for a `SimpleParticleType`).
    pub fn options_class(&self) -> &'static str {
        OPTIONS[usize::from(self.0)]
    }
}

impl fmt::Debug for ParticleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}
