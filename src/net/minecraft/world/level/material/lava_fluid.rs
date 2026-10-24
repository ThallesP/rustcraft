//! net/minecraft/world/level/material/LavaFluid.java
//!
//! `isSame` (still and flowing lava are the same fluid), and what the client's animations ask:
//! its pops and sounds (`animateTick`) and its drip (`getDripParticle`). Not ported yet: its
//! ticking and fire.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub fn is_same(other: Fluids) -> bool {
    other == Fluids::LAVA || other == Fluids::FLOWING_LAVA
}

/// `animateTick`: under open air, a pop of lava now and then, and its sound.
pub fn animate_tick(level: &LevelWithEffects<'_>, pos: &BlockPos, random: &mut dyn RandomSource) {
    let above = pos.above();
    if level.get_block_state(&above).is_air() && !level.get_block_state(&above).is_solid_render() {
        if random.next_int_bounded(100) == 0 {
            let xx = f64::from(pos.get_x()) + random.next_double();
            let yy = f64::from(pos.get_y()) + 1.0;
            let zz = f64::from(pos.get_z()) + random.next_double();
            level.add_particle(ParticleTypes::LAVA, xx, yy, zz, 0.0, 0.0, 0.0);
            let volume = 0.2 + random.next_float() * 0.2;
            let pitch = 0.9 + random.next_float() * 0.15;
            level.play_local_sound(
                xx,
                yy,
                zz,
                SoundEvents::LAVA_POP,
                SoundSource::AMBIENT,
                volume,
                pitch,
                false,
            );
        }

        if random.next_int_bounded(200) == 0 {
            let volume = 0.2 + random.next_float() * 0.2;
            let pitch = 0.9 + random.next_float() * 0.15;
            level.play_local_sound(
                f64::from(pos.get_x()),
                f64::from(pos.get_y()),
                f64::from(pos.get_z()),
                SoundEvents::LAVA_AMBIENT,
                SoundSource::AMBIENT,
                volume,
                pitch,
                false,
            );
        }
    }
}

/// `getDripParticle`.
pub fn get_drip_particle() -> ParticleOptions {
    ParticleTypes::DRIPPING_LAVA.into()
}
