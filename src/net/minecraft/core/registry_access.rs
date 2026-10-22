//! net/minecraft/core/RegistryAccess.java
//!
//! Not ported yet: nearly all of it, with `Registry`. What the client builds from the server's
//! registry data (`RegistryDataCollector.collectGameRegistries`, a `RegistryAccess.Frozen`) is
//! the synchronized registries, each its elements' keys in id order, which is what the game
//! protocol's codecs need to read an element by id. The static registries (blocks, items, entity
//! types), which the client has built in, come with their classes.

use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::resources::resource_key::ResourceKey;

/// The registries a connection reads elements of, by their keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegistryAccess {
    registries: Vec<Registry>,
}

impl RegistryAccess {
    /// `ImmutableRegistryAccess`'s, frozen.
    pub fn new(registries: Vec<Registry>) -> RegistryAccess {
        RegistryAccess { registries }
    }

    /// Panics for a registry it doesn't have, where Java throws `IllegalStateException`.
    pub fn lookup_or_throw(&self, key: &ResourceKey) -> &Registry {
        self.lookup(key)
            .unwrap_or_else(|| panic!("Missing registry: {key}"))
    }

    pub fn lookup(&self, key: &ResourceKey) -> Option<&Registry> {
        self.registries
            .iter()
            .find(|registry| registry.key() == key)
    }

    /// `listRegistryKeys()`.
    pub fn list_registry_keys(&self) -> impl Iterator<Item = &ResourceKey> {
        self.registries.iter().map(Registry::key)
    }
}
