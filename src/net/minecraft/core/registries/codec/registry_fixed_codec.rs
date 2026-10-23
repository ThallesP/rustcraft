//! net/minecraft/core/registries/codec/RegistryFixedCodec.java
//!
//! A holder of a registry's element by its identifier, looked up in the registry the ops has
//! (`RegistryOps`). `canSerializeIn` holds for every holder here: a holder is its key, which any
//! registry of its kind can name.

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

#[derive(Clone, Debug)]
pub struct RegistryFixedCodec {
    registry_key: ResourceKey,
}

impl RegistryFixedCodec {
    pub fn create(registry_key: &ResourceKey) -> RegistryFixedCodec {
        RegistryFixedCodec {
            registry_key: registry_key.clone(),
        }
    }
}

impl Valued for RegistryFixedCodec {
    type Value = Holder;
}

impl<T: OpsValue> Encoder<T> for RegistryFixedCodec {
    fn encode(&self, input: &Holder, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        if let Some(registry_ops) = ops.as_registry_ops()
            && registry_ops.getter(&self.registry_key).is_some()
        {
            let Holder::Reference(key) = input;
            return Identifier::CODEC.encode(key.identifier(), ops, prefix);
        }
        let identifier = self.registry_key.identifier().clone();
        DataResult::error(move || format!("Can't access registry {identifier}"))
    }
}

impl<T: OpsValue> Decoder<T> for RegistryFixedCodec {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Holder, T>> {
        if let Some(registry_ops) = ops.as_registry_ops()
            && let Some(lookup) = registry_ops.getter(&self.registry_key)
        {
            return Identifier::CODEC.decode(ops, input).flat_map(|pair| {
                let (identifier, rest) = pair.into_parts();
                let element_key = ResourceKey::create(&self.registry_key, identifier);
                match lookup.get(&element_key) {
                    Some(holder) => DataResult::success(holder),
                    None => DataResult::error(|| {
                        format!(
                            "Failed to get element {} from registry {}",
                            element_key.identifier(),
                            element_key.registry()
                        )
                    }),
                }
                .map(|holder| Pair::new(holder, rest))
                .set_lifecycle(Lifecycle::stable())
            });
        }
        let identifier = self.registry_key.identifier().clone();
        DataResult::error(move || format!("Can't access registry {identifier}"))
    }
}
