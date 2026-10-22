//! net/minecraft/world/level/levelgen/XoroshiroRandomSource.java
//!
//! Not ported yet: CODEC, until DataFixerUpper's Codec is.

use super::marsaglia_polar_gaussian::MarsagliaPolarGaussian;
use super::positional_random_factory::PositionalRandomFactory;
use super::random_support::{RandomSupport, Seed128bit};
use super::xoroshiro128_plus_plus::Xoroshiro128PlusPlus;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;

const FLOAT_UNIT: f32 = 5.9604645E-8;
/// 2^-53, multiplied as doubles; see bit_random_source::DOUBLE_MULTIPLIER.
const DOUBLE_UNIT: f64 = 1.110223E-16_f32 as f64;

#[derive(Clone, Debug)]
pub struct XoroshiroRandomSource {
    random_number_generator: Xoroshiro128PlusPlus,
    gaussian_source: MarsagliaPolarGaussian,
}

impl XoroshiroRandomSource {
    pub fn new(seed: i64) -> XoroshiroRandomSource {
        XoroshiroRandomSource::from_generator(Xoroshiro128PlusPlus::from_seed128bit(
            RandomSupport::upgrade_seed_to128bit(seed),
        ))
    }

    pub fn from_seed128bit(seed: Seed128bit) -> XoroshiroRandomSource {
        XoroshiroRandomSource::from_generator(Xoroshiro128PlusPlus::from_seed128bit(seed))
    }

    pub fn from_seed_lo_hi(seed_lo: i64, seed_hi: i64) -> XoroshiroRandomSource {
        XoroshiroRandomSource::from_generator(Xoroshiro128PlusPlus::new(seed_lo, seed_hi))
    }

    fn from_generator(random_number_generator: Xoroshiro128PlusPlus) -> XoroshiroRandomSource {
        XoroshiroRandomSource {
            random_number_generator,
            gaussian_source: MarsagliaPolarGaussian::new(),
        }
    }

    fn next_bits(&mut self, bits: i32) -> i64 {
        ((self.random_number_generator.next_long() as u64) >> (64 - bits)) as i64
    }
}

impl RandomSource for XoroshiroRandomSource {
    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(XoroshiroRandomSource::from_seed_lo_hi(
            self.random_number_generator.next_long(),
            self.random_number_generator.next_long(),
        ))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(XoroshiroPositionalRandomFactory::new(
            self.random_number_generator.next_long(),
            self.random_number_generator.next_long(),
        ))
    }

    fn set_seed(&mut self, seed: i64) {
        self.random_number_generator =
            Xoroshiro128PlusPlus::from_seed128bit(RandomSupport::upgrade_seed_to128bit(seed));
        self.gaussian_source.reset();
    }

    fn next_int(&mut self) -> i32 {
        self.random_number_generator.next_long() as i32
    }

    // Lemire's method: multiply into 64 bits and reject the few samples that would bias the result.
    fn next_int_bounded(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");

        let mut random_bits = i64::from(self.next_int() as u32);
        let mut multiplied_random_bits = random_bits * i64::from(bound);
        let mut fractional_part = multiplied_random_bits & 4294967295;
        if fractional_part < i64::from(bound) {
            let unbiased_buckets_start_index =
                i64::from(bound.wrapping_neg() as u32 % bound as u32);
            while fractional_part < unbiased_buckets_start_index {
                random_bits = i64::from(self.next_int() as u32);
                multiplied_random_bits = random_bits * i64::from(bound);
                fractional_part = multiplied_random_bits & 4294967295;
            }
        }

        let integer_part = multiplied_random_bits >> 32;
        integer_part as i32
    }

    fn next_long(&mut self) -> i64 {
        self.random_number_generator.next_long()
    }

    fn next_boolean(&mut self) -> bool {
        (self.random_number_generator.next_long() & 1) != 0
    }

    fn next_float(&mut self) -> f32 {
        self.next_bits(24) as f32 * FLOAT_UNIT
    }

    fn next_double(&mut self) -> f64 {
        self.next_bits(53) as f64 * DOUBLE_UNIT
    }

    fn next_gaussian(&mut self) -> f64 {
        let mut gaussian_source = self.gaussian_source;
        let gaussian = gaussian_source.next_gaussian(self);
        self.gaussian_source = gaussian_source;
        gaussian
    }

    fn consume_count(&mut self, rounds: i32) {
        for _ in 0..rounds {
            self.random_number_generator.next_long();
        }
    }
}

#[derive(Clone, Debug)]
pub struct XoroshiroPositionalRandomFactory {
    seed_lo: i64,
    seed_hi: i64,
}

impl XoroshiroPositionalRandomFactory {
    pub fn new(seed_lo: i64, seed_hi: i64) -> XoroshiroPositionalRandomFactory {
        XoroshiroPositionalRandomFactory { seed_lo, seed_hi }
    }
}

impl PositionalRandomFactory for XoroshiroPositionalRandomFactory {
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        let seed = RandomSupport::seed_from_hash_of(name);
        Box::new(XoroshiroRandomSource::from_seed128bit(
            seed.xor(self.seed_lo, self.seed_hi),
        ))
    }

    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(XoroshiroRandomSource::from_seed_lo_hi(
            seed ^ self.seed_lo,
            seed ^ self.seed_hi,
        ))
    }

    #[allow(deprecated)]
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        let positional_seed = Mth::get_seed(x, y, z);
        let random_seed = positional_seed ^ self.seed_lo;
        Box::new(XoroshiroRandomSource::from_seed_lo_hi(
            random_seed,
            self.seed_hi,
        ))
    }

    fn parity_config_string(&self, sb: &mut String) {
        sb.push_str(&format!(
            "seedLo: {}, seedHi: {}",
            self.seed_lo, self.seed_hi
        ));
    }
}
