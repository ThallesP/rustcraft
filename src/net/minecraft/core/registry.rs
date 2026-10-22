//! net/minecraft/core/Registry.java
//!
//! Not ported yet: nearly all of it. Java's `Registry` is an interface, whose registries the
//! client gets from the server are `MappedRegistry`s; the port keeps what reading elements by
//! their network id needs, `asHolderIdMap()`'s `byIdOrThrow` and `getIdOrThrow`: the registry's
//! key, and its elements' keys in id order, which is the order the server sent them in
//! (`RegistryDataLoader.loadContentsFromNetwork` registers them as they come). The elements'
//! values come with the registries' classes (`DimensionType`, `Biome`, …).

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::resources::resource_key::ResourceKey;

/// A registry's elements, by network id.
#[derive(Clone, Debug, PartialEq)]
pub struct Registry {
    key: ResourceKey,
    by_id: Vec<ResourceKey>,
}

impl Registry {
    pub fn new(key: ResourceKey, by_id: Vec<ResourceKey>) -> Registry {
        Registry { key, by_id }
    }

    pub fn key(&self) -> &ResourceKey {
        &self.key
    }

    pub fn size(&self) -> i32 {
        self.by_id.len() as i32
    }

    /// `asHolderIdMap().byIdOrThrow(id)`: panics for an id it doesn't have, where Java throws
    /// `IllegalArgumentException`.
    pub fn holder_by_id_or_throw(&self, id: i32) -> Holder {
        match usize::try_from(id)
            .ok()
            .and_then(|index| self.by_id.get(index))
        {
            Some(key) => Holder::Reference(key.clone()),
            None => panic!("No value with id {id}"),
        }
    }

    /// `asHolderIdMap().getIdOrThrow(holder)`.
    pub fn holder_id_or_throw(&self, holder: &Holder) -> i32 {
        let Holder::Reference(key) = holder;
        match self.by_id.iter().position(|element| element == key) {
            Some(id) => id as i32,
            None => panic!("Can't find id for '{key}' in map {}", self.key),
        }
    }
}
