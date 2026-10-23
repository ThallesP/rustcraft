//! net/minecraft/world/level/chunk/LevelChunk.java
//!
//! The client's side of a chunk: its sections, its heightmaps (the server's, kept up as blocks
//! are set), its sky light sources, its block entities, and reading and setting blocks. Java's
//! `LevelChunk` extends `ChunkAccess`, whose fields it holds here. `setBlockState` tells the
//! level's light engine (Java's `level.getChunkSource().getLightEngine()`, a parameter here) when a
//! section's emptiness or a block's light changes. The block entities are a Java `HashMap` by
//! position, as Java's iterates; `replaceWithPacketData` loads the packet's with the level's
//! registries (a parameter, Java's `level.registryAccess()`). Not ported yet: the block entities'
//! tickers (`tickersInLevel`), the pending ones (`pendingBlockEntities`, the server's), the debug
//! level's blocks (`getBlockState` in a debug world), priming a heightmap whose data didn't fit,
//! the chunk source's `onSectionEmptinessChanged` and the level's `onBlockEntityAdded` (the
//! renderer's), ticks, and everything the server does with a chunk.

use std::collections::BTreeMap;

use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::problem_reporter::ScopedCollector;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BY_BLOCK;
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
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

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
    block_entities: HashMap<BlockPos, BlockEntity>,
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
                    Heightmap::new(level_height.get_min_y(), level_height.get_height(), r#type),
                )
            })
            .collect();
        LevelChunk {
            chunk_pos: pos,
            level_height,
            sections,
            heightmaps,
            sky_light_sources: ChunkSkyLightSources::new(&level_height),
            block_entities: HashMap::new(),
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
            .or_insert_with(|| {
                Heightmap::new(level_height.get_min_y(), level_height.get_height(), key)
            })
            .set_raw_data(data);
    }

    /// The server's chunk: its block entities dropped, every section read from its buffer in
    /// turn, then its heightmaps, and the block entities the packet has tags of made where their
    /// blocks make them and loaded from the tags (`TagValueInput`, with `registries`).
    pub fn replace_with_packet_data(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_data: &ClientboundLevelChunkPacketData,
        registries: &RegistryAccess,
    ) {
        let mut buffer = chunk_data.get_read_buffer();
        self.clear_all_block_entities();
        for section in &mut self.sections {
            section.read(&mut buffer);
        }

        for (&r#type, data) in chunk_data.get_heightmaps() {
            self.set_heightmap(r#type, data);
        }
        self.initialize_light_sources();

        let reporter = ScopedCollector::new(format!("chunk {}", self.chunk_pos));
        chunk_data.for_each_block_entity_tag(chunk_x, chunk_z, &mut |pos, r#type, tag| {
            if let Some(block_entity) = self.get_block_entity_immediate(&pos)
                && let Some(tag) = tag
                && block_entity.get_type() == r#type
            {
                block_entity
                    .load_with_components(&TagValueInput::create(&reporter, registries, tag));
            }
        });
    }

    /// `clearAllBlockEntities`: each dropped (Java marks them removed for whoever still holds
    /// one, which nothing here can).
    fn clear_all_block_entities(&mut self) {
        self.block_entities = HashMap::new();
    }

    /// `getBlockEntity(pos)`, `CHECK`: the block entity there, if it has one that isn't removed.
    /// Java also drops a removed one from the map, which only saves the next look.
    pub fn get_block_entity(&self, pos: &BlockPos) -> Option<&BlockEntity> {
        self.block_entities
            .get(pos)
            .filter(|block_entity| !block_entity.is_removed())
    }

    /// `getBlockEntity(pos, IMMEDIATE)`: the block entity there, made if its block makes one and
    /// it has none.
    pub fn get_block_entity_immediate(&mut self, pos: &BlockPos) -> Option<&mut BlockEntity> {
        match self.block_entities.get(pos).map(BlockEntity::is_removed) {
            None => {
                if let Some(block_entity) = self.create_block_entity(pos) {
                    self.add_and_register_block_entity(block_entity);
                }
            }
            Some(true) => {
                self.block_entities.remove(pos);
                return None;
            }
            Some(false) => {}
        }
        self.block_entities.get_mut(pos)
    }

    /// The block entities, by position, in the map's order.
    pub fn get_block_entities(&self) -> &HashMap<BlockPos, BlockEntity> {
        &self.block_entities
    }

    /// `createBlockEntity`: the block entity the state's block makes (`newBlockEntity`).
    fn create_block_entity(&self, pos: &BlockPos) -> Option<BlockEntity> {
        let state = self.get_block_state(pos);
        if !state.has_block_entity() {
            return None;
        }
        BY_BLOCK[state.get_block().0 as usize].map(|r#type| r#type.create(*pos, state))
    }

    /// `addAndRegisterBlockEntity`: `setBlockEntity`. The level's `onBlockEntityAdded` (the
    /// renderer's) and the ticker aren't ported.
    pub fn add_and_register_block_entity(&mut self, block_entity: BlockEntity) {
        self.set_block_entity(block_entity);
    }

    /// `setBlockEntity`: the block entity kept at its position, given the chunk's state there if
    /// its type takes it, unless the chunk's block makes none there.
    pub fn set_block_entity(&mut self, mut block_entity: BlockEntity) {
        let pos = block_entity.get_block_pos();
        let block_state = self.get_block_state(&pos);
        if !block_state.has_block_entity() {
            eprintln!(
                "Trying to set block entity {block_entity:?} at position {pos}, but state {block_state} does not allow it"
            );
            return;
        }

        let cached_block_state = block_entity.get_block_state();
        if block_state != cached_block_state {
            if !block_entity.get_type().is_valid(block_state) {
                eprintln!(
                    "Trying to set block entity {block_entity:?} at position {pos}, but state {block_state} does not allow it"
                );
                return;
            }

            if block_state.get_block() != cached_block_state.get_block() {
                eprintln!(
                    "Block state mismatch on block entity {block_entity:?} in position {pos}, {block_state} != {cached_block_state}, updating"
                );
            }

            block_entity.set_block_state(block_state);
        }

        block_entity.clear_removed();
        if let Some(mut previous_entry) = self.block_entities.put(pos, block_entity) {
            previous_entry.set_removed();
        }
    }

    /// `removeBlockEntity`: dropped, marked removed.
    pub fn remove_block_entity(&mut self, pos: &BlockPos) {
        if let Some(mut remove_this) = self.block_entities.remove(pos) {
            remove_this.set_removed();
        }
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

        let new_block = state.get_block();
        let is_empty = section.has_only_air();
        for r#type in [
            Types::MOTION_BLOCKING,
            Types::MOTION_BLOCKING_NO_LEAVES,
            Types::OCEAN_FLOOR,
            Types::WORLD_SURFACE,
        ] {
            // Java's heightmap reads its chunk, which holds it.
            let mut heightmap = self
                .heightmaps
                .remove(&r#type)
                .expect("a full chunk has its heightmaps");
            heightmap.update(local_x, y, local_z, state, self);
            self.heightmaps.insert(r#type, heightmap);
        }
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

        let block_changed = old_state.get_block() != new_block;
        if block_changed
            && old_state.has_block_entity()
            && !state.should_changed_state_keep_block_entity(old_state)
        {
            self.remove_block_entity(pos);
        }

        let section_index = self.level_height.get_section_index(y) as usize;
        if self.sections[section_index]
            .get_block_state(local_x, local_y, local_z)
            .get_block()
            != new_block
        {
            return None;
        }

        if state.has_block_entity() {
            let mut block_entity = self.get_block_entity(pos).is_some();
            if block_entity
                && !self
                    .get_block_entity(pos)
                    .is_some_and(|block_entity| block_entity.is_valid_block_state(state))
            {
                eprintln!(
                    "Found mismatched block entity @ {pos}: type = {}, state = {state}",
                    self.get_block_entity(pos)
                        .map_or("", |block_entity| block_entity.get_type().key())
                );
                self.remove_block_entity(pos);
                block_entity = false;
            }

            if !block_entity {
                if let Some(r#type) = BY_BLOCK[new_block.0 as usize] {
                    self.add_and_register_block_entity(r#type.create(*pos, state));
                }
            } else if let Some(block_entity) = self.block_entities.get_mut(pos) {
                block_entity.set_block_state(state);
            }
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
