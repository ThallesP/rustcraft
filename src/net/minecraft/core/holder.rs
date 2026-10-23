//! net/minecraft/core/Holder.java
//!
//! Not ported yet: everything but a reference holder's key. The registries' elements (dimension
//! types, biomes, …) aren't ported, so a holder can't hand out its value, and direct holders
//! (`Holder.direct`, an element sent whole instead of by id), tags and `Holder.Kind` come with
//! their first callers. Java's `Holder<T>` is generic in its element's class, which a key alone
//! doesn't need.
//!
//! `Holder.Reference` doesn't override `hashCode`, so a Java map of holders (`ClientboundSetTimePacket`'s
//! clocks) iterates in another order every run. A holder here hashes its key's identifier, which
//! only decides the order of maps whose order nothing reads.

use crate::java::lang::object::HashCode;
use crate::net::minecraft::resources::resource_key::ResourceKey;

/// An element of a registry, by the key it's registered under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Holder {
    /// `Holder.Reference`: an element registered under its key.
    Reference(ResourceKey),
}

impl Holder {
    /// `unwrapKey()`, which a reference always has.
    pub fn unwrap_key(&self) -> Option<&ResourceKey> {
        match self {
            Holder::Reference(key) => Some(key),
        }
    }

    pub fn is(&self, key: &ResourceKey) -> bool {
        self.unwrap_key() == Some(key)
    }
}

/// Not Java's, which is an identity hash: its key's identifier's, for the maps of holders (above).
impl HashCode for Holder {
    fn hash_code(&self) -> i32 {
        let Holder::Reference(key) = self;
        key.identifier().hash_code()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Holder;
    use crate::golden::{Decode, Encode};
    use crate::net::minecraft::resources::resource_key::ResourceKey;

    // A holder is its key, [registry, identifier].
    impl Decode for Holder {
        fn decode(value: &Value) -> Self {
            Holder::Reference(ResourceKey::decode(value))
        }
    }

    impl Encode for Holder {
        fn encode(&self) -> Value {
            let Holder::Reference(key) = self;
            key.encode()
        }
    }
}
