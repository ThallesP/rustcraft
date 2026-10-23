//! net/minecraft/client/renderer/block/BlockAndTintGetter.java
//!
//! A resolver is one of `BiomeColors`' statics, which a level tells apart by reference
//! (`ColorResolver::same`). `EMPTY` is `empty()`'s value: Java's constant holds
//! `LevelLightEngine.EMPTY`, which is the game thread's here, so each caller makes one.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;

/// What the renderer reads to mesh blocks: blocks, light, and tints.
pub trait BlockAndTintGetter: BlockAndLightGetter {
    /// The dimension's lighting by face.
    fn cardinal_lighting(&self) -> CardinalLighting;

    /// The resolver's colour at the block, blended over the biomes around it.
    fn get_block_tint(&self, pos: &BlockPos, color: &'static ColorResolver) -> i32;
}

/// `EMPTY`'s class: air everywhere, no light, no tint (-1), the default lighting.
pub struct EmptyBlockAndTintGetter {
    light_engine: LevelLightEngine,
}

/// `EMPTY`.
pub fn empty() -> EmptyBlockAndTintGetter {
    EmptyBlockAndTintGetter {
        light_engine: LevelLightEngine::empty(),
    }
}

impl BlockGetter for EmptyBlockAndTintGetter {
    fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
        Blocks::AIR.default_block_state()
    }

    fn get_fluid_state(&self, _pos: &BlockPos) -> FluidState {
        FluidState::EMPTY
    }
}

impl BlockAndLightGetter for EmptyBlockAndTintGetter {
    fn get_light_engine(&self) -> &LevelLightEngine {
        &self.light_engine
    }
}

impl BlockAndTintGetter for EmptyBlockAndTintGetter {
    fn cardinal_lighting(&self) -> CardinalLighting {
        CardinalLighting::DEFAULT
    }

    fn get_block_tint(&self, _pos: &BlockPos, _color: &'static ColorResolver) -> i32 {
        -1
    }
}
