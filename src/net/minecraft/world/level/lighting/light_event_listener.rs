//! net/minecraft/world/level/lighting/LightEventListener.java
//!
//! The engines' back-reference to the chunks they light (Java's `chunkSource` field) is a
//! parameter of the methods that read chunks (`run_light_updates`, `set_light_enabled`), which
//! take the chunk source.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

/// What a level tells its light engine.
pub trait LightEventListener {
    fn check_block(&mut self, pos: &BlockPos);

    fn has_light_work(&self) -> bool;

    fn run_light_updates(&mut self, chunk_source: &dyn LightChunkGetter) -> i32;

    /// `updateSectionStatus(BlockPos, boolean)`.
    fn update_section_status_block_pos(&mut self, pos: &BlockPos, section_empty: bool) {
        self.update_section_status(SectionPos::of_block_pos(pos), section_empty);
    }

    fn update_section_status(&mut self, pos: SectionPos, section_empty: bool);

    fn set_light_enabled(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        pos: ChunkPos,
        enable: bool,
    );
}
