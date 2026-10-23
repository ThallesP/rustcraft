//! net/minecraft/world/damagesource/DamageType.java
//!
//! Not ported yet: everything but `STREAM_CODEC`. A damage type's values (its message id,
//! scaling, exhaustion, effects and death message) come with the registries' elements, and its
//! `CODEC` and `DIRECT_CODEC` with DataFixerUpper.

use std::sync::LazyLock;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::DAMAGE_TYPE)));

/// A kind of damage: how it scales with the difficulty, the exhaustion it causes, its effects and
/// its death message.
pub struct DamageType;

impl DamageType {
    /// A damage type of the synchronized registry, by its id.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;
}
