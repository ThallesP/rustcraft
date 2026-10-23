//! net/minecraft/client/renderer/extract/LevelExtractor.java
//!
//! What the level's changes tell the renderer (`Minecraft.levelExtractor`): the sections around a
//! changed block, a loaded chunk, new biomes or new light are dirty, to compile again. Java's calls
//! go straight to the extractor's `SectionUpdateTracker`; the port's tracker is the level
//! renderer's (`LevelRenderer`, which also does the rest of Java's extractor: `extract_sections`),
//! which the level can't reach, so the level and its chunk cache share this, and the level
//! renderer takes its sections before it extracts a frame (`take_dirty_sections`), as Java's calls
//! would have reached the tracker by then (the packets come before the frame). A section marked
//! twice is dirty once, and from the player if either mark was, which is what the tracker makes of
//! two marks in a row (`SectionDirtyState.setDirty`).
//!
//! Until a renderer has the level (`listen`, from `allChanged`), nothing is kept: a new tracker
//! starts with every section dirty, so the marks before it change nothing, and a headless client
//! that never renders keeps none.
//!
//! `setBlockDirty(pos, oldState, newState)` asks `ModelManager.requiresRender`, whose model groups
//! (`ModelGroupCollector`, the states that look the same) aren't ported: every state is in no
//! group, which Java answers with true for any two states that differ. So a change between states
//! that look the same marks the sections around it too, which only compiles them again.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// The dirty sections the level renderer's tracker hasn't taken yet.
#[derive(Debug, Default)]
pub struct LevelExtractor {
    listening: Cell<bool>,
    /// Each section, by `SectionPos.asLong`, and whether the player's own change made it dirty.
    dirty_sections: RefCell<HashMap<i64, bool>>,
}

impl LevelExtractor {
    pub fn new() -> LevelExtractor {
        LevelExtractor::default()
    }

    /// A renderer has the level from now on (`allChanged`): what's marked before is dropped.
    pub fn listen(&self) {
        self.listening.set(true);
        self.dirty_sections.borrow_mut().clear();
    }

    /// The sections marked since the last time, each `(x, y, z, player_changed)`.
    pub fn take_dirty_sections(&self) -> Vec<(i32, i32, i32, bool)> {
        self.dirty_sections
            .borrow_mut()
            .drain()
            .map(|(node, player_changed)| {
                (
                    SectionPos::x_of(node),
                    SectionPos::y_of(node),
                    SectionPos::z_of(node),
                    player_changed,
                )
            })
            .collect()
    }

    /// `blockChanged`: `Level.sendBlockUpdated`'s, the player's own when the flags have 8
    /// (`Block.UPDATE_IMMEDIATE`).
    pub fn block_changed(&self, pos: &BlockPos, update_flags: i32) {
        self.set_block_dirty(pos, (update_flags & 8) != 0);
    }

    fn set_block_dirty(&self, pos: &BlockPos, player_changed: bool) {
        for z in pos.get_z() - 1..=pos.get_z() + 1 {
            for x in pos.get_x() - 1..=pos.get_x() + 1 {
                for y in pos.get_y() - 1..=pos.get_y() + 1 {
                    self.set_section_dirty_from(
                        SectionPos::block_to_section_coord(x),
                        SectionPos::block_to_section_coord(y),
                        SectionPos::block_to_section_coord(z),
                        player_changed,
                    );
                }
            }
        }
    }

    pub fn set_blocks_dirty(&self, x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32) {
        for z in z0 - 1..=z1 + 1 {
            for x in x0 - 1..=x1 + 1 {
                for y in y0 - 1..=y1 + 1 {
                    self.set_section_dirty(
                        SectionPos::block_to_section_coord(x),
                        SectionPos::block_to_section_coord(y),
                        SectionPos::block_to_section_coord(z),
                    );
                }
            }
        }
    }

    /// `setBlockDirty(BlockPos, BlockState, BlockState)`: `Level.setBlocksDirty`'s, when the
    /// change shows (see the header for `requiresRender`).
    pub fn set_block_dirty_states(
        &self,
        pos: &BlockPos,
        old_state: BlockState,
        new_state: BlockState,
    ) {
        if old_state != new_state {
            self.set_blocks_dirty(
                pos.get_x(),
                pos.get_y(),
                pos.get_z(),
                pos.get_x(),
                pos.get_y(),
                pos.get_z(),
            );
        }
    }

    pub fn set_section_dirty_with_neighbors(&self, section_x: i32, section_y: i32, section_z: i32) {
        self.set_section_range_dirty(
            section_x - 1,
            section_y - 1,
            section_z - 1,
            section_x + 1,
            section_y + 1,
            section_z + 1,
        );
    }

    pub fn set_section_range_dirty(
        &self,
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        max_section_x: i32,
        max_section_y: i32,
        max_section_z: i32,
    ) {
        for z in min_section_z..=max_section_z {
            for x in min_section_x..=max_section_x {
                for y in min_section_y..=max_section_y {
                    self.set_section_dirty(x, y, z);
                }
            }
        }
    }

    pub fn set_section_dirty(&self, section_x: i32, section_y: i32, section_z: i32) {
        self.set_section_dirty_from(section_x, section_y, section_z, false);
    }

    fn set_section_dirty_from(
        &self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        player_changed: bool,
    ) {
        if !self.listening.get() {
            return;
        }
        *self
            .dirty_sections
            .borrow_mut()
            .entry(SectionPos::as_long_xyz(section_x, section_y, section_z))
            .or_insert(false) |= player_changed;
    }
}

#[cfg(test)]
mod tests {
    use super::LevelExtractor;
    use crate::net::minecraft::core::block_pos::BlockPos;

    /// A block's 27 neighbours span the sections it touches: one section inside, eight at a
    /// section's corner; marks before `listen` are dropped, and a player's mark sticks.
    #[test]
    fn marks_the_sections_around_a_block() {
        let extractor = LevelExtractor::new();
        extractor.block_changed(&BlockPos::new(5, 5, 5), 3);
        extractor.listen();
        assert!(extractor.take_dirty_sections().is_empty());

        extractor.block_changed(&BlockPos::new(5, 5, 5), 3);
        assert_eq!(extractor.take_dirty_sections(), [(0, 0, 0, false)]);

        extractor.block_changed(&BlockPos::new(0, -64, 15), 3);
        extractor.block_changed(&BlockPos::new(0, -64, 15), 11);
        let mut sections = extractor.take_dirty_sections();
        sections.sort();
        assert_eq!(
            sections,
            [
                (-1, -5, 0, true),
                (-1, -5, 1, true),
                (-1, -4, 0, true),
                (-1, -4, 1, true),
                (0, -5, 0, true),
                (0, -5, 1, true),
                (0, -4, 0, true),
                (0, -4, 1, true),
            ]
        );
    }
}
