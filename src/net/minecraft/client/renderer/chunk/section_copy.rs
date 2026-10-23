//! net/minecraft/client/renderer/chunk/SectionCopy.java
//!
//! A section's blocks, copied for a compile so the level can change meanwhile. Not ported yet:
//! the chunk's block entities (the client's chunks hold none yet: the world track's), which
//! `getBlockEntity` hands out, and the debug level's blocks (`DebugLevelSource`), which panic.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::chunk::level_chunk::LevelChunk;
use crate::net::minecraft::world::level::chunk::paletted_container::PalettedContainer;

pub struct SectionCopy {
    section: Option<PalettedContainer<BlockState>>,
    debug: bool,
}

impl SectionCopy {
    /// The section of a chunk, or of none for a chunk the level doesn't have (Java's
    /// `EmptyLevelChunk`), at `section_index`; `debug` is the chunk's level's `isDebug()`.
    pub fn new(level_chunk: Option<&LevelChunk>, section_index: i32, debug: bool) -> SectionCopy {
        let section = level_chunk.and_then(|level_chunk| {
            let sections = level_chunk.get_sections();
            if section_index >= 0 && (section_index as usize) < sections.len() {
                let level_chunk_section = &sections[section_index as usize];
                (!level_chunk_section.has_only_air())
                    .then(|| level_chunk_section.get_states().copy())
            } else {
                None
            }
        });
        SectionCopy { section, debug }
    }

    /// A copy of these states, or of a section of only air: what the constructor keeps.
    pub fn of_states(section: Option<PalettedContainer<BlockState>>) -> SectionCopy {
        SectionCopy {
            section,
            debug: false,
        }
    }

    pub fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        assert!(
            !self.debug,
            "the debug level's blocks (DebugLevelSource) aren't ported"
        );
        match &self.section {
            None => Blocks::AIR.default_block_state(),
            Some(section) => section.get(pos.get_x() & 15, pos.get_y() & 15, pos.get_z() & 15),
        }
    }
}
