//! net/minecraft/world/level/material/WaterFluid.java
//!
//! `isSame` (still and flowing water are the same fluid), and what the client's animations ask:
//! its sounds and particles (`animateTick`) and its drip (`getDripParticle`). Not ported yet: its
//! ticking.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub fn is_same(other: Fluids) -> bool {
    other == Fluids::WATER || other == Fluids::FLOWING_WATER
}

/// `animateTick`: flowing water's sound now and then, still or falling water's bubbles.
pub fn animate_tick(
    level: &LevelWithEffects<'_>,
    pos: &BlockPos,
    fluid_state: FluidState,
    random: &mut dyn RandomSource,
) {
    if !fluid_state.is_source() && fluid_state.falling() != Some(true) {
        if random.next_int_bounded(64) == 0 {
            let volume = random.next_float() * 0.25 + 0.75;
            let pitch = random.next_float() + 0.5;
            level.play_local_sound(
                f64::from(pos.get_x()) + 0.5,
                f64::from(pos.get_y()) + 0.5,
                f64::from(pos.get_z()) + 0.5,
                SoundEvents::WATER_AMBIENT,
                SoundSource::AMBIENT,
                volume,
                pitch,
                false,
            );
        }
    } else if random.next_int_bounded(10) == 0 {
        let x = f64::from(pos.get_x()) + random.next_double();
        let y = f64::from(pos.get_y()) + random.next_double();
        let z = f64::from(pos.get_z()) + random.next_double();
        level.add_particle(ParticleTypes::UNDERWATER, x, y, z, 0.0, 0.0, 0.0);
    }
}

/// `getDripParticle`.
pub fn get_drip_particle() -> ParticleOptions {
    ParticleTypes::DRIPPING_WATER.into()
}
