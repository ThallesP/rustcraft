//! net/minecraft/core/registries/codec/RegistryFileCodec.java
//!
//! A holder of a registry's element: its identifier, looked up in the registry the ops has, or
//! where inline definitions are allowed, the element written out, which Java makes a direct
//! holder of. Direct holders aren't ported (`Holder` is a reference), so an inline element fails
//! to decode saying so; the vanilla data's are references. Not ported yet: encoding (the server's).

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
pub struct RegistryFileCodec {
    registry_key: ResourceKey,
    // The element's class, which a failed inline decode names.
    element: &'static str,
    allow_inline: bool,
}

impl RegistryFileCodec {
    /// `create(registryKey, elementCodec, allowInline)`, the element codec named by its class.
    pub fn create(
        registry_key: &ResourceKey,
        element: &'static str,
        allow_inline: bool,
    ) -> RegistryFileCodec {
        RegistryFileCodec {
            registry_key: registry_key.clone(),
            element,
            allow_inline,
        }
    }

    fn decode_inline<T: OpsValue>(&self) -> DataResult<Pair<Holder, T>> {
        let element = self.element;
        DataResult::error(move || format!("{element}'s inline definition isn't ported"))
    }
}

impl Valued for RegistryFileCodec {
    type Value = Holder;
}

impl<T: OpsValue> Decoder<T> for RegistryFileCodec {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Holder, T>> {
        let Some(registry_ops) = ops.as_registry_ops() else {
            return self.decode_inline();
        };
        let decoded = Identifier::CODEC.decode(ops, input);
        let Some(pair) = decoded.into_result() else {
            return if !self.allow_inline {
                DataResult::error(|| "Inline definitions not allowed here".to_owned())
            } else {
                self.decode_inline()
            };
        };
        let Some(lookup) = registry_ops.getter(&self.registry_key) else {
            let identifier = self.registry_key.identifier().clone();
            return DataResult::error(move || format!("Registry does not exist: {identifier}"));
        };
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
    }
}

impl<T: OpsValue> Encoder<T> for RegistryFileCodec {
    fn encode(&self, input: &Holder, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        let Holder::Reference(key) = input;
        Identifier::CODEC.encode(key.identifier(), ops, prefix)
    }
}
