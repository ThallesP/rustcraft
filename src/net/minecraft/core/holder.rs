//! net/minecraft/core/Holder.java
//!
//! Not ported yet: everything but a reference holder's key. The registries' elements (dimension
//! types, biomes, …) aren't ported, so a holder can't hand out its value, and direct holders
//! (`Holder.direct`, an element sent whole instead of by id), tags and `Holder.Kind` come with
//! their first callers. Java's `Holder<T>` is generic in its element's class, which a key alone
//! doesn't need.

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
