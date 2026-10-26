//! net/minecraft/client/renderer/chunk/RenderSectionRegion.java
//!
//! The blocks a section's compile reads: its own and its 26 neighbours', copied, with the level's
//! light, tints and cardinal lighting. Java's region holds the `ClientLevel` and its light engine,
//! which its compile reads on a worker thread as they are then; the port's level isn't shared
//! between threads, so a region of the level (`RenderRegionCache::create_region`) copies what its
//! compile reads of it when it's made, which is what a compile run then would read:
//! - the 27 sections' light as the engine's visible layers show it (`SectionLight`);
//! - their biomes, the zoom, the blend radius and the biome registry, which tints are worked out
//!   from as `ClientLevel.calculateBlockTint` works them out;
//! - the level's cached tints of the section's column (`BlockTintCache::copy_layers`), which the
//!   compile reads like Java's reads the cache, adds what it works out to, and gives back to the
//!   level with its mesh (`TintsToGiveBack`).
//!
//! A region can also borrow a level that answers light and tints itself (`over`), which a test
//! stands in for the level with: the region's type says which (`L`), and only a copy goes to
//! another thread.
//! Block entities aren't ported (`SectionCopy`). The region's own section, which its compile reads
//! most, is decoded once into its states the first time it's read (`center`, on the thread that
//! compiles), the rest read from their copies' containers: the same states.

use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use crate::net::minecraft::client::color::block::block_tint_cache::{BLOCKS_PER_LAYER, TintLayers};
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::chunk::section_copy::SectionCopy;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::biome::biome_manager::BiomeManager;
use crate::net::minecraft::world::level::biome::biome_resolver::BiomeResolver;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::chunk::paletted_container::PalettedContainer;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::layer_light_event_listener::SectionLight;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

/// A region whose light and tints are `L`'s: the level's copied (`LevelCopy`), or a level's that
/// answers them itself (`&dyn BlockAndTintGetter`).
pub struct RenderSectionRegion<L = LevelCopy> {
    min_section_x: i32,
    min_section_y: i32,
    min_section_z: i32,
    sections: Vec<Arc<SectionCopy>>,
    /// The region's own section's states, decoded the first time they're read: on the thread
    /// that compiles it.
    center: OnceCell<Option<Box<[BlockState]>>>,
    level: L,
    cardinal_lighting: CardinalLighting,
}

/// The level's light and tints a region copies.
pub struct LevelCopy {
    /// The 27 sections' light and biomes, by `index`.
    sections: Vec<Arc<SectionLevelCopy>>,
    /// Whether the level has each of the 9 columns' chunk, by x then z.
    has_chunk: [bool; 9],
    tints: RegionTints,
}

/// A section's light and biomes as the level has them: the light the engine's visible layers
/// show there, and the biomes of the level's chunk there, none where the level has no chunk or
/// the section is outside the world.
pub struct SectionLevelCopy {
    block_light: SectionLight,
    sky_light: SectionLight,
    biomes: Option<PalettedContainer<Holder>>,
}

impl SectionLevelCopy {
    pub fn new(
        level: &ClientLevel,
        section_x: i32,
        section_y: i32,
        section_z: i32,
    ) -> SectionLevelCopy {
        let section_node = SectionPos::as_long_xyz(section_x, section_y, section_z);
        let light_engine = level.get_chunk_source().get_light_engine();
        let section_index = level.get_section_index_from_section_y(section_y);
        let biomes = level
            .get_chunk_source()
            .get_chunk(section_x, section_z)
            .and_then(|chunk| {
                chunk
                    .get_sections()
                    .get(usize::try_from(section_index).ok()?)
            })
            .map(|section| section.get_biomes().copy());
        SectionLevelCopy {
            block_light: light_engine
                .get_layer_listener(LightLayer::BLOCK)
                .copy_section_light(section_node),
            sky_light: light_engine
                .get_layer_listener(LightLayer::SKY)
                .copy_section_light(section_node),
            biomes,
        }
    }
}

