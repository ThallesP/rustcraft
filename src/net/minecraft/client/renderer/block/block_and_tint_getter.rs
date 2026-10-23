//! net/minecraft/client/renderer/block/BlockAndTintGetter.java
//!
//! Not ported yet: `getBlockTint`, which needs the biomes' colours (`ColorResolver`), and
//! `EMPTY`. The trait is here so the renderer's chunk meshing can take a level, or a copy of its
//! sections, through it.

use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;

/// What the renderer reads to mesh blocks: blocks, light, and tints.
pub trait BlockAndTintGetter: BlockAndLightGetter {
    /// The dimension's lighting by face.
    fn cardinal_lighting(&self) -> CardinalLighting;
}
