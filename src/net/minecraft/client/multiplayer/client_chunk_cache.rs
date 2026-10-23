//! net/minecraft/client/multiplayer/ClientChunkCache.java
//!
//! The chunks the server sent, in an array as wide as the view distance, indexed by chunk
//! position modulo its width, so moving the view center needs no copying, and the level's light
//! engine, which lights them (the storage is its `LightChunkGetter`). Not ported yet: the empty
//! chunk Java hands out for one it doesn't have (`EmptyLevelChunk`, whose blocks are all void air:
//! `get_chunk` answers `None`, and the level answers void air) and the sets of sections and chunks added and removed since the renderer last looked
//! (`addedEmptySections` and the rest, `flipUpdateTrackingSets`), which come with the renderer's
//! section compiler. A chunk dropped or replaced unloads in Java's level (`ClientLevel.unload`),
//! which turns its light off; its block entities and entities aren't ported.

use std::rc::Rc;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::renderer::extract::level_extractor::LevelExtractor;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::net::minecraft::world::level::chunk::level_chunk::LevelChunk;
use crate::net::minecraft::world::level::chunk::light_chunk::LightChunk;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::Height;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;

/// The level's chunks, as far as the server's view distance reaches, and their light.
pub struct ClientChunkCache {
    light_engine: LevelLightEngine,
    storage: Storage,
    level_height: Height,
    container_factory: PalettedContainerFactory,
}

impl ClientChunkCache {
    /// `ClientChunkCache(level, serverChunkRadius)`, for a level of that height whose sections
    /// number their biomes by the factory's registry, with sky light if its dimension has a sky.
    pub fn new(
        level_height: Height,
        has_sky_light: bool,
        container_factory: PalettedContainerFactory,
        server_chunk_radius: i32,
        level_extractor: Rc<LevelExtractor>,
    ) -> ClientChunkCache {
        ClientChunkCache {
            light_engine: LevelLightEngine::new(level_height, true, has_sky_light),
            storage: Storage::new(
                ClientChunkCache::calculate_storage_range(server_chunk_radius),
                level_extractor,
            ),
            level_height,
            container_factory,
        }
    }

    pub fn get_light_engine(&self) -> &LevelLightEngine {
        &self.light_engine
    }

    /// The light engine, with the chunks it lights, which its methods that read chunks take.
    pub fn get_light_engine_mut(&mut self) -> (&mut LevelLightEngine, &dyn LightChunkGetter) {
        (&mut self.light_engine, &self.storage)
    }

    /// A chunk to set blocks in, with the light engine its `setBlockState` tells.
    pub fn get_chunk_and_light_engine_mut(
        &mut self,
        x: i32,
        z: i32,
    ) -> Option<(&mut LevelChunk, &mut LevelLightEngine)> {
        let light_engine = &mut self.light_engine;
        self.storage
            .get_chunk_mut(x, z)
            .map(|chunk| (chunk, light_engine))
    }

    fn is_valid_chunk(chunk: Option<&LevelChunk>, x: i32, z: i32) -> bool {
        chunk.is_some_and(|chunk| {
            let pos = chunk.get_pos();
            pos.x() == x && pos.z() == z
        })
    }

    pub fn drop(&mut self, pos: ChunkPos) {
        if self.storage.in_range(pos.x(), pos.z()) {
            let index = self.storage.get_index(pos.x(), pos.z());
            if ClientChunkCache::is_valid_chunk(
                self.storage.chunks[index].as_ref(),
                pos.x(),
                pos.z(),
            ) {
                self.storage.drop(index);
                self.unload(pos);
            }
        }
    }

    /// `ClientLevel.unload(chunk)`'s part that's ported: the chunk's light goes off.
    fn unload(&mut self, pos: ChunkPos) {
        self.light_engine
            .set_light_enabled(&self.storage, pos, false);
    }

    /// `getChunk(x, z, status, false)`: the chunk at that position, if the cache has it.
    #[inline]
    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&LevelChunk> {
        self.storage.get_chunk(x, z)
    }

    pub fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut LevelChunk> {
        self.storage.get_chunk_mut(x, z)
    }

