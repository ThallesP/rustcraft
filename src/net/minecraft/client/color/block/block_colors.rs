//! net/minecraft/client/color/block/BlockColors.java
//!
//! Not ported yet: `getColoringProperties`, which needs the sources' `relevantProperties`. Java's
//! `IdentityHashMap` of blocks is a map by block, whose order nothing reads.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::color::block::block_tint_source::BlockTintSource;
use crate::net::minecraft::client::color::block::block_tint_sources::BlockTintSources;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
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

    /// The game's: grass, leaves, water, redstone, stems and the constants, in Java's order (a
    /// block registered twice keeps the last).
    pub fn create_default() -> BlockColors {
        let blank_layer = BlockTintSources::constant(-1);
        let mut colors = BlockColors::new();
        colors.register(
            vec![BlockTintSources::double_tall_grass()],
            &[Blocks::LARGE_FERN, Blocks::TALL_GRASS],
        );
        colors.register(
            vec![BlockTintSources::grass()],
            &[
                Blocks::FERN,
                Blocks::SHORT_GRASS,
                Blocks::POTTED_FERN,
                Blocks::BUSH,
            ],
        );
        colors.register(
            vec![BlockTintSources::grass_block()],
            &[Blocks::GRASS_BLOCK],
        );
        colors.register(
            vec![blank_layer, BlockTintSources::grass()],
            &[Blocks::PINK_PETALS, Blocks::WILDFLOWERS],
        );
        colors.register(
            vec![BlockTintSources::constant(-10380959)],
            &[Blocks::SPRUCE_LEAVES],
        );
        colors.register(
            vec![BlockTintSources::constant(-8345771)],
            &[Blocks::BIRCH_LEAVES],
        );
        colors.register(
            vec![BlockTintSources::foliage()],
            &[
                Blocks::OAK_LEAVES,
                Blocks::JUNGLE_LEAVES,
                Blocks::ACACIA_LEAVES,
                Blocks::DARK_OAK_LEAVES,
                Blocks::VINE,
                Blocks::MANGROVE_LEAVES,
            ],
        );
        colors.register(
            vec![BlockTintSources::dry_foliage()],
            &[Blocks::LEAF_LITTER],
        );
        colors.register(vec![BlockTintSources::water()], &[Blocks::WATER_CAULDRON]);
        colors.register(
            vec![BlockTintSources::water_particles()],
            &[Blocks::WATER, Blocks::BUBBLE_COLUMN],
        );
        colors.register(vec![BlockTintSources::redstone()], &[Blocks::REDSTONE_WIRE]);
        colors.register(vec![BlockTintSources::sugar_cane()], &[Blocks::SUGAR_CANE]);
        colors.register(
            vec![BlockTintSources::constant(-2046180)],
            &[Blocks::ATTACHED_MELON_STEM, Blocks::ATTACHED_PUMPKIN_STEM],
        );
        colors.register(
            vec![BlockTintSources::stem()],
            &[Blocks::MELON_STEM, Blocks::PUMPKIN_STEM],
        );
        colors.register(
            vec![BlockTintSources::constant_in_world(-9321636, -14647248)],
            &[Blocks::LILY_PAD],
        );
        colors
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::BlockColors;
    use crate::golden::{Decode, Golden};
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::level::block::blocks;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;

    /// Which blocks have tint layers, and each layer's colours in hand and in an empty level, for
    /// every state of those.
    #[test]
    fn create_default_matches_vanilla() {
        load_colormaps();
        let golden = Golden::load("net.minecraft.client.color.block.BlockColors");
        let colors = BlockColors::create_default();
        let level = block_and_tint_getter::empty();
        let pos = BlockPos::new(1, 2, 3);
        golden.check("sources", |name: String| {
            let block = blocks::registry()
                .get_value(&Identifier::parse(&name))
                .expect("a vanilla block");
            colors.get_tint_sources(block.default_block_state()).len() as i32
        });
        for (args, expected) in golden.cases("colors") {
            let state = BlockState::decode(&args[0]);
            let actual: Vec<Value> = colors
                .get_tint_sources(state)
                .iter()
                .map(|source| {
                    json!([
                        source.color(state),
                        source.color_in_world(state, &level, &pos),
                        source.color_as_terrain_particle(state, &level, &pos)
                    ])
                })
                .collect();
            assert_eq!(Value::from(actual), expected, "{state}");
        }
        golden.finish();
    }
}
