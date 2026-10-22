//! net/minecraft/world/level/levelgen/ThreadSafeLegacyRandomSource.java
//!
//! Vanilla shares this across threads through a compare-and-set loop on an AtomicLong. With
//! `&mut self` methods, sharing it takes a Mutex instead, so the seed is a plain field. Unlike
//! LegacyRandomSource, `set_seed` doesn't reset the cached gaussian.

use super::bit_random_source::{self, BitRandomSource};
use super::legacy_random_source::LegacyPositionalRandomFactory;
use super::marsaglia_polar_gaussian::MarsagliaPolarGaussian;
use super::positional_random_factory::PositionalRandomFactory;
use crate::net::minecraft::util::random_source::RandomSource;

const MODULUS_BITS: i32 = 48;
const MODULUS_MASK: i64 = 281474976710655;
const MULTIPLIER: i64 = 25214903917;
const INCREMENT: i64 = 11;

#[deprecated]
#[derive(Clone, Debug)]
pub struct ThreadSafeLegacyRandomSource {
    seed: i64,
    gaussian_source: MarsagliaPolarGaussian,
}

#[allow(deprecated)]
impl ThreadSafeLegacyRandomSource {
    pub fn new(seed: i64) -> ThreadSafeLegacyRandomSource {
        let mut source = ThreadSafeLegacyRandomSource {
            seed: 0,
            gaussian_source: MarsagliaPolarGaussian::new(),
        };
        source.set_seed(seed);
        source
    }
}

#[allow(deprecated)]
impl BitRandomSource for ThreadSafeLegacyRandomSource {
    fn next(&mut self, bits: i32) -> i32 {
        let next_seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MODULUS_MASK;
        self.seed = next_seed;
        ((next_seed as u64) >> (MODULUS_BITS - bits)) as i32
    }
}

#[allow(deprecated)]
impl RandomSource for ThreadSafeLegacyRandomSource {
    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(ThreadSafeLegacyRandomSource::new(self.next_long()))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(LegacyPositionalRandomFactory::new(self.next_long()))
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MODULUS_MASK;
    }

    fn next_int(&mut self) -> i32 {
        bit_random_source::next_int(self)
    }

    fn next_int_bounded(&mut self, bound: i32) -> i32 {
        bit_random_source::next_int_bounded(self, bound)
    }

    fn next_long(&mut self) -> i64 {
        bit_random_source::next_long(self)
    }

    fn next_boolean(&mut self) -> bool {
        bit_random_source::next_boolean(self)
    }

    fn next_float(&mut self) -> f32 {
        bit_random_source::next_float(self)
    }

    fn next_double(&mut self) -> f64 {
        bit_random_source::next_double(self)
    }

    fn next_gaussian(&mut self) -> f64 {
        let mut gaussian_source = self.gaussian_source;
        let gaussian = gaussian_source.next_gaussian(self);
        self.gaussian_source = gaussian_source;
        gaussian
    }
}
