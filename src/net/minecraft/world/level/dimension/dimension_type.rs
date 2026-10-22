//! net/minecraft/world/level/dimension/DimensionType.java
//!
//! Not ported yet: everything but `STREAM_CODEC`. A dimension type's values (its height, light,
//! effects) come with the registries' elements, and the world track's level, which reads them.

use std::sync::LazyLock;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::DIMENSION_TYPE)));

/// The kind of a dimension: how high it is, its light, its sky.
pub struct DimensionType;

impl DimensionType {
    /// A dimension type of the synchronized registry, by its id.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;
}