/// What tints are worked out from, and the level's cached tints of the region's column at the
/// heights its compile asks for (its section's, and the one below for blocks tinted like the
/// block under them), each resolver's with the count of the cache's drops they're as of.
struct RegionTints {
    biomes: Arc<Registry>,
    biome_manager: BiomeManager,
    blend_radius: i32,
    /// `getUncachedNoiseBiome`'s, where the level has no chunk.
    plains: Holder,
    /// The level's height in quarts, which `ChunkAccess.getNoiseBiome` keeps a quart in.
    min_quart_y: i32,
    max_quart_y: i32,
    chunk_x: i32,
    chunk_z: i32,
    min_y: i32,
    resolvers: [&'static ColorResolver; 4],
    layers: [RefCell<(u64, TintLayers)>; 4],
}

/// The tints a compile worked out over its region's copy of the level's cache, for
/// `ClientLevel::give_back_tints`.
pub struct TintsToGiveBack {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub min_y: i32,
    /// Each resolver's layers, in the level's order, with the count of drops they're as of.
    pub layers: Vec<(u64, TintLayers)>,
}

impl RenderSectionRegion<LevelCopy> {
    /// A region of the level, with copies of its light, biomes and cached tints
    /// (`level_sections`, the 27 sections' by `index`, like `sections`).
    pub fn new(
        level: &ClientLevel,
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        sections: Vec<Arc<SectionCopy>>,
        level_sections: Vec<Arc<SectionLevelCopy>>,
    ) -> RenderSectionRegion<LevelCopy> {
        let chunk_x = min_section_x + 1;
        let chunk_z = min_section_z + 1;
        let min_y = SectionPos::section_to_block_coord(min_section_y + 1) - 1;
        let count = 17;
        let tint_caches = level.tint_caches();
        let has_chunk = std::array::from_fn(|column| {
            let x = min_section_x + (column % 3) as i32;
            let z = min_section_z + (column / 3) as i32;
            level.get_chunk_source().get_chunk(x, z).is_some()
        });
        let min_quart_y = QuartPos::from_block(level.get_min_y());
        let copy = LevelCopy {
            sections: level_sections,
            has_chunk,
            tints: RegionTints {
                biomes: Arc::clone(level.biome_registry()),
                biome_manager: level.biome_manager(),
                blend_radius: level.biome_blend_radius(),
                plains: level.get_uncached_noise_biome(0, 0, 0),
                min_quart_y,
                max_quart_y: min_quart_y + QuartPos::from_block(level.get_height()) - 1,
                chunk_x,
                chunk_z,
                min_y,
                resolvers: std::array::from_fn(|i| tint_caches[i].0),
                layers: std::array::from_fn(|i| {
                    RefCell::new(tint_caches[i].1.copy_layers(chunk_x, chunk_z, min_y, count))
                }),
            },
        };
        RenderSectionRegion {
            min_section_x,
            min_section_y,
            min_section_z,
            center: OnceCell::new(),
            sections,
            level: copy,
            cardinal_lighting: level.cardinal_lighting(),
        }
    }

    /// The tints its compile worked out over the level's cached ones, to give back to the level.
    pub fn into_tints(self) -> TintsToGiveBack {
        let tints = self.level.tints;
        TintsToGiveBack {
            chunk_x: tints.chunk_x,
            chunk_z: tints.chunk_z,
            min_y: tints.min_y,
            layers: tints.layers.into_iter().map(RefCell::into_inner).collect(),
        }
    }
}

impl<'a> RenderSectionRegion<&'a dyn BlockAndTintGetter> {
    /// A region whose light and tints `level` answers, read as its compile runs.
    pub fn over(
        level: &'a dyn BlockAndTintGetter,
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        sections: Vec<Arc<SectionCopy>>,
    ) -> RenderSectionRegion<&'a dyn BlockAndTintGetter> {
        RenderSectionRegion {
            min_section_x,
            min_section_y,
            min_section_z,
            center: OnceCell::new(),
            sections,
            level,
            cardinal_lighting: level.cardinal_lighting(),
        }
    }
}

/// The region's own section's index in `sections`.
const CENTER: usize = 13;

impl RenderSectionRegion {
    pub const RADIUS: i32 = 1;
    pub const SIZE: i32 = 3;

    pub fn index(
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        section_x: i32,
        section_y: i32,
        section_z: i32,
    ) -> usize {
        (section_x - min_section_x
            + (section_y - min_section_y) * 3
            + (section_z - min_section_z) * 3 * 3) as usize
    }
}

impl<L> RenderSectionRegion<L> {
    fn index_of(&self, pos: &BlockPos) -> usize {
        RenderSectionRegion::index(
            self.min_section_x,
            self.min_section_y,
            self.min_section_z,
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_y()),
            SectionPos::block_to_section_coord(pos.get_z()),
        )
    }
}

impl<L> BlockGetter for RenderSectionRegion<L> {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        let index = self.index_of(pos);
        if index == CENTER
            && let Some(center) = self.center.get_or_init(|| self.sections[CENTER].decode())
        {
            return center[((pos.get_y() & 15) << 8 | (pos.get_z() & 15) << 4 | (pos.get_x() & 15))
                as usize];
        }
        self.sections[index].get_block_state(pos)
    }

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        self.get_block_state(pos).get_fluid_state()
    }
}

/// A test level's light: `getLightEngine` is the level's, which `getBrightness` asks through the
/// level, so a level answering it itself is asked too.
impl BlockAndLightGetter for RenderSectionRegion<&dyn BlockAndTintGetter> {
    fn get_light_engine(&self) -> &LevelLightEngine {
        self.level.get_light_engine()
    }

    fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
        self.level.get_brightness(layer, pos)
    }

    fn get_raw_brightness(&self, pos: &BlockPos, darkening: i32) -> i32 {
        self.level.get_raw_brightness(pos, darkening)
    }
}

impl BlockAndTintGetter for RenderSectionRegion<&dyn BlockAndTintGetter> {
    fn cardinal_lighting(&self) -> CardinalLighting {
        self.cardinal_lighting
    }

    /// The level's.
    fn get_block_tint(&self, pos: &BlockPos, resolver: &'static ColorResolver) -> i32 {
        self.level.get_block_tint(pos, resolver)
    }
}

/// The level's light, copied: read by `get_brightness`, with no engine to hand out.
impl BlockAndLightGetter for RenderSectionRegion<LevelCopy> {
    fn get_light_engine(&self) -> &LevelLightEngine {
        panic!("a region's copy of the level's light is read by get_brightness")
    }

    fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
        let section = &self.level.sections[self.index_of(pos)];
        let light = match layer {
            LightLayer::BLOCK => &section.block_light,
            LightLayer::SKY => &section.sky_light,
        };
        light.get(pos.get_x() & 15, pos.get_y() & 15, pos.get_z() & 15)
    }

    /// `LevelLightEngine.getRawBrightness`: a level without the sky's light reads 0 for it.
    fn get_raw_brightness(&self, pos: &BlockPos, darkening: i32) -> i32 {
        self.get_brightness(LightLayer::BLOCK, pos)
            .max(self.get_brightness(LightLayer::SKY, pos) - darkening)
    }
}

impl BlockAndTintGetter for RenderSectionRegion<LevelCopy> {
    fn cardinal_lighting(&self) -> CardinalLighting {
        self.cardinal_lighting
    }

    /// The level's: from its cache, else worked out from the biomes and kept with the cache's.
    /// Panics for a resolver that isn't one of `BiomeColors`', like the level.
    fn get_block_tint(&self, pos: &BlockPos, resolver: &'static ColorResolver) -> i32 {
        let tints = &self.level.tints;
        let resolver_index = tints
            .resolvers
            .iter()
            .position(|cached| ColorResolver::same(cached, resolver))
            .unwrap_or_else(|| panic!("no tint cache for the resolver {}", resolver.name()));
        let calculate = || {
            ClientLevel::calculate_block_tint_in(
                &CopiedBiomes { region: self },
                &tints.biome_manager,
                &tints.biomes,
                tints.blend_radius,
                pos,
                resolver,
            )
        };
        let layers = &tints.layers[resolver_index];
        let layer_index = pos.get_y() - tints.min_y;
        if SectionPos::block_to_section_coord(pos.get_x()) != tints.chunk_x
            || SectionPos::block_to_section_coord(pos.get_z()) != tints.chunk_z
            || layer_index < 0
            || layer_index as usize >= layers.borrow().1.len()
        {
            return calculate();
        }

        let index = ((pos.get_z() & 15) << 4 | (pos.get_x() & 15)) as usize;
        let cached = layers.borrow().1[layer_index as usize]
            .as_ref()
            .map_or(-1, |layer| layer[index]);
        if cached != -1 {
            return cached;
        }

        let calculated = calculate();
        layers.borrow_mut().1[layer_index as usize]
            .get_or_insert_with(|| Box::new([-1; BLOCKS_PER_LAYER]))[index] = calculated;
        calculated
    }
}

/// The region's copy of the level's noise biomes: `LevelReader.getNoiseBiome`, the chunk's biome
/// at the quart (its y kept in the level's height), or plains where the level has no chunk.
/// Panics outside the region's 27 sections, which a tint's blend never reaches.
struct CopiedBiomes<'r> {
    region: &'r RenderSectionRegion<LevelCopy>,
}

impl BiomeResolver for CopiedBiomes<'_> {
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        let column_x = QuartPos::to_section(quart_x) - self.region.min_section_x;
        let column_z = QuartPos::to_section(quart_z) - self.region.min_section_z;
        assert!(
            (0..3).contains(&column_x) && (0..3).contains(&column_z),
            "a tint's biome outside the region"
        );
        let copy = &self.region.level;
        let tints = &copy.tints;
        if !copy.has_chunk[(column_x + 3 * column_z) as usize] {
            return tints.plains.clone();
        }

        let clamped_quart_y = Mth::clamp_i32(quart_y, tints.min_quart_y, tints.max_quart_y);
        let section_y = QuartPos::to_section(clamped_quart_y);
        let index = RenderSectionRegion::index(
            self.region.min_section_x,
            self.region.min_section_y,
            self.region.min_section_z,
            QuartPos::to_section(quart_x),
            section_y,
            QuartPos::to_section(quart_z),
        );
        assert!(
            (0..3).contains(&(section_y - self.region.min_section_y)),
            "a tint's biome outside the region"
        );
        copy.sections[index]
            .biomes
            .as_ref()
            .expect("a chunk's section in the world has biomes")
            .get(quart_x & 3, clamped_quart_y & 3, quart_z & 3)
    }
}

