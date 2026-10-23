//! net/minecraft/world/level/lighting/LayerLightSectionStorage.java
//!
//! One light layer's sections: which the engine stores light for (each section's state: whether
//! it has blocks, and how many of its 26 neighbours do), the layers being worked on
//! (`updatingSectionData`) and the ones the game reads (`visibleSectionData`, swapped in at the
//! end of each `runLightUpdates`), and the layers the server sent, queued until their sections
//! are stored.
//!
//! Java's abstract class is a struct here, generic in its map, and its virtual methods
//! (`getLightValue`, `onNodeAdded`, `onNodeRemoved`, `createDataLayer`) are the
//! `LayerLightSectionStorageClass` trait the block light's and the sky light's storages
//! implement; the methods that call them are functions of this module taking the storage
//! (`update_section_status`, `mark_new_inconsistencies`). fastutil's hash sets and maps are std's:
//! nothing reads their order but `markNewInconsistencies`, whose loops come out the same in any
//! order (each node's step reads no other node's, and the sky's `onNodeRemoved` runs after every
//! removal), and the calls to `onLightUpdate`, which the order of only changes when the renderer
//! hears of each section. Those are kept sorted. The `volatile` fields and the synchronized queue
//! (Java's light runs off the game thread on the server) are plain fields.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::data_layer_storage_map::{
    DataLayerStorageMapClass, SharedDataLayer,
};

/// What a layer's storage keeps of its sections.
pub struct LayerLightSectionStorage<M> {
    layer: LightLayer,
    section_states: HashMap<i64, i8>,
    columns_with_sources: HashSet<i64>,
    pub(super) visible_section_data: M,
    pub(super) updating_section_data: M,
    pub(super) changed_sections: HashSet<i64>,
    sections_affected_by_light_updates: BTreeSet<i64>,
    pub(super) queued_sections: BTreeMap<i64, SharedDataLayer>,
    columns_to_retain_queued_data_for: HashSet<i64>,
    to_remove: BTreeSet<i64>,
    has_inconsistencies: bool,
}

impl<M: DataLayerStorageMapClass> LayerLightSectionStorage<M> {
    pub(super) fn new(layer: LightLayer, initial_map: M) -> LayerLightSectionStorage<M> {
        LayerLightSectionStorage {
            layer,
            section_states: HashMap::new(),
            columns_with_sources: HashSet::new(),
            visible_section_data: initial_map.copy(),
            updating_section_data: initial_map,
            changed_sections: HashSet::new(),
            sections_affected_by_light_updates: BTreeSet::new(),
            queued_sections: BTreeMap::new(),
            columns_to_retain_queued_data_for: HashSet::new(),
            to_remove: BTreeSet::new(),
            has_inconsistencies: false,
        }
    }

    pub(super) fn storing_light_for_section(&self, section_node: i64) -> bool {
        self.get_data_layer(section_node, true).is_some()
    }

    pub(super) fn get_data_layer(
        &self,
        section_node: i64,
        updating: bool,
    ) -> Option<&SharedDataLayer> {
        LayerLightSectionStorage::get_data_layer_in(
            if updating {
                &self.updating_section_data
            } else {
                &self.visible_section_data
            },
            section_node,
        )
    }

    /// `getDataLayer(M, long)`.
    pub(super) fn get_data_layer_in(sections: &M, section_node: i64) -> Option<&SharedDataLayer> {
        sections.get_layer(section_node)
    }

    /// The section's layer to write to, copied first the first time since the last swap, so the
    /// visible map keeps the one it shares.
    pub(super) fn get_data_layer_to_write(&mut self, section_node: i64) -> Option<SharedDataLayer> {
        let data_layer = Rc::clone(self.updating_section_data.get_layer(section_node)?);
        if self.changed_sections.insert(section_node) {
            let data_layer = Rc::new(RefCell::new(data_layer.borrow().copy()));
            self.updating_section_data
                .set_layer(section_node, Rc::clone(&data_layer));
            return Some(data_layer);
        }

        Some(data_layer)
    }

    /// The section's layer as queued, or as the game last saw it. A copy, where Java hands out the
    /// layer.
    pub fn get_data_layer_data(&self, section_node: i64) -> Option<DataLayer> {
        self.queued_sections
            .get(&section_node)
            .or_else(|| self.get_data_layer(section_node, false))
            .map(|layer| layer.borrow().copy())
    }

    /// The light being worked on at a block. Panics in a section it doesn't store, where Java
    /// throws `NullPointerException`.
    pub(super) fn get_stored_level(&self, block_node: i64) -> i32 {
        let section_node = SectionPos::block_to_section(block_node);
        let layer = self
            .get_data_layer(section_node, true)
            .unwrap_or_else(|| panic!("no light stored for section {section_node}"));
        layer.borrow().get(
            SectionPos::section_relative(BlockPos::get_x_i64(block_node)),
            SectionPos::section_relative(BlockPos::get_y_i64(block_node)),
            SectionPos::section_relative(BlockPos::get_z_i64(block_node)),
        )
    }

