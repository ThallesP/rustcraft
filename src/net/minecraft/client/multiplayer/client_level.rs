//! net/minecraft/client/multiplayer/ClientLevel.java
//!
//! Not ported yet: nearly all of it. So far a level is what the login said about it, its chunks
//! (`ClientChunkCache`), reading and setting their blocks, their light (the chunk cache's
//! `LevelLightEngine`, which the queued light updates feed and `update` runs, once a frame), and
//! what an entity's movement asks of it (`CollisionGetter` and the entity's `Level`; its world
//! border is the default one until the border packets set it). Not ported: entities (so no entity
//! collides), block entities, the block state predictions (`blockStatePredictionHandler`, which
//! the player's own block changes make), the tint caches, neighbour updates and the renderer's
//! dirty sections (`setBlocksDirty`, `sendBlockUpdated`, `setSectionDirtyWithNeighbors`).
//!
//! Java's light updates are `Runnable`s that close over the packet listener; here they're closures
//! of the level (`queue_light_update`), since all they touch is the level.
//!
//! The game time and the spawn are what the server last sent. The level's tick (`tickTime`, which
//! adds a game tick and ticks the listener's clocks) isn't ported, and `setRespawnData` keeps the
//! spawn as it came, where Java moves one outside the world border to the heightmap's top at the
//! border's center (`getWorldBorderAdjustedRespawnData`), which waits for the heightmaps.
//!
//! The level's height is its dimension type's, which the vanilla pack's data gives for the
//! built-in types (`DimensionType::height_of`); a server sending a type of its own isn't read
//! yet.

use std::collections::VecDeque;
use std::fmt;

use crate::net::minecraft::client::multiplayer::client_chunk_cache::ClientChunkCache;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::border::world_border::WorldBorder;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::level_height_accessor::{Height, LevelHeightAccessor};
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::storage::level_data::RespawnData;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// The level the player is in, as the client knows it.
pub struct ClientLevel {
    dimension: ResourceKey,
    dimension_type: Holder,
    height: Height,
    chunk_source: ClientChunkCache,
    server_chunk_radius: i32,
    server_simulation_distance: i32,
    is_debug: bool,
    biome_zoom_seed: i64,
    sea_level: i32,
    hardcore: bool,
    is_flat: bool,
    world_border: WorldBorder,
    game_time: i64,
    respawn_data: RespawnData,
    light_update_queue: VecDeque<LightUpdate>,
}

/// A light update queued for the level, Java's `Runnable`.
pub type LightUpdate = Box<dyn FnOnce(&mut ClientLevel)>;

