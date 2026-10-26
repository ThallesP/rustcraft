//! net/minecraft/client/SectionUpdateTracker.java
//!
//! Which sections around the camera need compiling: all of them when they come into the render
//! distance, and those a block update marks (`setDirty`). `hasAllNeighbors` asks whether the
//! level has the chunks around a section and has turned their columns' light on
//! (`lightOnInColumn`), so a section first compiles with its neighbours' light in.

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::rotating_section_storage::{RotatingSectionStorage, Value};
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;

pub struct SectionUpdateTracker {
    storage: RotatingSectionStorage<SectionDirtyState>,
}

impl SectionUpdateTracker {
    pub fn new(level: &dyn LevelHeightAccessor, render_distance: i32) -> SectionUpdateTracker {
        SectionUpdateTracker {
            storage: RotatingSectionStorage::new(
                render_distance,
                level.get_min_section_y(),
                level.get_max_section_y(),
                |_, section_node| SectionDirtyState {
                    is_dirty: true,
                    is_dirty_from_player: false,
                    section_node,
                },
            ),
        }
    }

    pub fn set_dirty(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        player_changed: bool,
    ) {
        if let Some(section) = self
            .storage
            .get_value_mut(SectionPos::as_long_xyz(section_x, section_y, section_z))
        {
            section.set_dirty(player_changed);
        }
    }

    pub fn reposition_camera(&mut self, camera_section_pos: SectionPos) {
        self.storage.reposition_center(camera_section_pos);
    }

    pub fn size(&self) -> usize {
        self.storage.size()
    }

    pub fn get_dirty_state(&mut self, section_node: i64) -> Option<&mut SectionDirtyState> {
        self.storage.get_value_mut(section_node)
    }

    /// Whether the level has the eight chunks around the section's, with their light on.
    pub fn has_all_neighbors(&self, level: &ClientLevel, section_node: i64) -> bool {
        let x = SectionPos::x_of(section_node);
        let z = SectionPos::z_of(section_node);
        [
            (-1, 0),
            (0, -1),
            (1, 0),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ]
        .iter()
        .all(|(dx, dz)| SectionUpdateTracker::does_chunk_exist_at(level, x + dx, z + dz))
    }

    /// `doesChunkExistAt`: the level has the chunk, and the light engine has turned its column's
    /// light on (the chunk's queued light update has run).
    fn does_chunk_exist_at(level: &ClientLevel, chunk_x: i32, chunk_z: i32) -> bool {
        level
            .get_chunk_source()
            .get_chunk(chunk_x, chunk_z)
            .is_some()
            && level
                .get_chunk_source()
                .get_light_engine()
                .light_on_in_column(SectionPos::as_long_xyz(chunk_x, 0, chunk_z))
    }
}

/// `SectionUpdateTracker.SectionDirtyState`.
pub struct SectionDirtyState {
    is_dirty: bool,
    is_dirty_from_player: bool,
    section_node: i64,
}

impl SectionDirtyState {
    pub fn set_dirty(&mut self, from_player: bool) {
        let was_dirty = self.is_dirty;
        self.is_dirty = true;
        self.is_dirty_from_player = from_player | (was_dirty && self.is_dirty_from_player);
    }

    pub fn set_not_dirty(&mut self) {
        self.is_dirty = false;
        self.is_dirty_from_player = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn is_dirty_from_player(&self) -> bool {
        self.is_dirty_from_player
    }
}

impl Value for SectionDirtyState {
    /// A section that moves in is dirty.
    fn set_section_node(&mut self, section_node: i64) {
        if self.section_node != section_node {
            self.section_node = section_node;
            self.is_dirty = true;
            self.is_dirty_from_player = false;
        }
    }

    fn get_section_node(&self) -> i64 {
        self.section_node
    }
}