    pub(super) fn set_stored_level(&mut self, block_node: i64, level: i32) {
        let section_node = SectionPos::block_to_section(block_node);
        let layer = if self.changed_sections.insert(section_node) {
            self.updating_section_data.copy_data_layer(section_node)
        } else {
            Rc::clone(
                self.get_data_layer(section_node, true)
                    .unwrap_or_else(|| panic!("no light stored for section {section_node}")),
            )
        };

        layer.borrow_mut().set(
            SectionPos::section_relative(BlockPos::get_x_i64(block_node)),
            SectionPos::section_relative(BlockPos::get_y_i64(block_node)),
            SectionPos::section_relative(BlockPos::get_z_i64(block_node)),
            level,
        );
        let affected = &mut self.sections_affected_by_light_updates;
        SectionPos::around_and_at_block_pos_i64(block_node, |section| {
            affected.insert(section);
        });
    }

    fn mark_section_and_neighbors_as_affected(&mut self, section_node: i64) {
        let x = SectionPos::x_of(section_node);
        let y = SectionPos::y_of(section_node);
        let z = SectionPos::z_of(section_node);

        for offset_z in -1..=1 {
            for offset_x in -1..=1 {
                for offset_y in -1..=1 {
                    self.sections_affected_by_light_updates
                        .insert(SectionPos::as_long_xyz(
                            x + offset_x,
                            y + offset_y,
                            z + offset_z,
                        ));
                }
            }
        }
    }

    pub(super) fn has_inconsistencies(&self) -> bool {
        self.has_inconsistencies
    }

    pub(super) fn set_light_enabled(&mut self, zero_node: i64, enable: bool) {
        if enable {
            self.columns_with_sources.insert(zero_node);
        } else {
            self.columns_with_sources.remove(&zero_node);
        }
    }

    pub(super) fn light_on_in_section(&self, section_node: i64) -> bool {
        let zero_node = SectionPos::get_zero_node(section_node);
        self.columns_with_sources.contains(&zero_node)
    }

    pub(super) fn light_on_in_column(&self, section_zero_node: i64) -> bool {
        self.columns_with_sources.contains(&section_zero_node)
    }

    pub fn retain_data(&mut self, zero_node: i64, retain: bool) {
        if retain {
            self.columns_to_retain_queued_data_for.insert(zero_node);
        } else {
            self.columns_to_retain_queued_data_for.remove(&zero_node);
        }
    }

    pub(super) fn queue_section_data(&mut self, section_node: i64, data: Option<SharedDataLayer>) {
        match data {
            Some(data) => {
                self.queued_sections.insert(section_node, data);
                self.has_inconsistencies = true;
            }
            None => {
                self.queued_sections.remove(&section_node);
            }
        }
    }

    /// Swaps in a copy of the layers worked on, if any changed, and tells the chunk source which
    /// sections' light changed.
    pub(super) fn swap_section_map(&mut self, chunk_source: &dyn LightChunkGetter) {
        if !self.changed_sections.is_empty() {
            self.visible_section_data = self.updating_section_data.copy();
            self.changed_sections.clear();
        }

        if !self.sections_affected_by_light_updates.is_empty() {
            for &section_node in &self.sections_affected_by_light_updates {
                chunk_source.on_light_update(self.layer, SectionPos::of_long(section_node));
            }

            self.sections_affected_by_light_updates.clear();
        }
    }

    pub fn get_debug_section_type(&self, section_node: i64) -> SectionType {
        SectionState::r#type(self.section_states.get(&section_node).copied().unwrap_or(0))
    }
}

/// The virtual methods of a layer's storage.
pub trait LayerLightSectionStorageClass:
    DerefMut<Target = LayerLightSectionStorage<Self::Map>>
{
    type Map: DataLayerStorageMapClass;

    /// The light the game sees at a block.
    fn get_light_value(&self, block_node: i64) -> i32;

    fn on_node_added(&mut self, _section_node: i64) {}

    fn on_node_removed(&mut self, _section_node: i64) {}

    /// The layer a section starts with: the one queued for it, or an empty one.
    fn create_data_layer(&self, section_node: i64) -> SharedDataLayer {
        match self.queued_sections.get(&section_node) {
            Some(queued_layer) => Rc::clone(queued_layer),
            None => Rc::new(RefCell::new(DataLayer::new())),
        }
    }
}

/// Stores the layers queued for sections now stored, and drops the removed sections' layers
/// (keeping them queued for a column whose data is retained).
pub(super) fn mark_new_inconsistencies(this: &mut impl LayerLightSectionStorageClass) {
    if this.has_inconsistencies {
        this.has_inconsistencies = false;

        let to_remove: Vec<i64> = this.to_remove.iter().copied().collect();
        for &node in &to_remove {
            let queued = this.queued_sections.remove(&node);
            let stored = this.updating_section_data.remove_layer(node);
            if this
                .columns_to_retain_queued_data_for
                .contains(&SectionPos::get_zero_node(node))
            {
                if let Some(queued) = queued {
                    this.queued_sections.insert(node, queued);
                } else if let Some(stored) = stored {
                    this.queued_sections.insert(node, stored);
                }
            }
        }

        for &node in &to_remove {
            this.on_node_removed(node);
            this.changed_sections.insert(node);
        }

        this.to_remove.clear();
        let queued: Vec<(i64, SharedDataLayer)> = this
            .queued_sections
            .iter()
            .map(|(&node, layer)| (node, Rc::clone(layer)))
            .collect();
        for (section_node, data) in queued {
            if this.storing_light_for_section(section_node) {
                let stored = this.updating_section_data.get_layer(section_node);
                if !stored.is_some_and(|stored| Rc::ptr_eq(stored, &data)) {
                    this.updating_section_data.set_layer(section_node, data);
                    this.changed_sections.insert(section_node);
                }

                this.queued_sections.remove(&section_node);
            }
        }
    }
}