    /// The chunk's biomes, from the server's: ignored, like Java (which logs a warning), for a
    /// chunk out of range or that the cache doesn't have.
    pub fn replace_biomes(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        read_buffer: &mut FriendlyByteBuf,
    ) {
        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            chunk.replace_biomes(read_buffer);
        }
    }

    /// The server's chunk, read into the chunk the cache has there, or a new one. A chunk out of
    /// the view range is ignored (Java logs a warning).
    pub fn replace_with_packet_data(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        chunk_data: &ClientboundLevelChunkPacketData,
    ) -> Option<&LevelChunk> {
        if !self.storage.in_range(chunk_x, chunk_z) {
            return None;
        }

        let index = self.storage.get_index(chunk_x, chunk_z);
        if ClientChunkCache::is_valid_chunk(self.storage.chunks[index].as_ref(), chunk_x, chunk_z) {
            let chunk = self.storage.chunks[index]
                .as_mut()
                .expect("a valid chunk is there");
            chunk.replace_with_packet_data(chunk_x, chunk_z, chunk_data);
        } else {
            let mut chunk = LevelChunk::new(
                self.level_height,
                &self.container_factory,
                ChunkPos::new(chunk_x, chunk_z),
            );
            chunk.replace_with_packet_data(chunk_x, chunk_z, chunk_data);
            if let Some(removed_chunk) = self.storage.replace(index, Some(chunk)) {
                self.unload(removed_chunk);
            }
        }

        self.storage.chunks[index].as_ref()
    }

    pub fn update_view_center(&mut self, x: i32, z: i32) {
        self.storage.view_center_x = x;
        self.storage.view_center_z = z;
    }

    /// A storage for the new range, keeping the chunks that are in it.
    pub fn update_view_radius(&mut self, view_range: i32) {
        let chunk_radius = self.storage.chunk_radius;
        let new_chunk_radius = ClientChunkCache::calculate_storage_range(view_range);
        if chunk_radius != new_chunk_radius {
            let mut new_storage =
                Storage::new(new_chunk_radius, Rc::clone(&self.storage.level_extractor));
            new_storage.view_center_x = self.storage.view_center_x;
            new_storage.view_center_z = self.storage.view_center_z;

            for chunk in std::mem::take(&mut self.storage.chunks)
                .into_iter()
                .flatten()
            {
                let pos = chunk.get_pos();
                if new_storage.in_range(pos.x(), pos.z()) {
                    let index = new_storage.get_index(pos.x(), pos.z());
                    // Each slot of the new storage takes one of the chunks in its range.
                    let _ = new_storage.replace(index, Some(chunk));
                }
            }

            self.storage = new_storage;
        }
    }

    fn calculate_storage_range(view_range: i32) -> i32 {
        view_range.max(2) + 3
    }

    pub fn get_loaded_chunks_count(&self) -> i32 {
        self.storage.chunk_count
    }

    /// `gatherStats()`: the storage's size and how many chunks it holds.
    pub fn gather_stats(&self) -> String {
        format!(
            "{}, {}",
            self.storage.chunks.len(),
            self.get_loaded_chunks_count()
        )
    }
}

/// `ClientChunkCache.Storage`: `viewRange²` slots, a chunk's at its position modulo `viewRange`.
struct Storage {
    chunks: Vec<Option<LevelChunk>>,
    chunk_radius: i32,
    view_range: i32,
    view_center_x: i32,
    view_center_z: i32,
    chunk_count: i32,
    /// `Minecraft.levelExtractor`, which `onLightUpdate` tells.
    level_extractor: Rc<LevelExtractor>,
}

impl Storage {
    fn new(chunk_radius: i32, level_extractor: Rc<LevelExtractor>) -> Storage {
        let view_range = chunk_radius * 2 + 1;
        Storage {
            chunks: (0..view_range * view_range).map(|_| None).collect(),
            chunk_radius,
            view_range,
            view_center_x: 0,
            view_center_z: 0,
            chunk_count: 0,
            level_extractor,
        }
    }

