//! it/unimi/dsi/fastutil/HashCommon.java
//!
//! From fastutil's bytecode (it publishes no sources with the game): what its open hash sets
//! size and place their keys with. `MixHasher` hashes a `long` key with `mix`, for std's maps
//! standing in for fastutil's `Long2ObjectOpenHashMap`s that are only looked up in, never iterated
//! (the light engine's layers): std's default hasher, SipHash, costs more than the lookup.

/*
 * Copyright (C) 2002-2024 Sebastiano Vigna
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust and changed for rustcraft.
 */

use std::hash::{BuildHasherDefault, Hasher};

/// fastutil's hashing helpers.
pub struct HashCommon;

/// A hasher of one `long`, its `mix`.
#[derive(Default)]
pub struct MixHasher(u64);

/// What std's maps take to hash their keys with `MixHasher`.
pub type BuildMixHasher = BuildHasherDefault<MixHasher>;

impl Hasher for MixHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_i64((self.0 << 8 | u64::from(byte)) as i64);
        }
    }

    fn write_i64(&mut self, key: i64) {
        self.0 = HashCommon::mix_i64(key) as u64;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

impl HashCommon {
    /// `mix(int)`: a hash code's bits spread by the golden ratio, which picks its slot.
    pub fn mix_i32(x: i32) -> i32 {
        let h = x.wrapping_mul(-1640531527);
        h ^ ((h as u32) >> 16) as i32
    }

    /// `mix(long)`: the key's bits spread by the golden ratio, which picks its slot.
    pub fn mix_i64(x: i64) -> i64 {
        let mut h = x.wrapping_mul(-7046029254386353131);
        h ^= ((h as u64) >> 32) as i64;
        h ^ ((h as u64) >> 16) as i64
    }

    /// `nextPowerOfTwo(int)`: the smallest power of two at least `x` (1 for 0), Java's shift
    /// masked to 5 bits.
    pub fn next_power_of_two_i32(x: i32) -> i32 {
        1i32.wrapping_shl(32 - x.wrapping_sub(1).leading_zeros())
    }

    /// `nextPowerOfTwo(long)`.
    pub fn next_power_of_two_i64(x: i64) -> i64 {
        1i64.wrapping_shl(64 - x.wrapping_sub(1).leading_zeros())
    }

    /// `maxFill(int, float)`: how many keys a table of `n` slots takes before it grows.
    pub fn max_fill(n: i32, f: f32) -> i32 {
        (((n as f64) * (f as f64)).ceil() as i32).min(n - 1)
    }

    /// `arraySize(int, float)`: the table for `expected` keys at load factor `f`, at least 2.
    /// Panics past 2^30 slots, where Java throws `IllegalArgumentException`.
    pub fn array_size(expected: i32, f: f32) -> i32 {
        let s = 2i64.max(HashCommon::next_power_of_two_i64(
            ((expected as f64) / (f as f64)).ceil() as i64,
        ));
        if s > 1 << 30 {
            panic!("Too large ({expected} expected elements with load factor {f})");
        }
        s as i32
    }
}
