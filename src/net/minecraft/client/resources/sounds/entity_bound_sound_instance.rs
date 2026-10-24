//! net/minecraft/client/resources/sounds/EntityBoundSoundInstance.java
//!
//! What the level hands the sound manager for a sound that follows an entity, as its constructor
//! makes it: the random source is `RandomSource.create(seed)`, kept as its seed, and the entity is
//! its id. Not ported yet: its tick, which follows the entity.

use crate::net::minecraft::sounds::sound_event::SoundEvent;
use crate::net::minecraft::sounds::sound_source::SoundSource;

/// A sound played once where an entity is, following it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityBoundSoundInstance {
    pub sound: SoundEvent,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub entity: i32,
    pub seed: i64,
}
