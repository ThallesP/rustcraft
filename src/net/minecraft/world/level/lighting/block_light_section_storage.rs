//! net/minecraft/world/level/lighting/BlockLightSectionStorage.java

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::data_layer_storage_map::{
    DataLayerStorageMap, DataLayerStorageMapClass,
};
use crate::net::minecraft::world::level::lighting::layer_light_event_listener::SectionLight;
use crate::net::minecraft::world::level::lighting::layer_light_section_storage::{
    LayerLightSectionStorage, LayerLightSectionStorageClass,
};

/// The blocks' light, by section.
pub struct BlockLightSectionStorage {
    base: LayerLightSectionStorage<BlockDataLayerStorageMap>,
}

impl BlockLightSectionStorage {
    pub(super) fn new() -> BlockLightSectionStorage {
        BlockLightSectionStorage {
            base: LayerLightSectionStorage::new(
                LightLayer::BLOCK,
                BlockDataLayerStorageMap(DataLayerStorageMap::default()),
            ),
        }
    }
}

impl LayerLightSectionStorageClass for BlockLightSectionStorage {
    type Map = BlockDataLayerStorageMap;

    /// The visible layer's value, or 0 in a section without one.
    fn get_light_value(&self, block_node: i64) -> i32 {
        let section_node = SectionPos::block_to_section(block_node);
        match self.get_data_layer(section_node, false) {
            None => 0,
            Some(layer) => layer.borrow().get(
                SectionPos::section_relative(BlockPos::get_x_i64(block_node)),
                SectionPos::section_relative(BlockPos::get_y_i64(block_node)),
                SectionPos::section_relative(BlockPos::get_z_i64(block_node)),
            ),
        }
    }
    /// The visible layer, or 0 everywhere in a section without one.
    fn copy_section_light(&self, section_node: i64) -> SectionLight {
        match self.get_data_layer(section_node, false) {
            None => SectionLight::Uniform(0),
            Some(layer) => SectionLight::Layer(Arc::new(layer.borrow().copy())),
        }
    }
}

impl Deref for BlockLightSectionStorage {
    type Target = LayerLightSectionStorage<BlockDataLayerStorageMap>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BlockLightSectionStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// `BlockLightSectionStorage.BlockDataLayerStorageMap`: the map, nothing added.
#[derive(Clone)]
pub struct BlockDataLayerStorageMap(DataLayerStorageMap);

impl DataLayerStorageMapClass for BlockDataLayerStorageMap {
    fn copy(&self) -> BlockDataLayerStorageMap {
        self.clone()
    }
}

impl Deref for BlockDataLayerStorageMap {
    type Target = DataLayerStorageMap;

    fn deref(&self) -> &DataLayerStorageMap {
        &self.0
    }
}

impl DerefMut for BlockDataLayerStorageMap {
    fn deref_mut(&mut self) -> &mut DataLayerStorageMap {
        &mut self.0
    }
}
