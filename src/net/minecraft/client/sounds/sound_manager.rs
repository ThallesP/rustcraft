//! net/minecraft/client/sounds/SoundManager.java
//!
//! Only what the level hands it, as a trait the game's sound manager implements: a sound to play
//! now (`play`) or after a number of ticks (`playDelayed`). The client plays no sound yet, so its
//! sound manager is `NoSound`, which drops them. Not ported yet: the sound engine, the listener,
//! the options' volumes and the sounds' resources.

use crate::net::minecraft::client::resources::sounds::entity_bound_sound_instance::EntityBoundSoundInstance;
use crate::net::minecraft::client::resources::sounds::simple_sound_instance::SimpleSoundInstance;

/// A sound the level plays (Java's `SoundInstance`s it makes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundInstance {
    Simple(SimpleSoundInstance),
    EntityBound(EntityBoundSoundInstance),
}

/// What plays the level's sounds.
pub trait SoundManager {
    /// `play(SoundInstance)`.
    fn play(&self, sound: SoundInstance);

    /// `playDelayed(SoundInstance, int)`: after that many ticks.
    fn play_delayed(&self, sound: SoundInstance, delay: i32);
}

/// The client's sound manager until sound is ported: it plays nothing.
pub struct NoSound;

impl SoundManager for NoSound {
    fn play(&self, _sound: SoundInstance) {}

    fn play_delayed(&self, _sound: SoundInstance, _delay: i32) {}
}
