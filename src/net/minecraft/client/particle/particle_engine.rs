//! net/minecraft/client/particle/ParticleEngine.java
//!
//! Only what the level hands it, as a trait the game's particle engine implements: a particle to
//! make (`createParticle`), which the level's `addParticle` has decided shows (its distance from
//! the camera, the options' particles). The engine itself (its providers by type, the particles'
//! classes, their ticking and rendering) is the render track's, which implements this; until then
//! the client's is `NoParticles`, which drops them.

use crate::net::minecraft::core::particles::particle_options::ParticleOptions;

/// What makes the level's particles.
pub trait ParticleEngine {
    /// `createParticle(ParticleOptions, double, double, double, double, double, double)`: a
    /// particle of the options at the position, with the speed.
    #[allow(clippy::too_many_arguments)]
    fn create_particle(
        &self,
        options: &ParticleOptions,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    );
}

/// The client's particle engine until the render track's comes: it makes nothing.
pub struct NoParticles;

impl ParticleEngine for NoParticles {
    fn create_particle(
        &self,
        _options: &ParticleOptions,
        _x: f64,
        _y: f64,
        _z: f64,
        _xd: f64,
        _yd: f64,
        _zd: f64,
    ) {
    }
}
