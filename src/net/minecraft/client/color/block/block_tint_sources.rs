//! net/minecraft/client/color/block/BlockTintSources.java
//!
//! Java's factories make anonymous classes; here each kind is a struct implementing
//! `BlockTintSource`. Not ported yet: `relevantProperties` (`ModelGroupCollector`'s).

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::client::color::block::block_tint_source::BlockTintSource;
use crate::net::minecraft::client::renderer::biome_colors::BiomeColors;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::double_plant_block::DoublePlantBlock;
use crate::net::minecraft::world::level::block::redstone_wire_block::RedstoneWireBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::stem_block::StemBlock;
use crate::net::minecraft::world::level::dry_foliage_color::DryFoliageColor;
use crate::net::minecraft::world::level::foliage_color::FoliageColor;
use crate::net::minecraft::world::level::grass_color::GrassColor;

/// The tint sources `BlockColors.createDefault` registers.
pub struct BlockTintSources;

impl BlockTintSources {
    pub fn constant(color: i32) -> Arc<dyn BlockTintSource> {
        Arc::new(Constant {
            color_in_hand: color,
            color_in_world: None,
        })
    }

    /// `constant(colorInHand, colorInWorld)`.
    pub fn constant_in_world(color_in_hand: i32, color_in_world: i32) -> Arc<dyn BlockTintSource> {
        Arc::new(Constant {
            color_in_hand,
            color_in_world: Some(color_in_world),
        })
    }

    pub fn double_tall_grass() -> Arc<dyn BlockTintSource> {
        Arc::new(DoubleTallGrass)
    }

    pub fn grass() -> Arc<dyn BlockTintSource> {
        Arc::new(Grass { block: false })
    }

    pub fn grass_block() -> Arc<dyn BlockTintSource> {
        Arc::new(Grass { block: true })
    }

    pub fn sugar_cane() -> Arc<dyn BlockTintSource> {
        Arc::new(SugarCane)
    }

    pub fn foliage() -> Arc<dyn BlockTintSource> {
        Arc::new(Foliage { dry: false })
    }

    pub fn dry_foliage() -> Arc<dyn BlockTintSource> {
        Arc::new(Foliage { dry: true })
    }

    pub fn water() -> Arc<dyn BlockTintSource> {
        Arc::new(Water { particles: false })
    }

    pub fn water_particles() -> Arc<dyn BlockTintSource> {
        Arc::new(Water { particles: true })
    }

    pub fn redstone() -> Arc<dyn BlockTintSource> {
        Arc::new(Redstone)
    }

    pub fn stem() -> Arc<dyn BlockTintSource> {
        Arc::new(Stem)
    }
}

struct Constant {
    color_in_hand: i32,
    color_in_world: Option<i32>,
}

impl BlockTintSource for Constant {
    fn color(&self, _state: BlockState) -> i32 {
        self.color_in_hand
    }

    fn color_in_world(
        &self,
        state: BlockState,
        _level: &dyn BlockAndTintGetter,
        _pos: &BlockPos,
    ) -> i32 {
        self.color_in_world.unwrap_or_else(|| self.color(state))
    }
}

struct DoubleTallGrass;

impl BlockTintSource for DoubleTallGrass {
    fn color(&self, _state: BlockState) -> i32 {
        GrassColor::get_default_color()
    }

    /// The lower half's, for both halves.
    fn color_in_world(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        let pos = if state.get_value(DoublePlantBlock::HALF) == DoubleBlockHalf::UPPER {
            pos.below()
        } else {
            *pos
        };
        BiomeColors::get_average_grass_color(level, &pos)
    }
}

/// `grass()`, and `grassBlock()`, whose terrain particles aren't tinted.
struct Grass {
    block: bool,
}

impl BlockTintSource for Grass {
    fn color(&self, _state: BlockState) -> i32 {
        GrassColor::get_default_color()
    }

    fn color_in_world(
        &self,
        _state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        BiomeColors::get_average_grass_color(level, pos)
    }

    fn color_as_terrain_particle(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        if self.block {
            -1
        } else {
            self.color_in_world(state, level, pos)
        }
    }
}

struct SugarCane;

impl BlockTintSource for SugarCane {
    fn color(&self, _state: BlockState) -> i32 {
        -1
    }

    fn color_in_world(
        &self,
        _state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        BiomeColors::get_average_grass_color(level, pos)
    }
}

/// `foliage()`, and `dryFoliage()`.
struct Foliage {
    dry: bool,
}

impl BlockTintSource for Foliage {
    fn color(&self, _state: BlockState) -> i32 {
        if self.dry {
            DryFoliageColor::FOLIAGE_DRY_DEFAULT
        } else {
            FoliageColor::FOLIAGE_DEFAULT
        }
    }

    fn color_in_world(
        &self,
        _state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        if self.dry {
            BiomeColors::get_average_dry_foliage_color(level, pos)
        } else {
            BiomeColors::get_average_foliage_color(level, pos)
        }
    }
}

/// `water()`, and `waterParticles()`, which tints only terrain particles.
struct Water {
    particles: bool,
}

impl BlockTintSource for Water {
    fn color(&self, _state: BlockState) -> i32 {
        -1
    }

    fn color_in_world(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        if self.particles {
            self.color(state)
        } else {
            BiomeColors::get_average_water_color(level, pos)
        }
    }

    fn color_as_terrain_particle(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> i32 {
        if self.particles {
            BiomeColors::get_average_water_color(level, pos)
        } else {
            self.color_in_world(state, level, pos)
        }
    }
}

/// `RedstoneWireBlock.COLORS`: each power's colour.
static REDSTONE_COLORS: LazyLock<[i32; 16]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let power = i as f32 / 15.0;
        let red = power * 0.6 + if power > 0.0 { 0.4 } else { 0.3 };
        let green = Mth::clamp_f32(power * power * 0.7 - 0.5, 0.0, 1.0);
        let blue = Mth::clamp_f32(power * power * 0.6 - 0.7, 0.0, 1.0);
        ARGB::color(
            ARGB::as8_bit_channel(1.0),
            ARGB::as8_bit_channel(red),
            ARGB::as8_bit_channel(green),
            ARGB::as8_bit_channel(blue),
        )
    })
});

struct Redstone;

impl BlockTintSource for Redstone {
    /// `RedstoneWireBlock.getColorForPower`.
    fn color(&self, state: BlockState) -> i32 {
        REDSTONE_COLORS[state.get_value(RedstoneWireBlock::POWER) as usize]
    }
}

struct Stem;

impl BlockTintSource for Stem {
    fn color(&self, state: BlockState) -> i32 {
        let age = state.get_value(StemBlock::AGE);
        ARGB::color_rgb(age * 32, 255 - age * 8, age * 4)
    }
}
