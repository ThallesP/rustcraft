//! net/minecraft/resources/RegistryOps.java
//!
//! An ops with the registries a codec can look elements up in: what the game protocol's codecs
//! read NBT with (`RegistryAccess.createSerializationContext`). Every `DynamicOps` method forwards
//! to the delegate (`DelegatingOps`), and `getter` looks a registry up. Java's
//! `RegistryInfoLookup` over a `HolderLookup.Provider` is the `RegistryAccess` here, the client's
//! registries, and while the synchronized registries load, also the registries being loaded,
//! whose getters hand out a holder for any key (`HolderGetter::Registration`, Java's
//! `createRegistrationLookup`). The holder codecs ask for a getter through
//! `DynamicOps::as_registry_ops`, Java's `instanceof RegistryOps`. `retrieveGetter`,
//! `retrieveElement` and `withParent` come with the codecs that call them.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsKind, OpsValue};
use crate::com::mojang::serialization::list_builder::ListBuilder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::java::lang::number::Number;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::holder_set::HolderSet;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::resources::delegating_ops::{
    DelegateListBuilder, DelegateRecordBuilder,
};
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::tag_key::TagKey;

pub struct RegistryOps<'a, T> {
    delegate: &'a dyn DynamicOps<T>,
    lookup_provider: &'a RegistryAccess,
    registering: &'a [ResourceKey],
}

impl<'a, T: OpsValue> RegistryOps<'a, T> {
    /// `create(DynamicOps, HolderLookup.Provider)`, which `createSerializationContext` calls.
    pub fn create(
        parent: &'a dyn DynamicOps<T>,
        lookup_provider: &'a RegistryAccess,
    ) -> RegistryOps<'a, T> {
        RegistryOps {
            delegate: parent,
            lookup_provider,
            registering: &[],
        }
    }

    /// `create(DynamicOps, RegistryInfoLookup)` over `RegistryDataLoader.createContext`'s
    /// lookup: the context's registries, and the registries being loaded (`registering`), whose
    /// getters create what they're asked for.
    pub fn create_loading(
        parent: &'a dyn DynamicOps<T>,
        context: &'a RegistryAccess,
        registering: &'a [ResourceKey],
    ) -> RegistryOps<'a, T> {
        RegistryOps {
            delegate: parent,
            lookup_provider: context,
            registering,
        }
    }
}

/// `RegistryOps.RegistryInfoLookup`'s getters, what the holder codecs look elements and tags up in.
pub trait RegistryInfoLookup {
    /// `getter`: the registry's getter, if the ops has it.
    fn getter(&self, registry_key: &ResourceKey) -> Option<HolderGetter<'_>>;
}

impl<T: OpsValue> RegistryInfoLookup for RegistryOps<'_, T> {
    fn getter(&self, registry_key: &ResourceKey) -> Option<HolderGetter<'_>> {
        // createContext puts the registries being loaded over the context's.
        if self.registering.contains(registry_key) {
            return Some(HolderGetter::Registration(registry_key.clone()));
        }
        self.lookup_provider
            .lookup(registry_key)
            .map(HolderGetter::Registry)
    }
}

/// `HolderGetter`: a registry's elements and tags by key.
#[derive(Clone, Debug)]
pub enum HolderGetter<'a> {
    /// A frozen registry's: the elements it has. Its tags are what the server sends
    /// (`update_tags`), which isn't ported, so every tag is a named set of its key.
    Registry(&'a Registry),
    /// `MappedRegistry.createRegistrationLookup()` of a registry being loaded: a holder for any
    /// key and a named set for any tag, bound once the registry is (Java's `freeze` fails on a
    /// holder that never was).
    Registration(ResourceKey),
}

impl HolderGetter<'_> {
    /// `get(ResourceKey)`.
    pub fn get(&self, id: &ResourceKey) -> Option<Holder> {
        match self {
            HolderGetter::Registry(registry) => registry.get(id),
            HolderGetter::Registration(_) => Some(Holder::Reference(id.clone())),
        }
    }

    /// `get(TagKey)`.
    pub fn get_tag(&self, id: &TagKey) -> Option<HolderSet> {
        Some(HolderSet::Named(id.clone()))
    }
}

/// Java's `Object.toString`, an identity hash, which no message prints.
impl<T> fmt::Display for RegistryOps<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RegistryOps")
    }
}

