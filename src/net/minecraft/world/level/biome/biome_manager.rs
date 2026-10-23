//! net/minecraft/world/level/biome/BiomeManager.java
//!
//! A block's biome: the nearest of the eight quarts around it, each moved by a fiddle the seed
//! hash decides, so biome edges aren't straight. Java's holds its noise biome source, the level (a
//! back-reference), which is a parameter here (`get_biome`'s `noise_biome_source`). Not ported yet:
//! `obfuscateSeed` (Guava's SHA-256), which only the server hashes the seed with.

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::util::linear_congruential_generator::LinearCongruentialGenerator;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::biome::biome_resolver::BiomeResolver;

/// The zoomed biome lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiomeManager {
    biome_zoom_seed: i64,
}

impl BiomeManager {
    pub const CHUNK_CENTER_QUART: i32 = 2;
    const ZOOM_BITS: i32 = 2;
    const ZOOM: i32 = 4;
    const ZOOM_MASK: i32 = 3;

    /// `BiomeManager(noiseBiomeSource, seed)`, with the seed hash the login sent.
    pub fn new(seed: i64) -> BiomeManager {
        BiomeManager {
            biome_zoom_seed: seed,
        }
    }

    pub fn get_biome(&self, noise_biome_source: &dyn BiomeResolver, pos: &BlockPos) -> Holder {
        self.get_biome_xyz(noise_biome_source, pos.get_x(), pos.get_y(), pos.get_z())
    }

    pub fn get_biome_xyz(
        &self,
        noise_biome_source: &dyn BiomeResolver,
        x: i32,
        y: i32,
        z: i32,
    ) -> Holder {
        let abs_x = x.wrapping_sub(2);
        let abs_y = y.wrapping_sub(2);
        let abs_z = z.wrapping_sub(2);
        let parent_x = abs_x >> 2;
        let parent_y = abs_y >> 2;
        let parent_z = abs_z >> 2;
        let fract_x = f64::from(abs_x & 3) / 4.0;
        let fract_y = f64::from(abs_y & 3) / 4.0;
        let fract_z = f64::from(abs_z & 3) / 4.0;
        let mut min_i = 0;
        let mut min_fiddled_distance = f64::INFINITY;

        for i in 0..8 {
            let x_even = (i & 4) == 0;
            let y_even = (i & 2) == 0;
            let z_even = (i & 1) == 0;
            let corner_x = if x_even {
                parent_x
            } else {
                parent_x.wrapping_add(1)
            };
            let corner_y = if y_even {
                parent_y
            } else {
                parent_y.wrapping_add(1)
            };
            let corner_z = if z_even {
                parent_z
            } else {
                parent_z.wrapping_add(1)
            };
            let distance_x = if x_even { fract_x } else { fract_x - 1.0 };
            let distance_y = if y_even { fract_y } else { fract_y - 1.0 };
            let distance_z = if z_even { fract_z } else { fract_z - 1.0 };
            let next = BiomeManager::get_fiddled_distance(
                self.biome_zoom_seed,
                corner_x,
                corner_y,
                corner_z,
                distance_x,
                distance_y,
                distance_z,
            );
            if min_fiddled_distance > next {
                min_i = i;
                min_fiddled_distance = next;
            }
        }

        let biome_x = if (min_i & 4) == 0 {
            parent_x
        } else {
            parent_x.wrapping_add(1)
        };
        let biome_y = if (min_i & 2) == 0 {
            parent_y
        } else {
            parent_y.wrapping_add(1)
        };
        let biome_z = if (min_i & 1) == 0 {
            parent_z
        } else {
            parent_z.wrapping_add(1)
        };
        noise_biome_source.get_noise_biome(biome_x, biome_y, biome_z)
    }

    pub fn get_noise_biome_at_position(
        &self,
        noise_biome_source: &dyn BiomeResolver,
        x: f64,
        y: f64,
        z: f64,
    ) -> Holder {
        let quart_x = QuartPos::from_block(Mth::floor_f64(x));
        let quart_y = QuartPos::from_block(Mth::floor_f64(y));
        let quart_z = QuartPos::from_block(Mth::floor_f64(z));
        noise_biome_source.get_noise_biome(quart_x, quart_y, quart_z)
    }

    #[allow(clippy::too_many_arguments)]
    fn get_fiddled_distance(
        seed: i64,
        x_random: i32,
        y_random: i32,
        z_random: i32,
        distance_x: f64,
        distance_y: f64,
        distance_z: f64,
    ) -> f64 {
        let mut rval = seed;
        rval = LinearCongruentialGenerator::next(rval, i64::from(x_random));
        rval = LinearCongruentialGenerator::next(rval, i64::from(y_random));
        rval = LinearCongruentialGenerator::next(rval, i64::from(z_random));
        rval = LinearCongruentialGenerator::next(rval, i64::from(x_random));
        rval = LinearCongruentialGenerator::next(rval, i64::from(y_random));
        rval = LinearCongruentialGenerator::next(rval, i64::from(z_random));
        let fiddle_x = BiomeManager::get_fiddle(rval);
        rval = LinearCongruentialGenerator::next(rval, seed);
        let fiddle_y = BiomeManager::get_fiddle(rval);
        rval = LinearCongruentialGenerator::next(rval, seed);
        let fiddle_z = BiomeManager::get_fiddle(rval);
        Mth::square_f64(distance_z + fiddle_z)
            + Mth::square_f64(distance_y + fiddle_y)
            + Mth::square_f64(distance_x + fiddle_x)
    }

    fn get_fiddle(rval: i64) -> f64 {
        let uniform = Math::floor_mod_i64(rval >> 24, 1024) as f64 / 1024.0;
        (uniform - 0.5) * 0.9
    }
}

// Java's constants, which document the zoom.
const _: [i32; 3] = [
    BiomeManager::ZOOM_BITS,
    BiomeManager::ZOOM,
    BiomeManager::ZOOM_MASK,
];

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::BiomeManager;
    use crate::golden::Golden;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::biome::biome_resolver::BiomeResolver;

    /// A source that says which quart it was asked for.
    #[derive(Default)]
    struct Asked(Cell<(i32, i32, i32)>);

    impl BiomeResolver for Asked {
        fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
            self.0.set((quart_x, quart_y, quart_z));
            Holder::Reference(ResourceKey::create(
                Registries::BIOME,
                Identifier::with_default_namespace("plains"),
            ))
        }
    }

    /// The quart the zoomed lookup picks, for any seed and block.
    #[test]
    fn get_biome_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.biome.BiomeManager");
        golden.check(
            "getBiome(III)Lnet/minecraft/core/Holder;",
            |seed: i64, x: i32, y: i32, z: i32| {
                let asked = Asked::default();
                BiomeManager::new(seed).get_biome_xyz(&asked, x, y, z);
                asked.0.get()
            },
        );
        golden.finish();
    }
}
