//! net/minecraft/world/level/chunk/LevelChunk.java
//!
//! The client's side of a chunk: its sections, the heightmaps the server sends, its sky light
//! sources, and reading and setting blocks. Java's `LevelChunk` extends `ChunkAccess`, whose
//! fields it holds here. `setBlockState` tells the level's light engine (Java's
//! `level.getChunkSource().getLightEngine()`, a parameter here) when a section's emptiness or a
//! block's light changes. Not ported yet: block entities (`replaceWithPacketData` skips the
//! packet's, and `setBlockState` makes none), the debug level's blocks (`getBlockState` in a debug
//! world), heightmaps worked out from blocks (`update`, priming a heightmap whose data didn't fit),
//! the chunk source's `onSectionEmptinessChanged` (the renderer's), ticks, and everything the
//! server does with a chunk.

use std::collections::BTreeMap;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::net::minecraft::world::level::chunk::light_chunk::LightChunk;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::{Height, LevelHeightAccessor};
use crate::net::minecraft::world::level::levelgen::heightmap::{Heightmap, Types};
use crate::net::minecraft::world::level::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::lighting::light_engine::LightEngine;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
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
    sky_light_sources: ChunkSkyLightSources,
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
            sky_light_sources: ChunkSkyLightSources::new(&level_height),
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

    /// `ChunkAccess.getHighestFilledSectionIndex`: the highest section that isn't only air, or -1.
    pub fn get_highest_filled_section_index(&self) -> i32 {
        for section_index in (0..self.sections.len()).rev() {
            if !self.sections[section_index].has_only_air() {
                return section_index as i32;
            }
        }

        -1
    }

    /// `ChunkAccess.initializeLightSources`: the sky light sources, from the blocks.
    pub fn initialize_light_sources(&mut self) {
        let mut sources = std::mem::replace(
            &mut self.sky_light_sources,
            ChunkSkyLightSources::new(&self.level_height),
        );
        sources.fill_from(self);
        self.sky_light_sources = sources;
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
        self.initialize_light_sources();
    }

    /// `ChunkAccess.getNoiseBiome`: the biome at the quart, its y kept in the chunk's height.
    pub fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        let quart_min_y = QuartPos::from_block(self.get_min_y());
        let quart_max_y = quart_min_y + QuartPos::from_block(self.get_height()) - 1;
        let clamped_quart_y = Mth::clamp_i32(quart_y, quart_min_y, quart_max_y);
        let section_index = self.get_section_index(QuartPos::to_block(clamped_quart_y));
        self.sections[section_index as usize].get_noise_biome(
            quart_x & 3,
            clamped_quart_y & 3,
            quart_z & 3,
        )
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
    /// (air set in a section of only air, or the same state). The light engine hears of a section
    /// going from only air to not or back, and of a block whose light changed, whose column's sky
    /// light sources move with it. Panics past the level's height, where Java throws.
    pub fn set_block_state(
        &mut self,
        pos: &BlockPos,
        state: BlockState,
        light_engine: &mut LevelLightEngine,
    ) -> Option<BlockState> {
        let y = pos.get_y();
        let section = &mut self.sections[self.level_height.get_section_index(y) as usize];
        let was_empty = section.has_only_air();
        if was_empty && state.is_air() {
            return None;
        }

        let local_x = pos.get_x() & 15;
        let local_y = y & 15;
        let local_z = pos.get_z() & 15;
        let old_state = section.set_block_state(local_x, local_y, local_z, state);
        if old_state == state {
            return None;
        }

        let is_empty = section.has_only_air();
        if was_empty != is_empty {
            light_engine.update_section_status_block_pos(pos, is_empty);
        }

        if LightEngine::has_different_light_properties(old_state, state) {
            let mut sources = std::mem::replace(
                &mut self.sky_light_sources,
                ChunkSkyLightSources::new(&self.level_height),
            );
            sources.update(self, local_x, y, local_z);
            self.sky_light_sources = sources;
            light_engine.check_block(pos);
        }

        Some(old_state)
    }
}

impl LightChunk for LevelChunk {
    fn get_sky_light_sources(&self) -> &ChunkSkyLightSources {
        &self.sky_light_sources
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