impl<T: OpsValue> DynamicOps<T> for RegistryOps<'_, T> {
    fn kind(&self) -> OpsKind {
        OpsKind::Registry
    }

    fn as_registry_ops(&self) -> Option<&dyn RegistryInfoLookup> {
        Some(self)
    }

    fn delegate_kind(&self) -> OpsKind {
        self.delegate.kind()
    }

    fn empty(&self) -> T {
        self.delegate.empty()
    }

    fn empty_map(&self) -> T {
        self.delegate.empty_map()
    }

    fn empty_list(&self) -> T {
        self.delegate.empty_list()
    }

    fn get_number_value(&self, input: &T) -> DataResult<Number> {
        self.delegate.get_number_value(input)
    }

    fn create_numeric(&self, i: Number) -> T {
        self.delegate.create_numeric(i)
    }

    fn create_byte(&self, value: i8) -> T {
        self.delegate.create_byte(value)
    }

    fn create_short(&self, value: i16) -> T {
        self.delegate.create_short(value)
    }

    fn create_int(&self, value: i32) -> T {
        self.delegate.create_int(value)
    }

    fn create_long(&self, value: i64) -> T {
        self.delegate.create_long(value)
    }

    fn create_float(&self, value: f32) -> T {
        self.delegate.create_float(value)
    }

    fn create_double(&self, value: f64) -> T {
        self.delegate.create_double(value)
    }

    fn get_boolean_value(&self, input: &T) -> DataResult<bool> {
        self.delegate.get_boolean_value(input)
    }

    fn create_boolean(&self, value: bool) -> T {
        self.delegate.create_boolean(value)
    }

    fn get_string_value(&self, input: &T) -> DataResult<String> {
        self.delegate.get_string_value(input)
    }

    fn create_string(&self, value: &str) -> T {
        self.delegate.create_string(value)
    }

    fn merge_to_list(&self, list: &T, value: T) -> DataResult<T> {
        self.delegate.merge_to_list(list, value)
    }

    fn merge_to_list_values(&self, list: &T, values: Vec<T>) -> DataResult<T> {
        self.delegate.merge_to_list_values(list, values)
    }

    fn merge_to_map(&self, map: &T, key: T, value: T) -> DataResult<T> {
        self.delegate.merge_to_map(map, key, value)
    }

    fn merge_to_map_values(&self, map: &T, values: Vec<Pair<T, T>>) -> DataResult<T> {
        self.delegate.merge_to_map_values(map, values)
    }

    fn merge_to_map_like(&self, map: &T, values: &dyn MapLike<T>) -> DataResult<T> {
        self.delegate.merge_to_map_like(map, values)
    }

    fn merge_to_primitive(&self, prefix: &T, value: T) -> DataResult<T> {
        self.delegate.merge_to_primitive(prefix, value)
    }

    fn get_map_values(&self, input: &T) -> DataResult<Vec<Pair<T, T>>> {
        self.delegate.get_map_values(input)
    }

    fn get_map_entries(&self, input: &T) -> DataResult<Vec<Pair<T, T>>> {
        self.delegate.get_map_entries(input)
    }

    fn create_map(&self, map: Vec<Pair<T, T>>) -> T {
        self.delegate.create_map(map)
    }

    fn get_map<'b>(&self, input: &'b T) -> DataResult<Box<dyn MapLike<T> + 'b>> {
        self.delegate.get_map(input)
    }

    fn get_stream(&self, input: &T) -> DataResult<Vec<T>> {
        self.delegate.get_stream(input)
    }

    fn get_list(&self, input: &T) -> DataResult<Vec<T>> {
        self.delegate.get_list(input)
    }

    fn create_list(&self, input: Vec<T>) -> T {
        self.delegate.create_list(input)
    }

    fn get_byte_buffer(&self, input: &T) -> DataResult<Vec<i8>> {
        self.delegate.get_byte_buffer(input)
    }

    fn create_byte_list(&self, input: &[i8]) -> T {
        self.delegate.create_byte_list(input)
    }

    fn get_int_stream(&self, input: &T) -> DataResult<Vec<i32>> {
        self.delegate.get_int_stream(input)
    }

    fn create_int_list(&self, input: &[i32]) -> T {
        self.delegate.create_int_list(input)
    }

    fn get_long_stream(&self, input: &T) -> DataResult<Vec<i64>> {
        self.delegate.get_long_stream(input)
    }

    fn create_long_list(&self, input: &[i64]) -> T {
        self.delegate.create_long_list(input)
    }

    fn remove(&self, input: &T, key: &str) -> T {
        self.delegate.remove(input, key)
    }

    fn compress_maps(&self) -> bool {
        self.delegate.compress_maps()
    }

    fn list_builder(&self) -> Box<dyn ListBuilder<T> + '_> {
        Box::new(DelegateListBuilder {
            original: self.delegate.list_builder(),
            ops: self,
        })
    }

    fn map_builder(&self) -> Box<dyn RecordBuilder<T> + '_> {
        Box::new(DelegateRecordBuilder {
            original: self.delegate.map_builder(),
            ops: self,
        })
    }
}
