//! net/minecraft/client/renderer/chunk/RenderSectionRegion.java
//!
//! The blocks a section's compile reads: its own and its 26 neighbours', copied, with the level's
//! light, tints and cardinal lighting. Java's region holds the `ClientLevel` and its light engine;
//! here it holds a `BlockAndTintGetter`, the `ClientLevel` or a test's level, whose light engine it
//! hands out and whose light it asks.
//! Block entities aren't ported (`SectionCopy`). The region's own section, which its compile reads
//! most, is decoded once into its states (`center`), the rest read from their copies' containers:
//! the same states.

use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::chunk::section_copy::SectionCopy;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

pub struct RenderSectionRegion<'a> {
    min_section_x: i32,
    min_section_y: i32,
    min_section_z: i32,
    sections: Vec<Arc<SectionCopy>>,
    center: Option<Box<[BlockState]>>,
    level: &'a dyn BlockAndTintGetter,
    cardinal_lighting: CardinalLighting,
}

impl<'a> RenderSectionRegion<'a> {
    pub const RADIUS: i32 = 1;
    pub const SIZE: i32 = 3;
    /// The region's own section's index in `sections`.
    const CENTER: usize = 13;

    pub fn new(
        level: &'a dyn BlockAndTintGetter,
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        sections: Vec<Arc<SectionCopy>>,
    ) -> RenderSectionRegion<'a> {
        RenderSectionRegion {
            min_section_x,
            min_section_y,
            min_section_z,
            center: sections[RenderSectionRegion::CENTER].decode(),
            sections,
            level,
            cardinal_lighting: level.cardinal_lighting(),
        }
    }

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

impl BlockGetter for RenderSectionRegion<'_> {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        let index = RenderSectionRegion::index(
            self.min_section_x,
            self.min_section_y,
            self.min_section_z,
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_y()),
            SectionPos::block_to_section_coord(pos.get_z()),
        );
        if index == RenderSectionRegion::CENTER
            && let Some(center) = &self.center
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

/// `getLightEngine` is the level's, which `getBrightness` asks through the level, so a test level
/// answering it itself is asked too.
impl BlockAndLightGetter for RenderSectionRegion<'_> {
    fn get_light_engine(&self) -> &LevelLightEngine {
        self.level.get_light_engine()
    }

    fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
        self.level.get_brightness(layer, pos)
    }
}

impl BlockAndTintGetter for RenderSectionRegion<'_> {
    fn cardinal_lighting(&self) -> CardinalLighting {
        self.cardinal_lighting
    }

    /// The level's.
    fn get_block_tint(&self, pos: &BlockPos, resolver: &'static ColorResolver) -> i32 {
        self.level.get_block_tint(pos, resolver)
    }
}