impl ClientLevel {
    /// Panics on a dimension type that isn't one of the vanilla pack's, or registries without
    /// biomes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry_access: &RegistryAccess,
        hardcore: bool,
        is_flat: bool,
        dimension: ResourceKey,
        dimension_type: Holder,
        server_chunk_radius: i32,
        server_simulation_distance: i32,
        is_debug: bool,
        biome_zoom_seed: i64,
        sea_level: i32,
    ) -> ClientLevel {
        let height = DimensionType::height_of(&dimension_type);
        let factory = PalettedContainerFactory::create(registry_access);
        let has_sky_light = DimensionType::has_sky_light_of(&dimension_type);
        ClientLevel {
            dimension,
            dimension_type,
            height,
            chunk_source: ClientChunkCache::new(
                height,
                has_sky_light,
                factory,
                server_chunk_radius,
            ),
            server_chunk_radius,
            server_simulation_distance,
            is_debug,
            biome_zoom_seed,
            sea_level,
            hardcore,
            is_flat,
            world_border: WorldBorder::new(),
            game_time: 0,
            respawn_data: RespawnData::clone(RespawnData::DEFAULT),
            light_update_queue: VecDeque::new(),
        }
    }

    const NORMAL_LIGHT_UPDATES_PER_FRAME: usize = 10;
    const LIGHT_UPDATE_QUEUE_SIZE_THRESHOLD: usize = 1000;

    pub fn queue_light_update(&mut self, update: LightUpdate) {
        self.light_update_queue.push_back(update);
    }

    /// Runs a tenth of the queued light updates (at least 10), or all of them once there are a
    /// thousand.
    pub fn poll_light_updates(&mut self) {
        let size = self.light_update_queue.len();
        let light_updates_per_frame = if size < ClientLevel::LIGHT_UPDATE_QUEUE_SIZE_THRESHOLD {
            ClientLevel::NORMAL_LIGHT_UPDATES_PER_FRAME.max(size / 10)
        } else {
            size
        };

        for _ in 0..light_updates_per_frame {
            let Some(update) = self.light_update_queue.pop_front() else {
                break;
            };

            update(self);
        }
    }

    /// How many light updates are queued.
    #[cfg(test)]
    pub(crate) fn light_updates_queued(&self) -> usize {
        self.light_update_queue.len()
    }

    /// `update()`, once a frame: the queued light updates, then the light engine's own.
    pub fn update(&mut self) {
        self.poll_light_updates();
        let (light_engine, chunks) = self.chunk_source.get_light_engine_mut();
        light_engine.run_light_updates(chunks);
    }

    /// `getLightEngine()`, to change: the chunk source's, with the chunks it lights.
    pub fn get_light_engine_mut(&mut self) -> (&mut LevelLightEngine, &dyn LightChunkGetter) {
        self.chunk_source.get_light_engine_mut()
    }

    /// `getSectionYFromSectionIndex`, as `LevelHeightAccessor`'s.
    pub fn get_section_y_from_section_index(&self, section_index: i32) -> i32 {
        LevelHeightAccessor::get_section_y_from_section_index(self, section_index)
    }

    pub fn get_world_border_mut(&mut self) -> &mut WorldBorder {
        &mut self.world_border
    }

    pub fn dimension(&self) -> &ResourceKey {
        &self.dimension
    }

    pub fn dimension_type_registration(&self) -> &Holder {
        &self.dimension_type
    }

    pub fn server_chunk_radius(&self) -> i32 {
        self.server_chunk_radius
    }

    pub fn get_biome_zoom_seed(&self) -> i64 {
        self.biome_zoom_seed
    }

    /// `getLevelData().isHardcore()`.
    pub fn is_hardcore(&self) -> bool {
        self.hardcore
    }

    /// `getLevelData().isFlat()`.
    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn get_sea_level(&self) -> i32 {
        self.sea_level
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    /// `getLevelData().getGameTime()`.
    pub fn get_game_time(&self) -> i64 {
        self.game_time
    }

    pub fn set_time_from_server(&mut self, game_time: i64) {
        self.game_time = game_time;
    }

    pub fn get_respawn_data(&self) -> &RespawnData {
        &self.respawn_data
    }

    pub fn set_respawn_data(&mut self, respawn_data: RespawnData) {
        self.respawn_data = respawn_data;
    }

    pub fn set_server_simulation_distance(&mut self, server_simulation_distance: i32) {
        self.server_simulation_distance = server_simulation_distance;
    }

    pub fn get_chunk_source(&self) -> &ClientChunkCache {
        &self.chunk_source
    }

    pub fn get_chunk_source_mut(&mut self) -> &mut ClientChunkCache {
        &mut self.chunk_source
    }

    /// `Level.isInValidBounds`: in the build height, and in a chunk that's valid.
    pub fn is_in_valid_bounds(&self, pos: &BlockPos) -> bool {
        self.is_inside_build_height(pos.get_y())
            && ChunkPos::is_valid(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )
    }

    /// A block the server says is there. Java checks the player's own predictions first, which
    /// aren't ported, so it's always `Level.setBlock`.
    pub fn set_server_verified_block_state(
        &mut self,
        pos: &BlockPos,
        block_state: BlockState,
        update_flag: i32,
    ) {
        self.set_block(pos, block_state, update_flag, 512);
    }

    /// `Level.setBlock`: whether the chunk took the state. The neighbour and shape updates, and
    /// what the renderer is told, aren't ported.
    pub fn set_block(
        &mut self,
        pos: &BlockPos,
        block_state: BlockState,
        _update_flags: i32,
        _update_limit: i32,
    ) -> bool {
        if !self.is_in_valid_bounds(pos) {
            return false;
        }

        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        // Java's level hands out its empty chunk for one it doesn't have, which takes no blocks.
        match self
            .chunk_source
            .get_chunk_and_light_engine_mut(chunk_x, chunk_z)
        {
            Some((chunk, light_engine)) => chunk
                .set_block_state(pos, block_state, light_engine)
                .is_some(),
            None => false,
        }
    }
}

