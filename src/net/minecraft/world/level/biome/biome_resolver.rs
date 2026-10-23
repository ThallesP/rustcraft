//! net/minecraft/world/level/biome/BiomeResolver.java

use crate::net::minecraft::core::holder::Holder;

/// Where a biome lookup reads its biomes, by quart: a level, or a chunk.
pub trait BiomeResolver {
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder;
}
