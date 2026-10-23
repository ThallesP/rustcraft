//! net/minecraft/core/Registry.java
//!
//! Not ported yet: nearly all of it. Java's `Registry` is an interface, whose registries the
//! client gets from the server are `MappedRegistry`s; the port keeps what reading elements by
//! their network id needs, `asHolderIdMap()`'s `byIdOrThrow` and `getIdOrThrow`: the registry's
//! key, and its elements' keys in id order, which is the order the server sent them in
//! (`RegistryDataLoader.loadContentsFromNetwork` registers them as they come). The elements'
//! values come with the registries' classes: so far the biomes' (`Biome`), which the biome
//! registry holds in id order (`with_biomes`), and which `value_biome` hands out for a holder.

use std::sync::Arc;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::biome::biome::Biome;

/// A registry's elements, by network id.
#[derive(Clone, Debug, PartialEq)]
pub struct Registry {
    key: ResourceKey,
    by_id: Vec<ResourceKey>,
    biomes: Option<Arc<Vec<Biome>>>,
}

impl Registry {
    pub fn new(key: ResourceKey, by_id: Vec<ResourceKey>) -> Registry {
        Registry {
            key,
            by_id,
            biomes: None,
        }
    }

    /// The biome registry, with its elements' values in id order.
    pub fn with_biomes(key: ResourceKey, by_id: Vec<ResourceKey>, biomes: Vec<Biome>) -> Registry {
        assert_eq!(by_id.len(), biomes.len(), "a value for each element");
        Registry {
            key,
            by_id,
            biomes: Some(Arc::new(biomes)),
        }
    }

    /// `Holder.value()` of one of its biomes. Panics for a registry without them, or a holder
    /// it doesn't have.
    pub fn value_biome(&self, holder: &Holder) -> &Biome {
        let biomes = self
            .biomes
            .as_ref()
            .unwrap_or_else(|| panic!("{} holds no biomes", self.key));
        &biomes[self.holder_id_or_throw(holder) as usize]
    }

    /// `getOrThrow(key)`'s holder: panics for a key it doesn't have, where Java throws
    /// `IllegalStateException`.
    pub fn get_or_throw(&self, key: &ResourceKey) -> Holder {
        if self.by_id.contains(key) {
            Holder::Reference(key.clone())
        } else {
            panic!("Missing element {key}")
        }
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

/// `asHolderIdMap()`: its elements' holders by network id, which a biome container's palette
/// numbers its biomes with.
impl IdMap<Holder> for Registry {
    fn get_id(&self, thing: &Holder) -> i32 {
        let Holder::Reference(key) = thing;
        self.by_id
            .iter()
            .position(|element| element == key)
            .map_or(-1, |id| id as i32)
    }

    fn by_id(&self, id: i32) -> Option<Holder> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.by_id.get(index))
            .map(|key| Holder::Reference(key.clone()))
    }

    fn size(&self) -> i32 {
        Registry::size(self)
    }
}
