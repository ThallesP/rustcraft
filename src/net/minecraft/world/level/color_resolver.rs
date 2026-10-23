//! net/minecraft/world/level/ColorResolver.java
//!
//! Java's functional interface, whose instances a level tells apart by reference (its tint caches,
//! one for each of `BiomeColors`' resolvers): here a resolver is a named function in a static, and
//! two are the same resolver when they're the same static (`same`).

use crate::net::minecraft::world::level::biome::biome::Biome;

/// A biome's colour at a column.
#[derive(Debug)]
pub struct ColorResolver {
    name: &'static str,
    get_color: fn(&Biome, f64, f64) -> i32,
}

impl ColorResolver {
    pub const fn new(name: &'static str, get_color: fn(&Biome, f64, f64) -> i32) -> ColorResolver {
        ColorResolver { name, get_color }
    }

    pub fn get_color(&self, biome: &Biome, x: f64, z: f64) -> i32 {
        (self.get_color)(biome, x, z)
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Java's reference comparison.
    pub fn same(a: &ColorResolver, b: &ColorResolver) -> bool {
        std::ptr::eq(a, b)
    }
}