    #[inline]
    fn get_index(&self, chunk_x: i32, chunk_z: i32) -> usize {
        (Math::floor_mod_i32(chunk_z, self.view_range) * self.view_range
            + Math::floor_mod_i32(chunk_x, self.view_range)) as usize
    }

    /// Puts the chunk in the slot: where the chunk it replaced was, which its level unloads.
    fn replace(&mut self, index: usize, new_chunk: Option<LevelChunk>) -> Option<ChunkPos> {
        let added = new_chunk.is_some();
        let removed_chunk = std::mem::replace(&mut self.chunks[index], new_chunk);
        if removed_chunk.is_some() {
            self.chunk_count -= 1;
        }

        if added {
            self.chunk_count += 1;
        }

        removed_chunk.map(|chunk| chunk.get_pos())
    }

    fn get_chunk(&self, x: i32, z: i32) -> Option<&LevelChunk> {
        if self.in_range(x, z) {
            let chunk = self.chunks[self.get_index(x, z)].as_ref();
            if ClientChunkCache::is_valid_chunk(chunk, x, z) {
                return chunk;
            }
        }

        None
    }

    fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut LevelChunk> {
        if self.in_range(x, z) {
            let index = self.get_index(x, z);
            if ClientChunkCache::is_valid_chunk(self.chunks[index].as_ref(), x, z) {
                return self.chunks[index].as_mut();
            }
        }

        None
    }

    fn drop(&mut self, index: usize) {
        if self.chunks[index].take().is_some() {
            self.chunk_count -= 1;
        }
    }

    #[inline]
    fn in_range(&self, chunk_x: i32, chunk_z: i32) -> bool {
        Math::abs_i32(chunk_x.wrapping_sub(self.view_center_x)) <= self.chunk_radius
            && Math::abs_i32(chunk_z.wrapping_sub(self.view_center_z)) <= self.chunk_radius
    }
}

/// `getChunkForLighting(x, z)`: the chunk, if the cache has it; `onLightUpdate` marks the section
/// dirty for the renderer.
impl LightChunkGetter for Storage {
    fn get_chunk_for_lighting(&self, x: i32, z: i32) -> Option<&dyn LightChunk> {
        self.get_chunk(x, z).map(|chunk| chunk as &dyn LightChunk)
    }

