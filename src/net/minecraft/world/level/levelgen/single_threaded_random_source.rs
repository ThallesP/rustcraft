//! net/minecraft/world/level/levelgen/SingleThreadedRandomSource.java

use super::bit_random_source::{self, BitRandomSource};
use super::legacy_random_source::LegacyPositionalRandomFactory;
use super::marsaglia_polar_gaussian::MarsagliaPolarGaussian;
use super::positional_random_factory::PositionalRandomFactory;
use crate::net::minecraft::util::random_source::RandomSource;

const MODULUS_BITS: i32 = 48;
const MODULUS_MASK: i64 = 281474976710655;
const MULTIPLIER: i64 = 25214903917;
const INCREMENT: i64 = 11;

#[derive(Clone, Debug)]
pub struct SingleThreadedRandomSource {
    seed: i64,
    gaussian_source: Option<MarsagliaPolarGaussian>,
}

impl SingleThreadedRandomSource {
    pub fn new(seed: i64) -> SingleThreadedRandomSource {
        let mut source = SingleThreadedRandomSource {
            seed: 0,
            gaussian_source: None,
        };
        source.set_seed(seed);
        source
    }
}

impl BitRandomSource for SingleThreadedRandomSource {
    fn next(&mut self, bits: i32) -> i32 {
        let new_seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MODULUS_MASK;
        self.seed = new_seed;
        (new_seed >> (MODULUS_BITS - bits)) as i32
    }
}

impl RandomSource for SingleThreadedRandomSource {
    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(SingleThreadedRandomSource::new(self.next_long()))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(LegacyPositionalRandomFactory::new(self.next_long()))
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MODULUS_MASK;
        if let Some(gaussian_source) = &mut self.gaussian_source {
            gaussian_source.reset();
        }
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
        let mut gaussian_source = self.gaussian_source.unwrap_or_default();
        let gaussian = gaussian_source.next_gaussian(self);
        self.gaussian_source = Some(gaussian_source);
        gaussian
    }
}
