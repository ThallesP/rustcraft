//! net/minecraft/client/multiplayer/ClientLevel.java
//!
//! Not ported yet: nearly all of it. So far a level is what the login said about it, its chunks
//! (`ClientChunkCache`), reading and setting their blocks, their light (the chunk cache's
//! `LevelLightEngine`, which the queued light updates feed and `update` runs, once a frame), and
//! what an entity's movement asks of it (`CollisionGetter` and the entity's `Level`; its world
//! border is the default one until the border packets set it), and its other entities: the
//! entity storage (`TransientEntitySectionManager`'s lookup, `getEntity`, `entitiesForRendering`)
//! and the ticking ones (`EntityTickList`, `tickEntities`). An entity of a class not ported yet is
//! a plain `Entity` of its type; ticking one is its `commonTick`, its class's `tick` being what isn't
//! ported, so it stays where the server put it. The local player isn't in the storage here:
//! `Minecraft` holds it and ticks it first, as it is in Java's tick list, added at the login. An
//! entity ticks from when it's added, where Java waits for its chunk to tick (the chunk is loaded
//! first in all the server sends). Its block entities are its chunks' (`getBlockEntity`). Not
//! ported: entity sections and their visibility, other entities' collisions with the player,
//! block entities' ticking, the block state predictions (`blockStatePredictionHandler`, which
//! the player's own block changes make) and neighbour updates. What the renderer is told of the
//! level's changes (`setBlocksDirty`, `sendBlockUpdated`, `setSectionDirtyWithNeighbors`,
//! `setSectionRangeDirty`) goes to the `LevelExtractor` the level shares with its chunk cache and
//! the level renderer, Java's `Minecraft.levelExtractor`.
//!
//! Its biomes are its chunks' (`getNoiseBiome`, plains where it has no chunk), zoomed by its
//! `BiomeManager`, and their values the synchronized biome registry's, which the level keeps a
//! copy of. Its tints (`getBlockTint`) are `BiomeColors`' four resolvers' blends, each cached in a
//! `BlockTintCache`; the blend radius is the options', which Java reads from the game
//! (`Minecraft.getInstance().options`) and the level is given (`set_biome_blend_radius`).
//!
//! Java's light updates are `Runnable`s that close over the packet listener; here they're closures
//! of the level (`queue_light_update`), since all they touch is the level.
//!
//! The game time and the spawn are what the server last sent. The level's tick (`tickTime`, which
//! adds a game tick and ticks the listener's clocks) isn't ported, and `setRespawnData` keeps the
//! spawn as it came, where Java moves one outside the world border to the heightmap's top at the
//! border's center (`getWorldBorderAdjustedRespawnData`), which waits for the heightmaps.
//!
//! Its dimension type is the synchronized registry's value of the holder the login named
//! (`dimensionType()`), which gives its height, sky light, cardinal lighting and fast lava.