/// Whether a section has blocks: its state and its neighbours' counts change, and a section that
/// comes to have a state is stored, one that comes to have none removed.
pub(super) fn update_section_status(
    this: &mut impl LayerLightSectionStorageClass,
    section_node: i64,
    section_empty: bool,
) {
    let state = this.section_states.get(&section_node).copied().unwrap_or(0);
    let new_state = SectionState::has_data(state, !section_empty);
    if state != new_state {
        put_section_state(this, section_node, new_state);
        let neighbor_increment = if section_empty { -1 } else { 1 };

        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                for offset_z in -1..=1 {
                    if offset_x != 0 || offset_y != 0 || offset_z != 0 {
                        let neighbor_node =
                            SectionPos::offset_i64_xyz(section_node, offset_x, offset_y, offset_z);
                        let neighbor_state = this
                            .section_states
                            .get(&neighbor_node)
                            .copied()
                            .unwrap_or(0);
                        put_section_state(
                            this,
                            neighbor_node,
                            SectionState::neighbor_count(
                                neighbor_state,
                                SectionState::neighbor_count_of(neighbor_state)
                                    + neighbor_increment,
                            ),
                        );
                    }
                }
            }
        }
    }
}

fn put_section_state(this: &mut impl LayerLightSectionStorageClass, section_node: i64, state: i8) {
    if state != 0 {
        if this.section_states.insert(section_node, state).unwrap_or(0) == 0 {
            initialize_section(this, section_node);
        }
    } else if this.section_states.remove(&section_node).unwrap_or(0) != 0 {
        remove_section(this, section_node);
    }
}

fn initialize_section(this: &mut impl LayerLightSectionStorageClass, section_node: i64) {
    if !this.to_remove.remove(&section_node) {
        let data_layer = this.create_data_layer(section_node);
        this.updating_section_data
            .set_layer(section_node, data_layer);
        this.changed_sections.insert(section_node);
        this.on_node_added(section_node);
        this.mark_section_and_neighbors_as_affected(section_node);
        this.has_inconsistencies = true;
    }
}

fn remove_section(this: &mut impl LayerLightSectionStorageClass, section_node: i64) {
    this.to_remove.insert(section_node);
    this.has_inconsistencies = true;
}

/// `SectionState`: a section's state as a byte, whether it has blocks (`HAS_DATA_BIT`) and how many
/// of its neighbours do.
pub struct SectionState;

impl SectionState {
    pub const EMPTY: i8 = 0;
    const MAX_NEIGHBORS: i32 = 26;
    const HAS_DATA_BIT: i8 = 32;
    const NEIGHBOR_COUNT_BITS: i8 = 31;

    /// `hasData(byte, boolean)`.
    pub fn has_data(state: i8, has_data: bool) -> i8 {
        if has_data {
            state | SectionState::HAS_DATA_BIT
        } else {
            state & !SectionState::HAS_DATA_BIT
        }
    }

    /// `neighborCount(byte, int)`: panics outside 0 to 26, where Java throws
    /// `IllegalArgumentException`.
    pub fn neighbor_count(state: i8, neighbor_count: i32) -> i8 {
        if (0..=SectionState::MAX_NEIGHBORS).contains(&neighbor_count) {
            state & !SectionState::NEIGHBOR_COUNT_BITS | (neighbor_count as i8 & 31)
        } else {
            panic!("Neighbor count was not within range [0; 26]")
        }
    }

    /// `hasData(byte)`.
    pub fn has_data_of(state: i8) -> bool {
        state & SectionState::HAS_DATA_BIT != 0
    }

    /// `neighborCount(byte)`.
    pub fn neighbor_count_of(state: i8) -> i32 {
        i32::from(state & SectionState::NEIGHBOR_COUNT_BITS)
    }

    pub fn r#type(state: i8) -> SectionType {
        if state == 0 {
            SectionType::EMPTY
        } else if SectionState::has_data_of(state) {
            SectionType::LIGHT_AND_DATA
        } else {
            SectionType::LIGHT_ONLY
        }
    }
}

/// What the debug screens show of a section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum SectionType {
    EMPTY,
    LIGHT_ONLY,
    LIGHT_AND_DATA,
}

impl SectionType {
    pub fn display(&self) -> &'static str {
        match self {
            SectionType::EMPTY => "2",
            SectionType::LIGHT_ONLY => "1",
            SectionType::LIGHT_AND_DATA => "0",
        }
    }
}