    fn on_light_update(&self, _layer: LightLayer, pos: SectionPos) {
        self.level_extractor
            .set_section_dirty(pos.x(), pos.y(), pos.z());
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use crate::golden::Golden;
    use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::block::blocks::Blocks;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::tests::registries;
    use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
    use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;

    /// A level of the overworld's height, with a storage of that radius.
    pub(crate) fn overworld(
        registries: &crate::net::minecraft::core::registry_access::RegistryAccess,
        radius: i32,
    ) -> ClientLevel {
        let dimension = ResourceKey::create(
            Registries::DIMENSION,
            Identifier::with_default_namespace("overworld"),
        );
        let dimension_type = Holder::Reference(ResourceKey::create(
            Registries::DIMENSION_TYPE,
            Identifier::with_default_namespace("overworld"),
        ));
        ClientLevel::new(
            registries,
            false,
            true,
            dimension,
            dimension_type,
            radius,
            radius,
            false,
            0,
            -63,
        )
    }

    // Runs `[[value, count], …]` as the values one by one.
    fn expand(runs: &Value) -> Vec<i64> {
        let mut values = Vec::new();
        for run in runs.as_array().expect("runs") {
            let value = run[0].as_i64().expect("an id");
            values.extend(std::iter::repeat_n(
                value,
                run[1].as_u64().expect("a count") as usize,
            ));
        }
        values
    }

    /// The walk's chunks, put in a level by the port the way the listener does, read back block
    /// by block through the level: the same states as vanilla's sections read from the same bytes
    /// (the chunk packet's goldens), at every position of every chunk, and void air past the
    /// level's height and in chunks it doesn't have, like Java's `Level.getBlockState`.
    #[test]
    fn the_walks_chunks_read_back_like_vanillas_sections() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&golden.input("BIOMES"));
        let mut level = overworld(&registries, 8);
        let mut chunks = Vec::new();
        for (args, result) in golden.cases("captured") {
            let mut input = registry_reader(crate::golden::unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            level
                .get_chunk_source_mut()
                .replace_with_packet_data(packet.x(), packet.z(), packet.chunk_data())
                .expect("the walk's chunks are all in range");
            chunks.push((packet.x(), packet.z(), result[0][5].clone()));
        }
        assert!(chunks.len() >= 100, "the capture has the walk's chunks");
        assert_eq!(
            level.get_chunk_source().get_loaded_chunks_count(),
            chunks.len() as i32
        );

        let min_y = level.get_min_y();
        let mut checked = 0;
        for (chunk_x, chunk_z, sections) in &chunks {
            for (index, section) in sections.as_array().expect("sections").iter().enumerate() {
                let states = expand(&section[2]);
                for (i, &expected) in states.iter().enumerate() {
                    let (x, z, y) = ((i & 15) as i32, ((i >> 4) & 15) as i32, (i >> 8) as i32);
                    let pos = BlockPos::new(
                        chunk_x * 16 + x,
                        min_y + index as i32 * 16 + y,
                        chunk_z * 16 + z,
                    );
                    assert_eq!(
                        level.get_block_state(&pos).id() as i64,
                        expected,
                        "at {pos:?}"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, chunks.len() * 24 * 4096);

        let void_air = Blocks::VOID_AIR.default_block_state();
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, min_y - 1, 0)),
            void_air
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, level.get_max_y() + 1, 0)),
            void_air
        );
        assert_eq!(
            level.get_block_state(&BlockPos::new(16 * 100, 0, 0)),
            void_air
        );
    }

    /// Block updates, dropping and the view: what the handlers do to the level.
    #[test]
    fn block_updates_drops_and_the_view_move_chunks_like_java() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&golden.input("BIOMES"));
        let mut level = overworld(&registries, 2);
        let (args, _) = golden
            .cases("captured")
            .into_iter()
            .next()
            .expect("a captured chunk");
        let mut input = registry_reader(crate::golden::unhex(args[0].as_str().expect("hex")));
        let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
        let (x, z) = (packet.x(), packet.z());
        let cache = level.get_chunk_source_mut();
        cache.replace_with_packet_data(x, z, packet.chunk_data());
        // Again, into the same chunk.
        cache.replace_with_packet_data(x, z, packet.chunk_data());
        assert_eq!(cache.get_loaded_chunks_count(), 1);
        // Range 2 makes a storage of radius 5 around the center: 11 by 11.
        assert_eq!(cache.gather_stats(), "121, 1");
        assert!(
            cache
                .replace_with_packet_data(x + 6, z, packet.chunk_data())
                .is_none()
        );

        let pos = BlockPos::new(x * 16 + 3, 100, z * 16 + 5);
        let stone = Blocks::STONE.default_block_state();
        assert!(level.set_block(&pos, stone, 19, 512));
        assert_eq!(level.get_block_state(&pos), stone);
        // Setting what's there already changes nothing.
        assert!(!level.set_block(&pos, stone, 19, 512));
        // Air into a section of only air changes nothing either.
        let high = BlockPos::new(x * 16, 300, z * 16);
        assert!(!level.set_block(&high, Blocks::AIR.default_block_state(), 19, 512));
        let water: BlockState = Blocks::WATER.default_block_state();
        level.set_server_verified_block_state(&pos, water, 19);
        assert!(level.get_fluid_state(&pos).is_source());

        // The view moves away: the chunk's still there until it's dropped or replaced.
        level.get_chunk_source_mut().update_view_radius(8);
        assert_eq!(level.get_chunk_source().gather_stats(), "529, 1");
        assert_eq!(level.get_block_state(&pos), water);
        level.get_chunk_source_mut().drop(ChunkPos::new(x, z));
        assert_eq!(level.get_chunk_source().get_loaded_chunks_count(), 0);
        assert_eq!(
            level.get_block_state(&pos),
            Blocks::VOID_AIR.default_block_state()
        );
        assert!(!level.set_block(&pos, stone, 19, 512));
    }
}
