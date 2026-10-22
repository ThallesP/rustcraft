//! net/minecraft/world/level/levelgen/LegacyRandomSource.java
//!
//! Vanilla keeps the seed in an AtomicLong and throws if two threads race on it. `&mut self`
//! rules that race out, so the seed is a plain field.

use std::fmt::Write;

use super::bit_random_source::{self, BitRandomSource};
use super::marsaglia_polar_gaussian::MarsagliaPolarGaussian;
use super::positional_random_factory::PositionalRandomFactory;
use crate::java::lang::string::JavaString;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;

const MODULUS_BITS: i32 = 48;
const MODULUS_MASK: i64 = 281474976710655;
const MULTIPLIER: i64 = 25214903917;
const INCREMENT: i64 = 11;

#[derive(Clone, Debug)]
pub struct LegacyRandomSource {
    seed: i64,
    gaussian_source: MarsagliaPolarGaussian,
}

impl LegacyRandomSource {
    pub fn new(seed: i64) -> LegacyRandomSource {
        let mut source = LegacyRandomSource {
            seed: 0,
            gaussian_source: MarsagliaPolarGaussian::new(),
        };
        source.set_seed(seed);
        source
    }
}

impl BitRandomSource for LegacyRandomSource {
    fn next(&mut self, bits: i32) -> i32 {
        let new_seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & MODULUS_MASK;
        self.seed = new_seed;
        (new_seed >> (MODULUS_BITS - bits)) as i32
    }
}

impl RandomSource for LegacyRandomSource {
    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(self.next_long()))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(LegacyPositionalRandomFactory::new(self.next_long()))
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MODULUS_MASK;
        self.gaussian_source.reset();
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

#[derive(Clone, Debug)]
pub struct LegacyPositionalRandomFactory {
    seed: i64,
}

impl LegacyPositionalRandomFactory {
    pub fn new(seed: i64) -> LegacyPositionalRandomFactory {
        LegacyPositionalRandomFactory { seed }
    }
}

impl PositionalRandomFactory for LegacyPositionalRandomFactory {
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        let positional_seed = name.hash_code();
        Box::new(LegacyRandomSource::new(
            i64::from(positional_seed) ^ self.seed,
        ))
    }

    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(seed))
    }

    #[allow(deprecated)]
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        let positional_seed = Mth::get_seed(x, y, z);
        let random_seed = positional_seed ^ self.seed;
        Box::new(LegacyRandomSource::new(random_seed))
    }

    fn parity_config_string(&self, sb: &mut String) {
        write!(sb, "LegacyPositionalRandomFactory{{{}}}", self.seed).unwrap();
    }
}
