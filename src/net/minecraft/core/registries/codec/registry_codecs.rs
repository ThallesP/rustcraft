//! net/minecraft/core/registries/codec/RegistryCodecs.java
//!
//! The holder codecs: `holder(registryKey)` and `holderSet(registryKey)` by an element's
//! identifier, and `holder(registryKey, elementCodec)`, which also takes an element written
//! inline (`RegistryFileCodec`, where the element codec is named by its class: inline elements
//! aren't ported). `holderSet` of an element codec comes with its callers.

use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt};
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::holder_set::HolderSet;
use crate::net::minecraft::core::registries::codec::holder_set_codec::HolderSetCodec;
use crate::net::minecraft::core::registries::codec::registry_file_codec::RegistryFileCodec;
use crate::net::minecraft::core::registries::codec::registry_fixed_codec::RegistryFixedCodec;
use crate::net::minecraft::resources::resource_key::ResourceKey;

pub struct RegistryCodecs;

impl RegistryCodecs {
    pub fn holder(registry_key: &ResourceKey) -> BoxedCodec<Holder> {
        RegistryFixedCodec::create(registry_key).boxed()
    }

    /// `holder(registryKey, elementCodec)`: `RegistryFileCodec`, inline elements allowed.
    pub fn holder_by_file(registry_key: &ResourceKey, element: &'static str) -> BoxedCodec<Holder> {
        RegistryFileCodec::create(registry_key, element, true).boxed()
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
