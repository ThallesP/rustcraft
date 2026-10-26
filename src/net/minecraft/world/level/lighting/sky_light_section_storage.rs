//! net/minecraft/world/level/lighting/SkyLightSectionStorage.java
//!
//! The sky's light, by section. Above a column's highest stored section the sky is full (15);
//! below it, a section with no layer of its own shows the lowest layer of the next stored one up.
//! Its map keeps each column's top (one above its highest stored section) and the lowest section
//! stored anywhere, which only goes down.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::hash_common::BuildMixHasher;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::data_layer_storage_map::{
    DataLayerStorageMap, DataLayerStorageMapClass, SharedDataLayer,
};
use crate::net::minecraft::world::level::lighting::layer_light_event_listener::SectionLight;
use crate::net::minecraft::world::level::lighting::layer_light_section_storage::{
    LayerLightSectionStorage, LayerLightSectionStorageClass,
};

/// The sky's light, by section.
pub struct SkyLightSectionStorage {
    base: LayerLightSectionStorage<SkyDataLayerStorageMap>,
}

impl SkyLightSectionStorage {
    pub(super) fn new() -> SkyLightSectionStorage {
        SkyLightSectionStorage {
            base: LayerLightSectionStorage::new(
                LightLayer::SKY,
                SkyDataLayerStorageMap::new(
                    DataLayerStorageMap::default(),
                    HashMap::default(),
                    i32::MAX,
                ),
            ),
        }
    }

    /// `getLightValue(long, boolean)`: the sky's light at a block, in the layers being worked on
    /// or the visible ones.
    pub(super) fn get_light_value_updating(&self, mut block_node: i64, updating: bool) -> i32 {
        let mut section_node = SectionPos::block_to_section(block_node);
        let mut section_y = SectionPos::y_of(section_node);
        let sections = if updating {
            &self.updating_section_data
        } else {
            &self.visible_section_data
        };
        let top_section = sections.top_section(SectionPos::get_zero_node(section_node));
        if top_section != sections.current_lowest_y && section_y < top_section {
            let mut layer = LayerLightSectionStorage::get_data_layer_in(sections, section_node);
            if layer.is_none() {
                // The bottom of the next layer up.
                block_node = BlockPos::get_flat_index(block_node);

                while layer.is_none() {
                    section_y += 1;
                    if section_y >= top_section {
                        return 15;
                    }

                    section_node = SectionPos::offset_i64(section_node, Direction::UP);
                    layer = LayerLightSectionStorage::get_data_layer_in(sections, section_node);
                }
            }

            layer.expect("found above").borrow().get(
                SectionPos::section_relative(BlockPos::get_x_i64(block_node)),
                SectionPos::section_relative(BlockPos::get_y_i64(block_node)),
                SectionPos::section_relative(BlockPos::get_z_i64(block_node)),
            )
        } else if updating && !self.light_on_in_section(section_node) {
            0
        } else {
            15
        }
    }

    /// What `get_light_value` reads in the section's visible layers: its layer below its
    /// column's top, the bottom of the next one up where it stores none, and full above the top.
    fn copy_visible_section_light(&self, mut section_node: i64) -> SectionLight {
        let sections = &self.visible_section_data;
        let mut section_y = SectionPos::y_of(section_node);
        let top_section = sections.top_section(SectionPos::get_zero_node(section_node));
        if top_section == sections.current_lowest_y || section_y >= top_section {
            return SectionLight::Uniform(15);
        }

        if let Some(layer) = LayerLightSectionStorage::get_data_layer_in(sections, section_node) {
            return SectionLight::Layer(Arc::new(layer.borrow().copy()));
        }
        loop {
            section_y += 1;
            if section_y >= top_section {
                return SectionLight::Uniform(15);
            }

            section_node = SectionPos::offset_i64(section_node, Direction::UP);
            if let Some(layer) = LayerLightSectionStorage::get_data_layer_in(sections, section_node)
            {
                return SectionLight::BottomOf(Arc::new(layer.borrow().copy()));
            }
        }
    }

    /// The layer of a section below a column's top: the bottom of the next stored layer up,
    /// repeated.
    fn repeat_first_layer(data: &DataLayer) -> DataLayer {
        if data.is_definitely_homogenous() {
            return data.copy();
        }

        let input = data
            .raw_data()
            .expect("a layer that isn't homogenous has its bytes");
        let mut output = [0i8; 2048];

        for i in 0..16 {
            output[i * 128..(i + 1) * 128].copy_from_slice(&input[..128]);
        }

        DataLayer::from_data(&output)
    }

    pub(super) fn has_light_data_at_or_below(&self, section_y: i32) -> bool {
        section_y >= self.updating_section_data.current_lowest_y
    }

    pub(super) fn is_above_data(&self, section_node: i64) -> bool {
        let zero_node = SectionPos::get_zero_node(section_node);
        let top_section = self.updating_section_data.top_section(zero_node);
        top_section == self.updating_section_data.current_lowest_y
            || SectionPos::y_of(section_node) >= top_section
    }

