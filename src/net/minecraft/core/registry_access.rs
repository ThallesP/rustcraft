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

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::{Arc, LazyLock};

    use serde_json::Value;

    use super::RegistryAccess;
    use crate::golden::{Decode, Golden};
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::core::registry::Registry;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;

    static GOLDEN: LazyLock<Arc<RegistryAccess>> = LazyLock::new(|| {
        let golden = Golden::load("net.minecraft.core.RegistryAccess");
        let mut registries = BuiltInRegistries::registries();
        registries.extend(synchronized(&golden.input("synchronized")));
        Arc::new(RegistryAccess::new(registries))
    });

    /// The registries the golden harness's `RegistryFriendlyByteBuf`s carry (TESTING.md): the
    /// client's built-in ones the port has, and the synchronized ones as vanilla's server sends
    /// them, from the golden file.
    pub(crate) fn golden() -> Arc<RegistryAccess> {
        Arc::clone(&GOLDEN)
    }

    // [[registry, [identifier, …]], …], each registry's elements in id order.
    fn synchronized(value: &Value) -> Vec<Registry> {
        Vec::<(Identifier, Vec<Identifier>)>::decode(value)
            .into_iter()
            .map(|(name, elements)| {
                let key = ResourceKey::create_registry_key(name);
                let keys = elements
                    .into_iter()
                    .map(|element| ResourceKey::create(&key, element))
                    .collect();
                Registry::new(key, keys)
            })
            .collect()
    }

    fn identifiers(registry: &Registry) -> Vec<Identifier> {
        (0..registry.size())
            .map(|id| {
                registry
                    .holder_by_id_or_throw(id)
                    .unwrap_key()
                    .expect("a reference has its key")
                    .identifier()
                    .clone()
            })
            .collect()
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.RegistryAccess");
        // The harness's input, which every RegistryFriendlyByteBuf case reads with.
        assert!(!synchronized(&golden.input("synchronized")).is_empty());
        golden.field(
            "minecraft:attribute",
            identifiers(BuiltInRegistries::ATTRIBUTE),
        );
        golden.finish();
    }
}
