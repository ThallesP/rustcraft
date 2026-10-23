//! net/minecraft/client/renderer/chunk/RenderRegionCache.java
//!
//! The section copies a frame's compiles share, by section, so each section is copied once
//! however many regions read it. `lighting` is what the regions ask the level for besides blocks
//! (`RenderSectionRegion`), until `ClientLevel` answers it itself.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::chunk::render_section_region::RenderSectionRegion;
use crate::net::minecraft::client::renderer::chunk::section_copy::SectionCopy;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;

#[derive(Default)]
pub struct RenderRegionCache {
    section_copy_cache: HashMap<i64, Arc<SectionCopy>>,
}

impl RenderRegionCache {
    pub fn new() -> RenderRegionCache {
        RenderRegionCache::default()
    }

    pub fn create_region<'a>(
        &mut self,
        level: &ClientLevel,
        lighting: &'a dyn BlockAndTintGetter,
        section_node: i64,
    ) -> RenderSectionRegion<'a> {
        let section_x = SectionPos::x_of(section_node);
        let section_y = SectionPos::y_of(section_node);
        let section_z = SectionPos::z_of(section_node);
        let min_section_x = section_x - 1;
        let min_section_y = section_y - 1;
        let min_section_z = section_z - 1;
        let mut region_sections: Vec<Option<Arc<SectionCopy>>> = vec![None; 27];

        for region_section_z in min_section_z..=section_z + 1 {
            for region_section_y in min_section_y..=section_y + 1 {
                for region_section_x in min_section_x..=section_x + 1 {
                    let index = RenderSectionRegion::index(
                        min_section_x,
                        min_section_y,
                        min_section_z,
                        region_section_x,
                        region_section_y,
                        region_section_z,
                    );
                    region_sections[index] = Some(self.get_section_data_copy(
                        level,
                        region_section_x,
                        region_section_y,
                        region_section_z,
                    ));
                }
            }
        }

        RenderSectionRegion::new(
            lighting,
            min_section_x,
            min_section_y,
            min_section_z,
            region_sections
                .into_iter()
                .map(|section| section.expect("every section of the region is copied"))
                .collect(),
        )
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
