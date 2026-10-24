//! net/minecraft/world/entity/player/PlayerModelType.java
//!
//! Not ported yet: `STREAM_CODEC`, which comes with the skin patches' stream codec.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::StaticCodec;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

static CODEC: StaticCodec<PlayerModelType> =
    LazyLock::new(|| string_representable::from_values(&PlayerModelType::VALUES));

/// A skin's arms: slim (three pixels wide) or wide (four).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerModelType {
    SLIM,
    WIDE,
}

impl PlayerModelType {
    pub const VALUES: [PlayerModelType; 2] = [PlayerModelType::SLIM, PlayerModelType::WIDE];
    pub const CODEC: &'static StaticCodec<PlayerModelType> = &CODEC;

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

impl StringRepresentable for PlayerModelType {
    fn get_serialized_name(&self) -> &'static str {
        PlayerModelType::get_serialized_name(self)
    }
}
