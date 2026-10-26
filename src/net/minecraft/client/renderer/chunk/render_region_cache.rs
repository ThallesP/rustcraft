//! net/minecraft/client/renderer/chunk/RenderRegionCache.java
//!
//! The section copies a frame's compiles share, by section, so each section is copied once
//! however many regions read it: its blocks (`SectionCopy`), and its light and biomes
//! (`SectionLevelCopy`), which the port's regions copy with the blocks (`RenderSectionRegion`).
//! `create_region_over` makes a region whose light and tints a level answers itself, which a test
//! stands in for the level with.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::client_clock_manager::ClientClockManager;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::chunk::render_section_region::{
    RenderSectionRegion, SectionLevelCopy,
};
use crate::net::minecraft::client::renderer::chunk::section_copy::SectionCopy;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;

#[derive(Default)]
pub struct RenderRegionCache {
    section_copy_cache: HashMap<i64, Arc<SectionCopy>>,
    level_copy_cache: HashMap<i64, Arc<SectionLevelCopy>>,
}

impl RenderRegionCache {
    pub fn new() -> RenderRegionCache {
        RenderRegionCache::default()
    }

    /// The region around the section, with copies of the level's light, biomes and cached tints
    /// there.
    pub fn create_region(&mut self, level: &ClientLevel, section_node: i64) -> RenderSectionRegion {
        let (min_x, min_y, min_z) = RenderRegionCache::min_section(section_node);
        let sections = self.copy_sections(level, section_node);
        let level_sections = RenderRegionCache::region_nodes(section_node)
            .map(|(x, y, z)| {
                Arc::clone(
                    self.level_copy_cache
                        .entry(SectionPos::as_long_xyz(x, y, z))
                        .or_insert_with(|| Arc::new(SectionLevelCopy::new(level, x, y, z))),
                )
            })
            .collect();
        RenderSectionRegion::new(level, min_x, min_y, min_z, sections, level_sections)
    }

    /// The region around the section, whose light and tints `lighting` answers.
    pub fn create_region_over<'a>(
        &mut self,
        level: &ClientLevel,
        lighting: &'a dyn BlockAndTintGetter,
        section_node: i64,
    ) -> RenderSectionRegion<&'a dyn BlockAndTintGetter> {
        let (min_x, min_y, min_z) = RenderRegionCache::min_section(section_node);
        let sections = self.copy_sections(level, section_node);
        RenderSectionRegion::over(lighting, min_x, min_y, min_z, sections)
    }

    fn min_section(section_node: i64) -> (i32, i32, i32) {
        (
            SectionPos::x_of(section_node) - 1,
            SectionPos::y_of(section_node) - 1,
            SectionPos::z_of(section_node) - 1,
        )
    }

    /// The region's 27 sections in `RenderSectionRegion.index`'s order: x, then y, then z.
    fn region_nodes(section_node: i64) -> impl Iterator<Item = (i32, i32, i32)> {
        let (min_x, min_y, min_z) = RenderRegionCache::min_section(section_node);
        (min_z..=min_z + 2).flat_map(move |z| {
            (min_y..=min_y + 2).flat_map(move |y| (min_x..=min_x + 2).map(move |x| (x, y, z)))
        })
    }

    fn copy_sections(&mut self, level: &ClientLevel, section_node: i64) -> Vec<Arc<SectionCopy>> {
        RenderRegionCache::region_nodes(section_node)
            .map(|(x, y, z)| self.get_section_data_copy(level, x, y, z))
            .collect()
    }

    fn get_section_data_copy(
        &mut self,
        level: &ClientLevel,
        section_x: i32,
        section_y: i32,
        section_z: i32,
    ) -> Arc<SectionCopy> {
        Arc::clone(
            self.section_copy_cache
                .entry(SectionPos::as_long_xyz(section_x, section_y, section_z))
                .or_insert_with(|| {
                    let chunk = level.get_chunk_source().get_chunk(section_x, section_z);
                    let section_index = level.get_section_index_from_section_y(section_y);
                    Arc::new(SectionCopy::new(chunk, section_index, level.is_debug()))
                }),
        )
    }
}

/// What the environment attributes' layers read of a `ClientLevel` (`LayerLevel`), which has no
/// Java class: in Java the level answers it. The clocks are the packet listener's
/// (`ClientLevel.clockManager()`), a clock it doesn't have being at no ticks like Java's new
/// instance; the noise biomes are the chunks' (`Level.getNoiseBiome`, plains for a chunk the
/// level doesn't have, `getUncachedNoiseBiome`). The weather is the level's, as of this tick
/// (`getRainLevel(1.0F)`, `getThunderLevel(1.0F)`), and so is the lightning's flash.
pub struct LevelEnvironment<'a> {
    level: &'a ClientLevel,
    clocks: std::cell::RefCell<ClientClockManager>,
}

impl<'a> LevelEnvironment<'a> {
    pub fn new(level: &'a ClientLevel, clocks: ClientClockManager) -> LevelEnvironment<'a> {
        LevelEnvironment {
            level,
            clocks: std::cell::RefCell::new(clocks),
        }
    }
}

impl LayerLevel for LevelEnvironment<'_> {
    fn clock_total_ticks(&self, clock: &Holder) -> i64 {
        self.clocks.borrow_mut().get_instance(clock).total_ticks()
    }

    fn rain_level(&self) -> f32 {
        self.level.get_rain_level(1.0)
    }

    fn thunder_level(&self) -> f32 {
        self.level.get_thunder_level(1.0)
    }

    fn sky_flash_time(&self) -> i32 {
        self.level.get_sky_flash_time()
    }

    fn get_noise_biome_at_quart(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        let Some(chunk) = self
            .level
            .get_chunk_source()
            .get_chunk(quart_x >> 2, quart_z >> 2)
        else {
            return Holder::Reference(ResourceKey::create(
                Registries::BIOME,
                Identifier::with_default_namespace("plains"),
            ));
        };
        // ChunkAccess.getNoiseBiome: the quart's height kept in the chunk's.
        let min_quart_y = self.level.get_min_y() >> 2;
        let max_quart_y_inclusive = min_quart_y + (self.level.get_height() >> 2) - 1;
        let clamped_quart_y = Mth::clamp_i32(quart_y, min_quart_y, max_quart_y_inclusive);
        let section_index = self.level.get_section_index(clamped_quart_y << 2);
        chunk.get_sections()[section_index as usize].get_noise_biome(
            quart_x & 3,
            clamped_quart_y & 3,
            quart_z & 3,
        )
    }
}