impl LevelHeightAccessor for ClientLevel {
    fn get_height(&self) -> i32 {
        self.height.get_height()
    }

    fn get_min_y(&self) -> i32 {
        self.height.get_min_y()
    }
}

/// `Level.getBlockState` and `getFluidState`: void air (and no fluid) out of the level's bounds
/// and in a chunk it doesn't have (Java's `EmptyLevelChunk`), the chunk's otherwise.
impl BlockGetter for ClientLevel {
    #[inline]
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        if !self.is_in_valid_bounds(pos) {
            return Blocks::VOID_AIR.default_block_state();
        }

        match self.chunk_source.get_chunk(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            Some(chunk) => chunk.get_block_state(pos),
            None => Blocks::VOID_AIR.default_block_state(),
        }
    }

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        if !self.is_in_valid_bounds(pos) {
            return FluidState::EMPTY;
        }

        match self.chunk_source.get_chunk(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            Some(chunk) => chunk.get_fluid_state(pos),
            None => FluidState::EMPTY,
        }
    }
}

impl BlockAndLightGetter for ClientLevel {
    /// `Level.getLightEngine()`: the chunk source's.
    fn get_light_engine(&self) -> &LevelLightEngine {
        self.chunk_source.get_light_engine()
    }
}

impl BlockAndTintGetter for ClientLevel {
    /// `dimensionType().cardinalLightType().get()`.
    fn cardinal_lighting(&self) -> CardinalLighting {
        DimensionType::cardinal_light_type_of(&self.dimension_type).get()
    }
}

/// What collisions ask: the chunk's blocks, the border, and the entities' shapes, of which there
/// are none yet.
impl CollisionGetter for ClientLevel {
    fn get_world_border(&self) -> &WorldBorder {
        &self.world_border
    }

    /// `getChunkForCollisions`: `getChunk(x, z, FULL, false)`.
    fn get_chunk_for_collisions(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn BlockGetter> {
        self.chunk_source
            .get_chunk(chunk_x, chunk_z)
            .map(|chunk| chunk as &dyn BlockGetter)
    }

    fn get_entity_collisions(
        &self,
        _source: Option<&dyn EntityClass>,
        _test_area: &AABB,
    ) -> Vec<VoxelShape> {
        Vec::new()
    }
}

impl Level for ClientLevel {
    fn is_client_side(&self) -> bool {
        true
    }

    fn get_min_y(&self) -> i32 {
        LevelHeightAccessor::get_min_y(self)
    }

    /// `hasChunkAt(pos)`: whether the chunk source has the block's chunk.
    fn has_chunk_at(&self, pos: &BlockPos) -> bool {
        self.chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )
            .is_some()
    }

    /// `EntityFluidInteraction`'s scan of one chunk: whether any of its sections from `section_y0`
    /// to `section_y1` that the level has holds a fluid.
    fn chunk_has_fluid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        section_y0: i32,
        section_y1: i32,
    ) -> Option<bool> {
        let chunk = self.chunk_source.get_chunk(chunk_x, chunk_z)?;
        let sections = chunk.get_sections();
        let mut has_fluid = false;
        for section_y in section_y0..=section_y1 {
            let section_index = chunk.get_section_index_from_section_y(section_y);
            if section_index >= 0 && (section_index as usize) < sections.len() {
                has_fluid |= sections[section_index as usize].has_fluid();
            }
        }
        Some(has_fluid)
    }
}

impl fmt::Debug for ClientLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientLevel")
            .field("dimension", &self.dimension)
            .field("dimension_type", &self.dimension_type)
            .field("chunks", &self.chunk_source.get_loaded_chunks_count())
            .finish_non_exhaustive()
    }
}
