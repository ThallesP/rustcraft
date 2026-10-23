//! net/minecraft/world/damagesource/DamageScaling.java

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::StaticCodec;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

static CODEC: StaticCodec<DamageScaling> =
    LazyLock::new(|| string_representable::from_values(&DamageScaling::VALUES));

/// How a damage scales with the difficulty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum DamageScaling {
    NEVER,
    WHEN_CAUSED_BY_LIVING_NON_PLAYER,
    ALWAYS,
}

impl DamageScaling {
    pub const VALUES: [DamageScaling; 3] = [
        DamageScaling::NEVER,
        DamageScaling::WHEN_CAUSED_BY_LIVING_NON_PLAYER,
        DamageScaling::ALWAYS,
    ];
    pub const CODEC: &'static StaticCodec<DamageScaling> = &CODEC;
}

impl StringRepresentable for DamageScaling {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            DamageScaling::NEVER => "never",
            DamageScaling::WHEN_CAUSED_BY_LIVING_NON_PLAYER => "when_caused_by_living_non_player",
            DamageScaling::ALWAYS => "always",
        }
    }
}
