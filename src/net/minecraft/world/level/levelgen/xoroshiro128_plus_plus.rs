//! net/minecraft/world/level/levelgen/Xoroshiro128PlusPlus.java
//!
//! Not ported yet: CODEC, until DataFixerUpper's Codec is.

use super::random_support::{RandomSupport, Seed128bit};

#[derive(Clone, Debug)]
pub struct Xoroshiro128PlusPlus {
    seed_lo: i64,
    seed_hi: i64,
}

impl Xoroshiro128PlusPlus {
    pub fn from_seed128bit(seed: Seed128bit) -> Xoroshiro128PlusPlus {
        Xoroshiro128PlusPlus::new(seed.seed_lo(), seed.seed_hi())
    }

    pub fn new(seed_lo: i64, seed_hi: i64) -> Xoroshiro128PlusPlus {
        if (seed_lo | seed_hi) == 0 {
            return Xoroshiro128PlusPlus {
                seed_lo: RandomSupport::GOLDEN_RATIO_64,
                seed_hi: RandomSupport::SILVER_RATIO_64,
            };
        }
        Xoroshiro128PlusPlus { seed_lo, seed_hi }
    }

    pub fn next_long(&mut self) -> i64 {
        let s0 = self.seed_lo;
        let mut s1 = self.seed_hi;
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        s1 ^= s0;
        self.seed_lo = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.seed_hi = s1.rotate_left(28);
        result
    }
}
