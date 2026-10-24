//! net/minecraft/world/level/lighting/DataLayerStorageMap.java
//!
//! A light engine's layers, by section node. Java's abstract class is generic in its subclass,
//! which `copy` makes; here the subclasses (`BlockDataLayerStorageMap`, `SkyDataLayerStorageMap`)
//! hold this map, deref to it and implement `DataLayerStorageMapClass`. The layers are shared like
//! Java's objects, `Rc<RefCell<DataLayer>>`: `copy` (Java's `map.clone()`) hands the new map the
//! same layers, which the storage copies before it first writes to one it may share
//! (`changedSections`), and the storage compares them by reference. Its two-entry cache of the
//! last layers asked for isn't ported: Java clears it whenever a cached key could change, so
//! asking the map gives the same layers. The map hashes its keys with fastutil's `mix`, like
//! Java's `Long2ObjectOpenHashMap`: nothing iterates it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::it::unimi::dsi::fastutil::hash_common::BuildMixHasher;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;

/// A layer as the maps and the queue hold it, shared like Java's references.
pub type SharedDataLayer = Rc<RefCell<DataLayer>>;

/// The layers by section node.
#[derive(Clone, Default)]
pub struct DataLayerStorageMap {
    map: HashMap<i64, SharedDataLayer, BuildMixHasher>,
}

impl DataLayerStorageMap {
    /// A copy of the node's layer in its place. Panics when it has none, where Java throws
    /// `NullPointerException`.
    pub fn copy_data_layer(&mut self, section_node: i64) -> SharedDataLayer {
        let new_data_layer = Rc::new(RefCell::new(
            self.map
                .get(&section_node)
                .unwrap_or_else(|| panic!("no layer at {section_node} to copy"))
                .borrow()
                .copy(),
        ));
        self.map.insert(section_node, Rc::clone(&new_data_layer));
        new_data_layer
    }

    pub fn has_layer(&self, section_node: i64) -> bool {
        self.map.contains_key(&section_node)
    }

    pub fn get_layer(&self, section_node: i64) -> Option<&SharedDataLayer> {
        self.map.get(&section_node)
    }

    pub fn remove_layer(&mut self, section_node: i64) -> Option<SharedDataLayer> {
        self.map.remove(&section_node)
    }

    pub fn set_layer(&mut self, section_node: i64, layer: SharedDataLayer) {
        self.map.insert(section_node, layer);
    }
}

/// What a subclass adds: `copy`, a map sharing its layers.
pub trait DataLayerStorageMapClass: Deref<Target = DataLayerStorageMap> + DerefMut {
    fn copy(&self) -> Self;
}