    pub(super) fn get_top_section_y(&self, zero_node: i64) -> i32 {
        self.updating_section_data.top_section(zero_node)
    }

    pub(super) fn get_bottom_section_y(&self) -> i32 {
        self.updating_section_data.current_lowest_y
    }
}

impl LayerLightSectionStorageClass for SkyLightSectionStorage {
    type Map = SkyDataLayerStorageMap;

    fn get_light_value(&self, block_node: i64) -> i32 {
        self.get_light_value_updating(block_node, false)
    }

    fn copy_section_light(&self, section_node: i64) -> SectionLight {
        self.copy_visible_section_light(section_node)
    }

    /// A column's top rises to the section, and the lowest section falls to it.
    fn on_node_added(&mut self, section_node: i64) {
        let y = SectionPos::y_of(section_node);
        if self.updating_section_data.current_lowest_y > y {
            // Its map's default for a column without a top goes with it.
            self.updating_section_data.current_lowest_y = y;
        }

        let zero_node = SectionPos::get_zero_node(section_node);
        let old_top = self.updating_section_data.top_section(zero_node);
        if old_top < y + 1 {
            self.updating_section_data
                .top_sections
                .insert(zero_node, y + 1);
        }
    }

    /// A column's top falls to its next stored section down, or goes.
    fn on_node_removed(&mut self, section_node: i64) {
        let zero_node = SectionPos::get_zero_node(section_node);
        let mut y = SectionPos::y_of(section_node);
        if self.updating_section_data.top_section(zero_node) == y + 1 {
            let mut new_top_section = section_node;
            while !self.storing_light_for_section(new_top_section)
                && self.has_light_data_at_or_below(y)
            {
                y -= 1;
                new_top_section = SectionPos::offset_i64(new_top_section, Direction::DOWN);
            }

            if self.storing_light_for_section(new_top_section) {
                self.updating_section_data
                    .top_sections
                    .insert(zero_node, y + 1);
            } else {
                self.updating_section_data.top_sections.remove(&zero_node);
            }
        }
    }

    /// The queued layer, else below a column's top the bottom of the next layer up, else full or
    /// dark as the column's light is on or off.
    fn create_data_layer(&self, section_node: i64) -> SharedDataLayer {
        if let Some(queued_layer) = self.queued_sections.get(&section_node) {
            return Rc::clone(queued_layer);
        }

        let top_section = self
            .updating_section_data
            .top_section(SectionPos::get_zero_node(section_node));
        if top_section != self.updating_section_data.current_lowest_y
            && SectionPos::y_of(section_node) < top_section
        {
            let mut above_section = SectionPos::offset_i64(section_node, Direction::UP);

            let above_data = loop {
                match self.get_data_layer(above_section, true) {
                    Some(above_data) => break above_data,
                    None => above_section = SectionPos::offset_i64(above_section, Direction::UP),
                }
            };

            Rc::new(RefCell::new(SkyLightSectionStorage::repeat_first_layer(
                &above_data.borrow(),
            )))
        } else if self.light_on_in_section(section_node) {
            Rc::new(RefCell::new(DataLayer::with_default(15)))
        } else {
            Rc::new(RefCell::new(DataLayer::new()))
        }
    }
}

impl Deref for SkyLightSectionStorage {
    type Target = LayerLightSectionStorage<SkyDataLayerStorageMap>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for SkyLightSectionStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// `SkyLightSectionStorage.SkyDataLayerStorageMap`: the layers, each column's top section, and
/// the lowest section stored, which a column without a top answers (Java's
/// `defaultReturnValue`).
#[derive(Clone)]
pub struct SkyDataLayerStorageMap {
    base: DataLayerStorageMap,
    current_lowest_y: i32,
    top_sections: HashMap<i64, i32, BuildMixHasher>,
}

impl SkyDataLayerStorageMap {
    fn new(
        map: DataLayerStorageMap,
        top_sections: HashMap<i64, i32, BuildMixHasher>,
        current_lowest_y: i32,
    ) -> SkyDataLayerStorageMap {
        SkyDataLayerStorageMap {
            base: map,
            current_lowest_y,
            top_sections,
        }
    }

    /// `topSections.get(zeroNode)`.
    fn top_section(&self, zero_node: i64) -> i32 {
        self.top_sections
            .get(&zero_node)
            .copied()
            .unwrap_or(self.current_lowest_y)
    }
}

impl DataLayerStorageMapClass for SkyDataLayerStorageMap {
    fn copy(&self) -> SkyDataLayerStorageMap {
        self.clone()
    }
}

impl Deref for SkyDataLayerStorageMap {
    type Target = DataLayerStorageMap;

    fn deref(&self) -> &DataLayerStorageMap {
        &self.base
    }
}

impl DerefMut for SkyDataLayerStorageMap {
    fn deref_mut(&mut self) -> &mut DataLayerStorageMap {
        &mut self.base
    }
}
