//! net/minecraft/world/level/chunk/LightChunk.java
//!
//! Not ported yet: `findBlockLightSources`, which only the server's `propagateLightSources`
//! calls, when it lights a chunk it made.

use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::lighting::chunk_sky_light_sources::ChunkSkyLightSources;

/// A chunk as the light engines read it: its blocks and its sky's lowest sources.
pub trait LightChunk: BlockGetter {
    fn get_sky_light_sources(&self) -> &ChunkSkyLightSources;
}
