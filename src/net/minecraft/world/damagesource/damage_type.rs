//! net/minecraft/world/damagesource/DamageType.java
//!
//! A kind of damage, an element of the synchronized `damage_type` registry, as `DIRECT_CODEC`
//! reads it (`RegistryDataLoader`). `CODEC`, a holder by name, comes with its callers.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, FLOAT, STRING, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::damagesource::damage_effects::DamageEffects;
use crate::net::minecraft::world::damagesource::damage_scaling::DamageScaling;
use crate::net::minecraft::world::damagesource::death_message_type::DeathMessageType;

static DIRECT_CODEC: StaticCodec<DamageType> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            STRING
                .field_of("message_id")
                .for_getter(|o: &DamageType| o.msg_id.clone()),
            DamageScaling::CODEC
                .shared()
                .field_of("scaling")
                .for_getter(|o: &DamageType| o.scaling),
            FLOAT
                .field_of("exhaustion")
                .for_getter(|o: &DamageType| o.exhaustion),
            DamageEffects::CODEC
                .shared()
                .optional_field_of_default("effects", DamageEffects::HURT)
                .for_getter(|o: &DamageType| o.effects),
            DeathMessageType::CODEC
                .shared()
                .optional_field_of_default("death_message_type", DeathMessageType::DEFAULT)
                .for_getter(|o: &DamageType| o.death_message_type),
        ),
        DamageType::new,
    )
    .boxed()
});

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::DAMAGE_TYPE)));

/// A kind of damage: how it scales with the difficulty, the exhaustion it causes, its effects and
/// its death message.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageType {
    msg_id: String,
    scaling: DamageScaling,
    exhaustion: f32,
    effects: DamageEffects,
    death_message_type: DeathMessageType,
}

impl DamageType {
    pub const DIRECT_CODEC: &'static StaticCodec<DamageType> = &DIRECT_CODEC;

    /// A damage type of the synchronized registry, by its id.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;

    pub fn new(
        msg_id: String,
        scaling: DamageScaling,
        exhaustion: f32,
        effects: DamageEffects,
        death_message_type: DeathMessageType,
    ) -> DamageType {
        DamageType {
            msg_id,
            scaling,
            exhaustion,
            effects,
            death_message_type,
        }
    }

    pub fn msg_id(&self) -> &str {
        &self.msg_id
    }

    pub fn scaling(&self) -> DamageScaling {
        self.scaling
    }

    pub fn exhaustion(&self) -> f32 {
        self.exhaustion
    }

    pub fn effects(&self) -> DamageEffects {
        self.effects
    }

    pub fn death_message_type(&self) -> DeathMessageType {
        self.death_message_type
    }
}
