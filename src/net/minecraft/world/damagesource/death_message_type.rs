//! net/minecraft/world/damagesource/DeathMessageType.java

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::StaticCodec;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

static CODEC: StaticCodec<DeathMessageType> =
    LazyLock::new(|| string_representable::from_values(&DeathMessageType::VALUES));

/// Which death message a damage gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum DeathMessageType {
    DEFAULT,
    FALL_VARIANTS,
    INTENTIONAL_GAME_DESIGN,
}

impl DeathMessageType {
    pub const VALUES: [DeathMessageType; 3] = [
        DeathMessageType::DEFAULT,
        DeathMessageType::FALL_VARIANTS,
        DeathMessageType::INTENTIONAL_GAME_DESIGN,
    ];
    pub const CODEC: &'static StaticCodec<DeathMessageType> = &CODEC;
}

impl StringRepresentable for DeathMessageType {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            DeathMessageType::DEFAULT => "default",
            DeathMessageType::FALL_VARIANTS => "fall_variants",
            DeathMessageType::INTENTIONAL_GAME_DESIGN => "intentional_game_design",
        }
    }
}
