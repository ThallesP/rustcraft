//! net/minecraft/world/level/levelgen/PositionalRandomFactory.java
//!
//! Not ported yet: the `at(BlockPos)` and `fromHashOf(Identifier)` defaults, until BlockPos and
//! Identifier are.

use crate::net::minecraft::util::random_source::RandomSource;

pub trait PositionalRandomFactory {
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource>;

    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource>;

    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource>;

    fn parity_config_string(&self, sb: &mut String);
}
