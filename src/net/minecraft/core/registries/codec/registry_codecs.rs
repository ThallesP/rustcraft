//! net/minecraft/core/registries/codec/RegistryCodecs.java
//!
//! The holder codecs by an element's identifier: `holder(registryKey)` and
//! `holderSet(registryKey)`. The overloads taking an element codec (`RegistryFileCodec`, an
//! element written inline) come with their callers.

use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt};
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::holder_set::HolderSet;
use crate::net::minecraft::core::registries::codec::holder_set_codec::HolderSetCodec;
use crate::net::minecraft::core::registries::codec::registry_fixed_codec::RegistryFixedCodec;
use crate::net::minecraft::resources::resource_key::ResourceKey;

pub struct RegistryCodecs;

impl RegistryCodecs {
    pub fn holder(registry_key: &ResourceKey) -> BoxedCodec<Holder> {
        RegistryFixedCodec::create(registry_key).boxed()
    }

    pub fn holder_set(registry_key: &ResourceKey) -> BoxedCodec<HolderSet> {
        RegistryCodecs::holder_set_always_use_list(registry_key, false)
    }

    /// `holderSet(ResourceKey, boolean)`.
    pub fn holder_set_always_use_list(
        registry_key: &ResourceKey,
        always_use_list: bool,
    ) -> BoxedCodec<HolderSet> {
        HolderSetCodec::create(
            registry_key,
            RegistryCodecs::holder(registry_key),
            always_use_list,
        )
        .boxed()
    }
}
