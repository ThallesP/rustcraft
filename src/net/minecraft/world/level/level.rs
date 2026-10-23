//! net/minecraft/world/level/Level.java, as far as entities read it.
//!
//! Java's `Level` is the world an entity lives in; the client's is `ClientLevel`, which the world
//! track brings with chunk storage. Until then this is what an entity's movement asks of it
//! besides collisions, and tests implement it over a few blocks. Entities hold their level in
//! Java; here it's a parameter of what reads it.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;

/// A level, as an entity's tick reads it.
pub trait Level: CollisionGetter {
    fn is_client_side(&self) -> bool;

    /// `getMinY()`, the lowest block's height.
    fn get_min_y(&self) -> i32;

    /// `hasChunkAt(BlockPos)`: whether the block's chunk is loaded.
    fn has_chunk_at(&self, pos: &BlockPos) -> bool;

    /// Whether any section from `section_y0` to `section_y1` of the chunk holds a fluid, `None`
    /// when the chunk isn't loaded: `getChunk(x, z, ChunkStatus.FULL, false)` and its sections'
    /// `hasFluid()`, which `EntityFluidInteraction` asks before it reads fluid states.
    fn chunk_has_fluid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        section_y0: i32,
        section_y1: i32,
    ) -> Option<bool>;
}

/// The level as the collisions see it.
pub fn collisions(level: &dyn Level) -> &dyn CollisionGetter {
    level
}
