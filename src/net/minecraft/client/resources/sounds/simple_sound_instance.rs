//! net/minecraft/client/resources/sounds/SimpleSoundInstance.java
//!
//! What the level hands the sound manager, as its constructor of a sound event, a source, a
//! volume, a pitch, a random source and a position makes it (not looping, no delay, linear
//! attenuation, not relative): the random source is `RandomSource.create(seed)`, kept as its seed.
//! Not ported yet: the other constructors (the GUI's, music's, ambient sounds'), and
//! `AbstractSoundInstance`'s resolving of the sound (`resolve`, `getSound`).

use crate::net::minecraft::sounds::sound_event::SoundEvent;
use crate::net::minecraft::sounds::sound_source::SoundSource;

/// A sound played once at a place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleSoundInstance {
    pub sound: SoundEvent,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    /// The seed of its `RandomSource.create(seed)`, which picks among the event's sounds.
    pub seed: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl SimpleSoundInstance {
    /// `SimpleSoundInstance(SoundEvent, SoundSource, float, float, RandomSource, double, double, double)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sound: SoundEvent,
        source: SoundSource,
        volume: f32,
        pitch: f32,
        seed: i64,
        x: f64,
        y: f64,
        z: f64,
    ) -> SimpleSoundInstance {
        SimpleSoundInstance {
            sound,
            source,
            volume,
            pitch,
            seed,
            x,
            y,
            z,
        }
    }
}
