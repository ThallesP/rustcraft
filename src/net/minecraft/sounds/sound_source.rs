//! net/minecraft/sounds/SoundSource.java

/// The sound category a sound plays in, which the options' volumes are by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundSource {
    MASTER,
    MUSIC,
    RECORDS,
    WEATHER,
    BLOCKS,
    HOSTILE,
    NEUTRAL,
    PLAYERS,
    AMBIENT,
    VOICE,
    UI,
}

impl SoundSource {
    pub fn get_name(self) -> &'static str {
        match self {
            SoundSource::MASTER => "master",
            SoundSource::MUSIC => "music",
            SoundSource::RECORDS => "record",
            SoundSource::WEATHER => "weather",
            SoundSource::BLOCKS => "block",
            SoundSource::HOSTILE => "hostile",
            SoundSource::NEUTRAL => "neutral",
            SoundSource::PLAYERS => "player",
            SoundSource::AMBIENT => "ambient",
            SoundSource::VOICE => "voice",
            SoundSource::UI => "ui",
        }
    }
}
