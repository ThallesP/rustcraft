//! net/minecraft/world/level/chunk/LightChunkGetter.java
//!
//! Java's `getLevel()` hands the engines the level, of which they only ask its height (an empty
//! chunk's sky light sources), which `LevelLightEngine::new` takes instead.

use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::light_chunk::LightChunk;
use crate::net::minecraft::world::level::light_layer::LightLayer;

/// What the light engines read chunks from: the level's chunk source.
pub trait LightChunkGetter {
    fn get_chunk_for_lighting(&self, x: i32, z: i32) -> Option<&dyn LightChunk>;

    /// Told about each section whose light changed when the engines swap in what they worked
    /// out.
    fn on_light_update(&self, _layer: LightLayer, _pos: SectionPos) {}
}
