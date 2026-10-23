//! net/minecraft/world/damagesource/DamageEffects.java
//!
//! Not ported yet: each effect's sound (`sound()`), which comes with sounds.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::StaticCodec;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

static CODEC: StaticCodec<DamageEffects> =
    LazyLock::new(|| string_representable::from_values(&DamageEffects::VALUES));

/// What a damage looks and sounds like to the player taking it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum DamageEffects {
    HURT,
    THORNS,
    DROWNING,
    BURNING,
    POKING,
    FREEZING,
}

impl DamageEffects {
    pub const VALUES: [DamageEffects; 6] = [
        DamageEffects::HURT,
        DamageEffects::THORNS,
        DamageEffects::DROWNING,
        DamageEffects::BURNING,
        DamageEffects::POKING,
        DamageEffects::FREEZING,
    ];
    pub const CODEC: &'static StaticCodec<DamageEffects> = &CODEC;
}

impl StringRepresentable for DamageEffects {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            DamageEffects::HURT => "hurt",
            DamageEffects::THORNS => "thorns",
            DamageEffects::DROWNING => "drowning",
            DamageEffects::BURNING => "burning",
            DamageEffects::POKING => "poking",
            DamageEffects::FREEZING => "freezing",
        }
    }
}
