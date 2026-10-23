//! net/minecraft/world/entity/player/PlayerModelType.java
//!
//! Not ported yet: `CODEC` and `STREAM_CODEC`, which come with the skin patches that carry one.

/// A skin's arms: slim (three pixels wide) or wide (four).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerModelType {
    SLIM,
    WIDE,
}

impl PlayerModelType {
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            PlayerModelType::SLIM => "slim",
            PlayerModelType::WIDE => "wide",
        }
    }

    /// The name Mojang's services give the model (`"slim"` or `"default"`), wide for any other.
    pub fn by_legacy_services_name(name: Option<&str>) -> PlayerModelType {
        match name {
            Some("slim") => PlayerModelType::SLIM,
            _ => PlayerModelType::WIDE,
        }
    }
}
