//! net/minecraft/core/Registry.java
//!
//! Not ported yet: nearly all of it. Java's `Registry` is an interface, whose registries the
//! client gets from the server are `MappedRegistry`s; the port keeps what reading elements by
//! their network id needs, `asHolderIdMap()`'s `byIdOrThrow` and `getIdOrThrow`: the registry's
//! key, and its elements' keys in id order, which is the order the server sent them in
//! (`RegistryDataLoader.loadContentsFromNetwork` registers them as they come). The elements'
//! values come with the registries' classes: a registry whose class is ported holds its values in
//! id order (`with_values`, `RegistryValues`), which `value` hands out for a holder
//! (`Holder.value()`). Java's registries compare by reference; here two registries with values
//! are the same when they share them. An element's id is found by its key in a map, like
//! `MappedRegistry`'s `toId`, which the tints ask 25 times a block.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::network::chat::chat_type::ChatType;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::damagesource::damage_type::DamageType;
use crate::net::minecraft::world::entity::animal::wolf::wolf_variant::WolfVariant;
use crate::net::minecraft::world::entity::decoration::painting::painting_variant::PaintingVariant;
use crate::net::minecraft::world::level::biome::biome::Biome;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;

/// A registry's elements, by network id.
#[derive(Clone, Debug, PartialEq)]
pub struct Registry {
    key: ResourceKey,
    by_id: Vec<ResourceKey>,
    to_id: KeyIds,
    values: Option<RegistryValues>,
}

/// Each element's id by its key, the first for a key given twice: what `by_id` says, so it
/// prints as nothing and compares equal.
#[derive(Clone, Default)]
struct KeyIds(HashMap<ResourceKey, i32>);

impl KeyIds {
    fn of(by_id: &[ResourceKey]) -> KeyIds {
        let mut ids = HashMap::with_capacity(by_id.len());
        for (id, key) in by_id.iter().enumerate() {
            ids.entry(key.clone()).or_insert(id as i32);
        }
        KeyIds(ids)
    }
}

impl std::fmt::Debug for KeyIds {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("..")
    }
}

impl PartialEq for KeyIds {
    fn eq(&self, _other: &KeyIds) -> bool {
        true
    }
}

/// The values of a registry's elements in id order, for the registries whose classes are ported.
#[derive(Clone, Debug)]
pub enum RegistryValues {
    Biome(Arc<Vec<Biome>>),
    DimensionType(Arc<Vec<DimensionType>>),
    DamageType(Arc<Vec<DamageType>>),
    ChatType(Arc<Vec<ChatType>>),
    PaintingVariant(Arc<Vec<PaintingVariant>>),
    WolfVariant(Arc<Vec<WolfVariant>>),
}

/// Java's `Object.equals` of registries: the same values.
impl PartialEq for RegistryValues {
    fn eq(&self, other: &RegistryValues) -> bool {
        match (self, other) {
            (RegistryValues::Biome(a), RegistryValues::Biome(b)) => Arc::ptr_eq(a, b),
            (RegistryValues::DimensionType(a), RegistryValues::DimensionType(b)) => {
                Arc::ptr_eq(a, b)
            }
            (RegistryValues::DamageType(a), RegistryValues::DamageType(b)) => Arc::ptr_eq(a, b),
            (RegistryValues::ChatType(a), RegistryValues::ChatType(b)) => Arc::ptr_eq(a, b),
            (RegistryValues::PaintingVariant(a), RegistryValues::PaintingVariant(b)) => {
                Arc::ptr_eq(a, b)
            }
            (RegistryValues::WolfVariant(a), RegistryValues::WolfVariant(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// A class a registry holds values of: Java's `T` in `Registry<T>`.
pub trait RegistryValue: Sized {
    /// The values, if the registry holds this class's.
    fn of(values: &RegistryValues) -> Option<&[Self]>;

    /// A registry's values of this class.
    fn wrap(values: Vec<Self>) -> RegistryValues;
}

macro_rules! registry_value {
    ($($class:ident),*) => {
        $(
            impl RegistryValue for $class {
                fn of(values: &RegistryValues) -> Option<&[$class]> {
                    match values {
                        RegistryValues::$class(values) => Some(values),
                        _ => None,
                    }
                }

                fn wrap(values: Vec<$class>) -> RegistryValues {
                    RegistryValues::$class(Arc::new(values))
                }
            }
        )*
    };
}

registry_value!(
    Biome,
    DimensionType,
    DamageType,
    ChatType,
    PaintingVariant,
    WolfVariant
);

impl Registry {
    pub fn new(key: ResourceKey, by_id: Vec<ResourceKey>) -> Registry {
        Registry {
            key,
            to_id: KeyIds::of(&by_id),
            by_id,
            values: None,
        }
    }

    /// A registry with its elements' values in id order.
    pub fn with_values(
        key: ResourceKey,
        by_id: Vec<ResourceKey>,
        values: RegistryValues,
    ) -> Registry {
        let count = match &values {
            RegistryValues::Biome(values) => values.len(),
            RegistryValues::DimensionType(values) => values.len(),
            RegistryValues::DamageType(values) => values.len(),
            RegistryValues::ChatType(values) => values.len(),
            RegistryValues::PaintingVariant(values) => values.len(),
            RegistryValues::WolfVariant(values) => values.len(),
        };
        assert_eq!(by_id.len(), count, "a value for each element");
        Registry {
            key,
            to_id: KeyIds::of(&by_id),
            by_id,
            values: Some(values),
        }
    }

    /// `Holder.value()` of one of its elements. Panics for a registry without values of the
    /// class, or a holder it doesn't have.
    pub fn value<T: RegistryValue>(&self, holder: &Holder) -> &T {
        let values = self.values.as_ref().and_then(T::of).unwrap_or_else(|| {
            panic!(
                "{} holds no {}",
                self.key,
                std::any::type_name::<T>()
                    .rsplit("::")
                    .next()
                    .unwrap_or("values")
            )
        });
        &values[self.holder_id_or_throw(holder) as usize]
    }

    /// Its values in id order, if it holds the class's.
    pub fn values<T: RegistryValue>(&self) -> Option<&[T]> {
        self.values.as_ref().and_then(T::of)
    }

    /// `get(ResourceKey)`: the element's holder, if it has the key.
    pub fn get(&self, key: &ResourceKey) -> Option<Holder> {
        self.to_id
            .0
            .contains_key(key)
            .then(|| Holder::Reference(key.clone()))
    }

    /// `getOrThrow(key)`'s holder: panics for a key it doesn't have, where Java throws
    /// `IllegalStateException`.
    pub fn get_or_throw(&self, key: &ResourceKey) -> Holder {
        if self.to_id.0.contains_key(key) {
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
        match self.to_id.0.get(key) {
            Some(&id) => id,
            None => panic!("Can't find id for '{key}' in map {}", self.key),
        }
    }
}

/// `asHolderIdMap()`: its elements' holders by network id, which a biome container's palette
/// numbers its biomes with.
impl IdMap<Holder> for Registry {
    fn get_id(&self, thing: &Holder) -> i32 {
        let Holder::Reference(key) = thing;
        self.to_id.0.get(key).map_or(-1, |&id| id)
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
