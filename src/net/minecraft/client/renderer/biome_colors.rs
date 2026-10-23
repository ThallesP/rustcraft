//! net/minecraft/client/renderer/BiomeColors.java

use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::biome::biome::Biome;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;

/// `GRASS_COLOR_RESOLVER`: `Biome::getGrassColor`.
pub static GRASS_COLOR_RESOLVER: ColorResolver =
    ColorResolver::new("grass", |biome: &Biome, x, z| biome.get_grass_color(x, z));
/// `FOLIAGE_COLOR_RESOLVER`.
pub static FOLIAGE_COLOR_RESOLVER: ColorResolver =
    ColorResolver::new("foliage", |biome: &Biome, _x, _z| biome.get_foliage_color());
/// `DRY_FOLIAGE_COLOR_RESOLVER`.
pub static DRY_FOLIAGE_COLOR_RESOLVER: ColorResolver =
    ColorResolver::new("dry_foliage", |biome: &Biome, _x, _z| {
        biome.get_dry_foliage_color()
    });
/// `WATER_COLOR_RESOLVER`.
pub static WATER_COLOR_RESOLVER: ColorResolver =
    ColorResolver::new("water", |biome: &Biome, _x, _z| biome.get_water_color());

/// The biomes' colours as the level blends them.
pub struct BiomeColors;

impl BiomeColors {
    fn get_average_color(
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
        color_resolver: &'static ColorResolver,
    ) -> i32 {
        level.get_block_tint(pos, color_resolver)
    }

    pub fn get_average_grass_color(level: &dyn BlockAndTintGetter, pos: &BlockPos) -> i32 {
        BiomeColors::get_average_color(level, pos, &GRASS_COLOR_RESOLVER)
    }

    pub fn get_average_foliage_color(level: &dyn BlockAndTintGetter, pos: &BlockPos) -> i32 {
        BiomeColors::get_average_color(level, pos, &FOLIAGE_COLOR_RESOLVER)
    }

    pub fn get_average_dry_foliage_color(level: &dyn BlockAndTintGetter, pos: &BlockPos) -> i32 {
        BiomeColors::get_average_color(level, pos, &DRY_FOLIAGE_COLOR_RESOLVER)
    }

    pub fn get_average_water_color(level: &dyn BlockAndTintGetter, pos: &BlockPos) -> i32 {
        BiomeColors::get_average_color(level, pos, &WATER_COLOR_RESOLVER)
    }
}
