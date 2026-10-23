//! net/minecraft/world/level/chunk/LevelChunk.java
//!
//! The client's side of a chunk: its sections, the heightmaps the server sends, and reading and
//! setting blocks. Java's `LevelChunk` extends `ChunkAccess`, whose fields it holds here. Not
//! ported yet: block entities (`replaceWithPacketData` skips the packet's, and `setBlockState`
//! makes none), the debug level's blocks (`getBlockState` in a debug world), sky light sources
//! (`initializeLightSources`, `skyLightSources`), heightmaps worked out from blocks (`update`,
//! priming a heightmap whose data didn't fit), ticks, and everything the server does with a
//! chunk. `setBlockState`'s light checks and section status go to the level's light engine,
//! which isn't ported either.

use std::collections::BTreeMap;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::{Height, LevelHeightAccessor};
use crate::net::minecraft::world::level::levelgen::heightmap::{Heightmap, Types};
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

/// `ChunkStatus.FULL.heightmapsAfter()`: the heightmaps a full chunk keeps.
const FINAL_HEIGHTMAPS: [Types; 4] = [
    Types::WORLD_SURFACE,
    Types::OCEAN_FLOOR,
    Types::MOTION_BLOCKING,
    Types::MOTION_BLOCKING_NO_LEAVES,
];

/// A column of 16 by 16 blocks, the level's height high, in sections.
pub struct LevelChunk {
    chunk_pos: ChunkPos,
    // The level's height, which Java's `ChunkAccess` asks its `levelHeightAccessor`.
    level_height: Height,
    sections: Vec<LevelChunkSection>,
    heightmaps: BTreeMap<Types, Heightmap>,
}

impl LevelChunk {
    /// `LevelChunk(level, pos)`: every section new, air and the default biome, and the full
    /// chunk's heightmaps.
    pub fn new(
        level_height: Height,
        container_factory: &PalettedContainerFactory,
        pos: ChunkPos,
    ) -> LevelChunk {
        let sections = (0..level_height.get_sections_count())
            .map(|_| LevelChunkSection::new(container_factory))
            .collect();
        let heightmaps = FINAL_HEIGHTMAPS
            .iter()
            .map(|&r#type| {
                (
                    r#type,
                    Heightmap::new(level_height.get_min_y(), level_height.get_height()),
                )
            })
            .collect();
        LevelChunk {
            chunk_pos: pos,
            level_height,
            sections,
            heightmaps,
        }
    }

    pub fn get_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub fn get_sections(&self) -> &[LevelChunkSection] {
        &self.sections
    }

    /// Panics for an index past the sections, where Java throws.
    pub fn get_section(&self, section_index: i32) -> &LevelChunkSection {
        &self.sections[section_index as usize]
    }

    pub fn get_heightmap(&self, r#type: Types) -> Option<&Heightmap> {
        self.heightmaps.get(&r#type)
    }

    /// `setHeightmap(type, data)`: the server's heights, in a heightmap of the type made if
    /// there's none. Data of the wrong size leaves the heights as they were, where Java primes
    /// the heightmap from the chunk's blocks (with the types' predicates, which aren't ported).
    pub fn set_heightmap(&mut self, key: Types, data: &[i64]) {
        let level_height = self.level_height;
        self.heightmaps
            .entry(key)
            .or_insert_with(|| Heightmap::new(level_height.get_min_y(), level_height.get_height()))
            .set_raw_data(data);
    }

    /// The server's chunk: every section read from its buffer in turn, then its heightmaps.
    pub fn replace_with_packet_data(
        &mut self,
        _chunk_x: i32,
        _chunk_z: i32,
        chunk_data: &ClientboundLevelChunkPacketData,
    ) {
        let mut buffer = chunk_data.get_read_buffer();
        for section in &mut self.sections {
            section.read(&mut buffer);
        }

        for (&r#type, data) in chunk_data.get_heightmaps() {
            self.set_heightmap(r#type, data);
        }
    }

    /// `replaceBiomes`: every section's biomes read from the buffer in turn.
    pub fn replace_biomes(&mut self, buffer: &mut FriendlyByteBuf) {
        for section in &mut self.sections {
            section.read_biomes(buffer);
        }
    }

    /// `getBlockState(int, int, int)`, which the position overload calls: air in a section
    /// that's only air, or past the level's height.
    #[inline]
    pub fn get_block_state_xyz(&self, x: i32, y: i32, z: i32) -> BlockState {
        let section_index = self.get_section_index(y);
        if section_index >= 0 && (section_index as usize) < self.sections.len() {
            let current_section = &self.sections[section_index as usize];
            if !current_section.has_only_air() {
                return current_section.get_block_state(x & 15, y & 15, z & 15);
            }
        }

        Blocks::AIR.default_block_state()
    }

    pub fn get_fluid_state_xyz(&self, x: i32, y: i32, z: i32) -> FluidState {
        let section_index = self.get_section_index(y);
        if section_index >= 0 && (section_index as usize) < self.sections.len() {
            let current_section = &self.sections[section_index as usize];
            if !current_section.has_only_air() {
                return current_section.get_fluid_state(x & 15, y & 15, z & 15);
            }
        }

        FluidState::EMPTY
    }

    /// `setBlockState(pos, state, flags)`: the state it replaced, or none when nothing changed
    /// (air set in a section of only air, or the same state). Also says whether the section went
    /// from only air to not, or back, which Java tells the light engine and the chunk source.
    /// Panics past the level's height, where Java throws.
    pub fn set_block_state(
        &mut self,
        pos: &BlockPos,
        state: BlockState,
    ) -> Option<(BlockState, bool)> {
        let y = pos.get_y();
        let section = &mut self.sections[self.level_height.get_section_index(y) as usize];
        let was_empty = section.has_only_air();
        if was_empty && state.is_air() {
            return None;
        }

        let old_state = section.set_block_state(pos.get_x() & 15, y & 15, pos.get_z() & 15, state);
        if old_state == state {
            return None;
        }

        let emptiness_changed = was_empty != section.has_only_air();
        Some((old_state, emptiness_changed))
    }
}

impl LevelHeightAccessor for LevelChunk {
    fn get_height(&self) -> i32 {
        self.level_height.get_height()
    }

    fn get_min_y(&self) -> i32 {
        self.level_height.get_min_y()
    }
}

impl BlockGetter for LevelChunk {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        self.get_block_state_xyz(pos.get_x(), pos.get_y(), pos.get_z())
    }

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        self.get_fluid_state_xyz(pos.get_x(), pos.get_y(), pos.get_z())
    }
}
