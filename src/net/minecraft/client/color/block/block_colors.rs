//! net/minecraft/client/color/block/BlockColors.java
//!
//! Not ported yet: `createDefault`, whose tint sources (`BlockTintSources`) read the biomes'
//! colours through `BiomeColors` and `BlockAndTintGetter.getBlockTint`, which the world track is
//! porting; and `getColoringProperties`. Until then a level's blocks draw untinted, like vanilla
//! with an empty `BlockColors`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::color::block::block_tint_source::BlockTintSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// Each block's tint sources, by tint index.
#[derive(Default)]
pub struct BlockColors {
    sources: HashMap<Block, Vec<Arc<dyn BlockTintSource>>>,
}

impl BlockColors {
    pub const LILY_PAD_IN_WORLD: i32 = -14647248;
    pub const LILY_PAD_DEFAULT: i32 = -9321636;

    pub fn new() -> BlockColors {
        BlockColors::default()
    }

    pub fn get_tint_sources(&self, state: BlockState) -> &[Arc<dyn BlockTintSource>] {
        self.sources
            .get(&state.get_block())
            .map_or(&[], Vec::as_slice)
    }

    pub fn get_tint_source(
        &self,
        state: BlockState,
        layer: usize,
    ) -> Option<&Arc<dyn BlockTintSource>> {
        self.get_tint_sources(state).get(layer)
    }

    pub fn register(&mut self, layers: Vec<Arc<dyn BlockTintSource>>, blocks: &[Block]) {
        for &block in blocks {
            self.sources.insert(block, layers.clone());
        }
    }
}