use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use crate::net::minecraft::client::color::block::block_tint_cache::BlockTintCache;
use crate::net::minecraft::client::multiplayer::client_chunk_cache::ClientChunkCache;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::renderer::biome_colors;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::extract::level_extractor::LevelExtractor;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::cursor3d::Cursor3D;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::world::entity::entity::{Entity, EntityClass};
use crate::net::minecraft::world::level::biome::biome::Biome;
use crate::net::minecraft::world::level::biome::biome_manager::BiomeManager;
use crate::net::minecraft::world::level::biome::biome_resolver::BiomeResolver;
use crate::net::minecraft::world::level::biome::biomes::Biomes;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::border::world_border::WorldBorder;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;
use crate::net::minecraft::world::level::entity::entity_lookup::EntityLookup;
use crate::net::minecraft::world::level::entity::entity_tick_list::EntityTickList;
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
    dimension_type_value: DimensionType,
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
    biomes: Registry,
    biome_manager: BiomeManager,
    tint_caches: [(&'static ColorResolver, BlockTintCache); 4],
    biome_blend_radius: i32,
    /// `entityStorage`'s entities and `tickingEntities`, the player's aside.
    entity_storage: EntityLookup<Entity>,
    ticking_entities: EntityTickList,
    /// `Minecraft.levelExtractor`, which the level's changes mark sections dirty in.
    level_extractor: Rc<LevelExtractor>,
}

/// A light update queued for the level, Java's `Runnable`.
pub type LightUpdate = Box<dyn FnOnce(&mut ClientLevel)>;

impl ClientLevel {
    /// Panics on registries without the dimension type's value, or without biomes.
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
        let dimension_type_value = registry_access
            .lookup_or_throw(Registries::DIMENSION_TYPE)
            .value::<DimensionType>(&dimension_type)
            .clone();
        let height = dimension_type_value.level_height();
        let factory = PalettedContainerFactory::create(registry_access);
        let biomes = registry_access.lookup_or_throw(Registries::BIOME).clone();
        let has_sky_light = dimension_type_value.has_sky_light();
        let level_extractor = Rc::new(LevelExtractor::new());
        ClientLevel {
            dimension,
            dimension_type,
            dimension_type_value,
            height,
            chunk_source: ClientChunkCache::new(
                height,
                has_sky_light,
                factory,
                server_chunk_radius,
                Rc::clone(&level_extractor),
                std::sync::Arc::new(registry_access.clone()),
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
            biomes,
            biome_manager: BiomeManager::new(biome_zoom_seed),
            tint_caches: [
                (&biome_colors::GRASS_COLOR_RESOLVER, BlockTintCache::new()),
                (&biome_colors::FOLIAGE_COLOR_RESOLVER, BlockTintCache::new()),
                (
                    &biome_colors::DRY_FOLIAGE_COLOR_RESOLVER,
                    BlockTintCache::new(),
                ),
                (&biome_colors::WATER_COLOR_RESOLVER, BlockTintCache::new()),
            ],
            biome_blend_radius: Options::DEFAULT_BIOME_BLEND_RADIUS,
            entity_storage: EntityLookup::new(),
            ticking_entities: EntityTickList::new(),
            level_extractor,
        }
    }

    /// `addEntity(Entity)`: in place of one it had by the id, ticking from now on.
    pub fn add_entity(&mut self, entity: Entity) {
        self.remove_entity(entity.get_id());
        self.ticking_entities.add(entity.get_id());
        self.entity_storage.add(entity);
    }

    /// `removeEntity(int, RemovalReason)`: gone from the storage and the tick list
    /// (`setRemoved`'s callbacks).
    pub fn remove_entity(&mut self, id: i32) {
        if let Some(entity) = self.entity_storage.get_entity(id) {
            let uuid = entity.get_uuid();
            self.ticking_entities.remove(id);
            self.entity_storage.remove(id, uuid);
        }
    }

    /// `getEntity(int)`, the player's aside.
    pub fn get_entity(&self, id: i32) -> Option<&Entity> {
        self.entity_storage.get_entity(id)
    }

    pub fn get_entity_mut(&mut self, id: i32) -> Option<&mut Entity> {
        self.entity_storage.get_entity_mut(id)
    }

    /// `entitiesForRendering()`: the entities in the order they were added, the player's aside.
    pub fn entities_for_rendering(&self) -> impl Iterator<Item = &Entity> {
        self.entity_storage.get_all_entities()
    }

    /// `tickEntities()` for the entities besides the player, in the tick list's order:
    /// `tickNonPassenger`'s `commonTick`; their classes' `tick` isn't ported.
    pub fn tick_entities(&mut self) {
        for id in self.ticking_entities.for_each() {
            if let Some(entity) = self.entity_storage.get_entity_mut(id) {
                entity.common_tick();
            }
        }
    }

    /// The options' `biomeBlendRadius`, which the tints blend over.
    pub fn set_biome_blend_radius(&mut self, biome_blend_radius: i32) {
        self.biome_blend_radius = biome_blend_radius;
    }

    /// `getBiome(pos)`: the biome the level's `BiomeManager` picks for the block.
    pub fn get_biome(&self, pos: &BlockPos) -> Holder {
        self.biome_manager.get_biome(self, pos)
    }

    /// `Holder.value()` of one of the level's biomes.
    pub fn biome_value(&self, biome: &Holder) -> &Biome {
        self.biomes.value::<Biome>(biome)
    }

    /// `getUncachedNoiseBiome`: plains, where the level has no chunk.
    pub fn get_uncached_noise_biome(&self, _quart_x: i32, _quart_y: i32, _quart_z: i32) -> Holder {
        self.biomes.get_or_throw(Biomes::PLAINS)
    }

    /// `onChunkLoaded`: the tints around the chunk are worked out again. Java also starts its
    /// entities ticking, which aren't ported.
    pub fn on_chunk_loaded(&self, pos: ChunkPos) {
        for (_, cache) in &self.tint_caches {
            cache.invalidate_for_chunk(pos.x(), pos.z());
        }
    }

    pub fn clear_tint_caches(&self) {
        for (_, cache) in &self.tint_caches {
            cache.invalidate_all();
        }
    }

    /// The resolver's colour at the block, averaged over the blend radius's square of columns
    /// around it at its height.
    pub fn calculate_block_tint(&self, pos: &BlockPos, color_resolver: &ColorResolver) -> i32 {
        let dist = self.biome_blend_radius;
        if dist == 0 {
            return color_resolver.get_color(
                self.biome_value(&self.get_biome(pos)),
                f64::from(pos.get_x()),
                f64::from(pos.get_z()),
            );
        }

        let count = (dist * 2 + 1) * (dist * 2 + 1);
        let mut total_red = 0;
        let mut total_green = 0;
        let mut total_blue = 0;
        let mut cursor = Cursor3D::new(
            pos.get_x() - dist,
            pos.get_y(),
            pos.get_z() - dist,
            pos.get_x() + dist,
            pos.get_y(),
            pos.get_z() + dist,
        );

        while cursor.advance() {
            let next_pos = BlockPos::new(cursor.next_x(), cursor.next_y(), cursor.next_z());
            let color = color_resolver.get_color(
                self.biome_value(&self.get_biome(&next_pos)),
                f64::from(next_pos.get_x()),
                f64::from(next_pos.get_z()),
            );
            total_red += ARGB::red(color);
            total_green += ARGB::green(color);
            total_blue += ARGB::blue(color);
        }

        ARGB::color_rgb(total_red / count, total_green / count, total_blue / count)
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

    /// Whether light updates are queued, which `update` runs some of a frame.
    pub fn has_light_updates(&self) -> bool {
        !self.light_update_queue.is_empty()
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

    /// `dimensionType()`: the registration's value.
    pub fn dimension_type(&self) -> &DimensionType {
        &self.dimension_type_value
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

    /// `Level.getBlockEntity(pos)`: the chunk's block entity there, made if the block makes one
    /// (`IMMEDIATE`); none out of bounds or where the level has no chunk.
    pub fn get_block_entity(&mut self, pos: &BlockPos) -> Option<&mut BlockEntity> {
        if !self.is_in_valid_bounds(pos) {
            return None;
        }
        self.chunk_source
            .get_chunk_mut(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )?
            .get_block_entity_immediate(pos)
    }

    /// `BlockGetter.getBlockEntity(pos, type)`: the block entity there, if it's of the type.
    pub fn get_block_entity_typed(
        &mut self,
        pos: &BlockPos,
        r#type: BlockEntityType,
    ) -> Option<&mut BlockEntity> {
        self.get_block_entity(pos)
            .filter(|block_entity| block_entity.get_type() == r#type)
    }

    /// The block entity there if the chunk has one, without making it (`CHECK`), which what
    /// only reads the level (the shapes, the renderer) asks.
    pub fn get_existing_block_entity(&self, pos: &BlockPos) -> Option<&BlockEntity> {
        self.chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )?
            .get_block_entity(pos)
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

    /// `Level.setBlock`: whether the chunk took the state, and the renderer told of the change
    /// (`setBlocksDirty`, and `sendBlockUpdated` with `Block.UPDATE_CLIENTS` but not
    /// `UPDATE_INVISIBLE`). The neighbour and shape updates aren't ported.
    pub fn set_block(
        &mut self,
        pos: &BlockPos,
        block_state: BlockState,
        update_flags: i32,
        _update_limit: i32,
    ) -> bool {
        if !self.is_in_valid_bounds(pos) {
            return false;
        }

        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        // Java's level hands out its empty chunk for one it doesn't have, which takes no blocks.
        let old_state = match self
            .chunk_source
            .get_chunk_and_light_engine_mut(chunk_x, chunk_z)
        {
            Some((chunk, light_engine)) => chunk.set_block_state(pos, block_state, light_engine),
            None => None,
        };
        let Some(old_state) = old_state else {
            return false;
        };

        let new_state = self.get_block_state(pos);
        if new_state == block_state {
            if old_state != new_state {
                self.set_blocks_dirty(pos, old_state, new_state);
            }

            if (update_flags & 2) != 0 && (update_flags & 4) == 0 {
                self.send_block_updated(pos, old_state, block_state, update_flags);
            }
        }
        true
    }

    /// `sendBlockUpdated`: the sections around the block are dirty.
    pub fn send_block_updated(
        &self,
        pos: &BlockPos,
        _old: BlockState,
        _current: BlockState,
        update_flags: i32,
    ) {
        self.level_extractor.block_changed(pos, update_flags);
    }

    /// `setBlocksDirty`: the sections around the block are dirty when its look changed.
    pub fn set_blocks_dirty(&self, pos: &BlockPos, old_state: BlockState, new_state: BlockState) {
        self.level_extractor
            .set_block_dirty_states(pos, old_state, new_state);
    }

    pub fn set_section_dirty_with_neighbors(&self, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
        self.level_extractor
            .set_section_dirty_with_neighbors(chunk_x, chunk_y, chunk_z);
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
        self.level_extractor.set_section_range_dirty(
            min_section_x,
            min_section_y,
            min_section_z,
            max_section_x,
            max_section_y,
            max_section_z,
        );
    }

    /// `Minecraft.levelExtractor`'s dirty sections, which the level renderer takes each frame.
    pub fn level_extractor(&self) -> &Rc<LevelExtractor> {
        &self.level_extractor
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
        self.dimension_type_value.cardinal_light_type().get()
    }

    /// The resolver's cached tint. Panics for a resolver that isn't one of `BiomeColors`', where
    /// Java's map has no cache and throws `NullPointerException`.
    fn get_block_tint(&self, pos: &BlockPos, resolver: &'static ColorResolver) -> i32 {
        let (_, cache) = self
            .tint_caches
            .iter()
            .find(|(cached, _)| ColorResolver::same(cached, resolver))
            .unwrap_or_else(|| panic!("no tint cache for the resolver {}", resolver.name()));
        cache.get_color(pos, |pos| self.calculate_block_tint(pos, resolver))
    }
}

/// `LevelReader.getNoiseBiome`: the chunk's biome at the quart, or the uncached one where the
/// level has no chunk.
impl BiomeResolver for ClientLevel {
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        match self
            .chunk_source
            .get_chunk(QuartPos::to_section(quart_x), QuartPos::to_section(quart_z))
        {
            Some(chunk) => chunk.get_noise_biome(quart_x, quart_y, quart_z),
            None => self.get_uncached_noise_biome(quart_x, quart_y, quart_z),
        }
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

    /// `environmentAttributes().getDimensionValue(FAST_LAVA)`: the dimension type's.
    fn has_fast_lava(&self) -> bool {
        self.dimension_type_value.fast_lava()
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

#[cfg(test)]
mod tests {
    use super::ClientLevel;
    use crate::golden::{Decode, Golden, unhex};
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
    use crate::net::minecraft::client::renderer::biome_colors;
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_chunks_biomes_packet::ClientboundChunksBiomesPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::biome::biome::tests::biome_registries;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;
    use crate::net::minecraft::world::level::color_resolver::ColorResolver;

    const HEIGHTS: [i32; 3] = [-61, 0, 70];

    /// The walk's chunks, biomes laid on them by a `chunks_biomes` packet, and every resolver's
    /// tint at each column (at three heights) as vanilla's `calculateBlockTint` blends them: from
    /// the level's tint caches, so read twice (the second time cached), and the second layout
    /// laid on the first's level, whose tints the packet's chunks must forget.
    #[test]
    fn tints_match_vanillas_blend() {
        load_colormaps();
        let chunk_golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = biome_registries(&chunk_golden.input("BIOMES"));
        let mut packets = Vec::new();
        for (args, _) in chunk_golden.cases("captured") {
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            packets.push(ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input));
        }
        let resolvers: [&'static ColorResolver; 4] = [
            &biome_colors::GRASS_COLOR_RESOLVER,
            &biome_colors::FOLIAGE_COLOR_RESOLVER,
            &biome_colors::DRY_FOLIAGE_COLOR_RESOLVER,
            &biome_colors::WATER_COLOR_RESOLVER,
        ];

        let golden = Golden::load("net.minecraft.client.multiplayer.ClientLevel");
        let layouts = golden.cases("layouts");
        assert_eq!(layouts.len(), 4);
        let mut level = None;
        let mut level_radius = -1;
        for (number, (args, expected)) in layouts.iter().enumerate() {
            let radius = args[0].as_i64().expect("a radius") as i32;
            if radius != level_radius {
                let mut new_level = ClientLevel::new(
                    &registries,
                    false,
                    true,
                    ResourceKey::create(
                        Registries::DIMENSION,
                        Identifier::with_default_namespace("overworld"),
                    ),
                    Holder::Reference(ResourceKey::create(
                        Registries::DIMENSION_TYPE,
                        Identifier::with_default_namespace("overworld"),
                    )),
                    8,
                    8,
                    false,
                    i64::decode(&golden.input("ZOOM_SEED")),
                    -63,
                );
                new_level.set_biome_blend_radius(radius);
                for packet in &packets {
                    ClientPacketListener::handle_level_chunk_with_light_in(&mut new_level, packet);
                }
                level = Some(new_level);
                level_radius = radius;
            }
            let level = level.as_mut().expect("made above");
            let mut buffer = unpooled::wrapped_buffer(unhex(args[1].as_str().expect("hex")));
            let biomes = ClientboundChunksBiomesPacket::STREAM_CODEC.decode(&mut buffer);
            ClientPacketListener::handle_chunks_biomes_in(level, &biomes);

            let expected = expected.as_array().expect("tints");
            for pass in 0..2 {
                let mut index = 0;
                for column in args[2].as_array().expect("columns") {
                    let x = column[0].as_i64().expect("x") as i32;
                    let z = column[1].as_i64().expect("z") as i32;
                    for y in HEIGHTS {
                        for resolver in resolvers {
                            let pos = BlockPos::new(x, y, z);
                            assert_eq!(
                                i64::from(level.get_block_tint(&pos, resolver)),
                                expected[index].as_i64().expect("a tint"),
                                "layout {number} (pass {pass}): {} at {pos:?}",
                                resolver.name()
                            );
                            index += 1;
                        }
                    }
                }
                assert_eq!(index, expected.len());
            }
        }
        golden.finish();
    }
}
