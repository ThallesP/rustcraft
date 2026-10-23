//! net/minecraft/core/registries/codec/HolderSetCodec.java
//!
//! A set of a registry's elements: a tag (`#ns:path`) looked up in the registry the ops has, or
//! the elements themselves, a list or one alone (`compactListCodec`) unless `alwaysUseList`.

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::holder_set::HolderSet;
use crate::net::minecraft::resources::registry_ops::HolderGetter;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::tag_key::TagKey;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;

pub struct HolderSetCodec {
    registry_key: ResourceKey,
    tag_key_or_values_codec: EitherCodec<BoxedCodec<TagKey>, BoxedCodec<Vec<Holder>>>,
}

impl HolderSetCodec {
    fn direct_codec(
        element_codec: BoxedCodec<Holder>,
        always_use_list: bool,
    ) -> BoxedCodec<Vec<Holder>> {
        let list_codec = element_codec.clone().list_of().boxed();
        if always_use_list {
            list_codec
        } else {
            ExtraCodecs::compact_list_codec(element_codec, list_codec).boxed()
        }
    }

    pub fn create(
        registry_key: &ResourceKey,
        element_codec: BoxedCodec<Holder>,
        always_use_list: bool,
    ) -> HolderSetCodec {
        HolderSetCodec {
            registry_key: registry_key.clone(),
            tag_key_or_values_codec: EitherCodec::new(
                TagKey::hashed_codec(registry_key),
                HolderSetCodec::direct_codec(element_codec, always_use_list),
            ),
        }
    }

    fn lookup_tag(registry: &HolderGetter<'_>, key: &TagKey) -> DataResult<HolderSet> {
        match registry.get_tag(key) {
            Some(set) => DataResult::success(set),
            None => DataResult::error(|| {
                format!(
                    "Missing tag: '{}' in '{}'",
                    key.location(),
                    key.registry().identifier()
                )
            }),
        }
    }
}

impl Valued for HolderSetCodec {
    type Value = HolderSet;
}

impl<T: OpsValue> Decoder<T> for HolderSetCodec {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<HolderSet, T>> {
        self.tag_key_or_values_codec
            .decode(ops, input)
            .flat_map(|tag_key_or_values| {
                let (tag_key_or_values, rest) = tag_key_or_values.into_parts();
                let result = match tag_key_or_values {
                    Either::Left(tag_key) => match ops.as_registry_ops() {
                        Some(registry_ops) => match registry_ops.getter(&self.registry_key) {
                            Some(registry) => HolderSetCodec::lookup_tag(&registry, &tag_key),
                            None => {
                                let identifier = self.registry_key.identifier().clone();
                                DataResult::error(move || {
                                    format!(
                                        "Registry {identifier} is not available in this context"
                                    )
                                })
                            }
                        },
                        None => DataResult::error(|| {
                            "Registries are not available in this context".to_owned()
                        }),
                    },
                    Either::Right(values) => DataResult::success(HolderSet::Direct(values)),
                };
                result.map(|holders| Pair::new(holders, rest))
            })
    }
}

impl<T: OpsValue> Encoder<T> for HolderSetCodec {
    fn encode(&self, input: &HolderSet, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        match input {
            HolderSet::Named(key) if ops.as_registry_ops().is_some() => {
                let registry_ops = ops.as_registry_ops().expect("checked");
                if registry_ops.getter(&self.registry_key).is_none() {
                    let identifier = self.registry_key.identifier().clone();
                    return DataResult::error(move || {
                        format!("Registry {identifier} is not available in this context")
                    });
                }
                self.tag_key_or_values_codec
                    .encode(&Either::Left(key.clone()), ops, prefix)
            }
            HolderSet::Named(key) => {
                // A named set's stream outside registry ops: its elements, which aren't bound.
                let message = format!("the elements of {key} aren't bound");
                DataResult::error(move || message)
            }
            HolderSet::Direct(values) => {
                self.tag_key_or_values_codec
                    .encode(&Either::Right(values.clone()), ops, prefix)
            }
        }
    }
}