#[cfg(test)]
mod tests {
    use crate::golden::{Decode, Golden, unhex};
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
    use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
    use crate::net::minecraft::client::renderer::biome_colors;
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
    use crate::net::minecraft::client::renderer::chunk::render_region_cache::RenderRegionCache;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::section_pos::SectionPos;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_chunks_biomes_packet::ClientboundChunksBiomesPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::biome::biome::tests::biome_registries;
    use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;
    use crate::net::minecraft::world::level::color_resolver::ColorResolver;
    use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
    use crate::net::minecraft::world::level::light_layer::LightLayer;

    /// A region of the level reads what the level reads: the walk's chunks, lit from their packets
    /// and with a layout of biomes laid on them (`ClientLevel`'s golden), copied around sections
    /// in the middle, at the edge and at the top and bottom of the world. Every block's light and
    /// raw brightness in the 27 sections, and every resolver's tint in the section and the layer
    /// below, read from the copy, then again over a copy of the level's cache with the first
    /// region's tints given back to it.
    #[test]
    fn reads_what_the_level_reads() {
        load_colormaps();
        let chunk_golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = biome_registries(&chunk_golden.input("BIOMES"));
        let level_golden = Golden::load("net.minecraft.client.multiplayer.ClientLevel");
        let (args, _) = &level_golden.cases("layouts")[1];
        let mut level = ClientLevel::new(
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
            i64::decode(&level_golden.input("ZOOM_SEED")),
            -63,
        );
        level.set_biome_blend_radius(args[0].as_i64().expect("a radius") as i32);
        let mut chunks = Vec::new();
        for (args, _) in chunk_golden.cases("captured") {
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            ClientPacketListener::handle_level_chunk_with_light_in(&mut level, &packet);
            chunks.push((packet.x(), packet.z()));
        }
        while level.has_light_updates() {
            level.update();
        }
        level.update();
        let mut buffer = unpooled::wrapped_buffer(unhex(args[1].as_str().expect("hex")));
        let biomes = ClientboundChunksBiomesPacket::STREAM_CODEC.decode(&mut buffer);
        ClientPacketListener::handle_chunks_biomes_in(&mut level, &biomes);

        let resolvers: [&'static ColorResolver; 4] = [
            &biome_colors::GRASS_COLOR_RESOLVER,
            &biome_colors::FOLIAGE_COLOR_RESOLVER,
            &biome_colors::DRY_FOLIAGE_COLOR_RESOLVER,
            &biome_colors::WATER_COLOR_RESOLVER,
        ];
        let (min_x, max_x) = chunks
            .iter()
            .fold((i32::MAX, i32::MIN), |(min, max), &(x, _)| {
                (min.min(x), max.max(x))
            });
        let middle_z = chunks[chunks.len() / 2].1;
        let (bottom, top) = (level.get_min_section_y(), level.get_max_section_y());
        for (x, y, z) in [
            (min_x + 1, bottom, middle_z),
            (min_x + 1, bottom + 1, middle_z),
            ((min_x + max_x) / 2, bottom + 2, middle_z),
            ((min_x + max_x) / 2, top, middle_z),
            (max_x, bottom + 1, middle_z),
        ] {
            for pass in 0..2 {
                let mut cache = RenderRegionCache::new();
                let region = cache.create_region(&level, SectionPos::as_long_xyz(x, y, z));
                for dy in -16..32 {
                    for dz in -16..32 {
                        for dx in -16..32 {
                            let pos = BlockPos::new(16 * x + dx, 16 * y + dy, 16 * z + dz);
                            for layer in [LightLayer::BLOCK, LightLayer::SKY] {
                                assert_eq!(
                                    region.get_brightness(layer, &pos),
                                    level.get_brightness(layer, &pos),
                                    "{layer:?} light at {pos:?}"
                                );
                            }
                            assert_eq!(
                                region.get_raw_brightness(&pos, 5),
                                level.get_raw_brightness(&pos, 5),
                                "raw brightness at {pos:?}"
                            );
                        }
                    }
                }
                for dy in -1..16 {
                    for dz in 0..16 {
                        for dx in 0..16 {
                            let pos = BlockPos::new(16 * x + dx, 16 * y + dy, 16 * z + dz);
                            for resolver in resolvers {
                                assert_eq!(
                                    region.get_block_tint(&pos, resolver),
                                    level.calculate_block_tint(&pos, resolver),
                                    "{} at {pos:?} (pass {pass})",
                                    resolver.name()
                                );
                            }
                        }
                    }
                }
                level.give_back_tints(region.into_tints());
            }
        }
    }
}
