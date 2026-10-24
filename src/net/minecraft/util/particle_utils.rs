//! net/minecraft/util/ParticleUtils.java
//!
//! Only what blocks' `animateTick`s call: `spawnParticleBelow` and `spawnParticlesAlongAxis`. Not
//! ported yet: the others, which the server's level events call.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::util::valueproviders::uniform_int::UniformInt;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct ParticleUtils;

impl ParticleUtils {
    /// `spawnParticlesAlongAxis`: sparks within the block, spread along the axis it's attached
    /// on, from the level's random.
    pub fn spawn_particles_along_axis(
        attached_axis: Axis,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        radius: f64,
        particle: ParticleOptions,
        spark_count: UniformInt,
    ) {
        let center_of_block = Vec3::at_center_of(pos);
        let step_x = attached_axis == Axis::X;
        let step_y = attached_axis == Axis::Y;
        let step_z = attached_axis == Axis::Z;
        let particle_count = spark_count.sample(&mut *level.get_random());

        for _ in 0..particle_count {
            let (x, y, z, x_base_speed, y_base_speed, z_base_speed) = {
                let mut random = level.get_random();
                let random = &mut *random;
                let x = center_of_block.x
                    + Mth::next_double(random, -1.0, 1.0) * if step_x { 0.5 } else { radius };
                let y = center_of_block.y
                    + Mth::next_double(random, -1.0, 1.0) * if step_y { 0.5 } else { radius };
                let z = center_of_block.z
                    + Mth::next_double(random, -1.0, 1.0) * if step_z { 0.5 } else { radius };
                let x_base_speed = if step_x {
                    Mth::next_double(random, -1.0, 1.0)
                } else {
                    0.0
                };
                let y_base_speed = if step_y {
                    Mth::next_double(random, -1.0, 1.0)
                } else {
                    0.0
                };
                let z_base_speed = if step_z {
                    Mth::next_double(random, -1.0, 1.0)
                } else {
                    0.0
                };
                (x, y, z, x_base_speed, y_base_speed, z_base_speed)
            };
            level.add_particle(particle, x, y, z, x_base_speed, y_base_speed, z_base_speed);
        }
    }

    /// `spawnParticleBelow`: just under the block, anywhere across it.
    pub fn spawn_particle_below(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
        particle: ParticleOptions,
    ) {
        let x = f64::from(pos.get_x()) + random.next_double();
        let y = f64::from(pos.get_y()) - 0.05;
        let z = f64::from(pos.get_z()) + random.next_double();
        level.add_particle(particle, x, y, z, 0.0, 0.0, 0.0);
    }
}
