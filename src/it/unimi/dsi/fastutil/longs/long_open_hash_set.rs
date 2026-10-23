//! it/unimi/dsi/fastutil/longs/LongOpenHashSet.java
//!
//! From fastutil's bytecode (it publishes no sources with the game), the parts the light engines
//! iterate: a table of keys placed by `HashCommon.mix` and probed linearly, 0 kept apart
//! (`containsNull`), grown when it's past its load factor, and iterated from the table's last slot
//! down, 0 first. So the order the game sees its keys in is fastutil's. Not ported: removing keys
//! (`remove`, `shiftKeys`, removing while iterating), which nothing iterated calls.

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

use crate::it::unimi::dsi::fastutil::hash_common::HashCommon;

/// A set of longs in an open-addressed table.
#[derive(Clone, Debug)]
pub struct LongOpenHashSet {
    key: Vec<i64>,
    mask: i32,
    contains_null: bool,
    n: i32,
    max_fill: i32,
    min_n: i32,
    size: i32,
    f: f32,
}

impl LongOpenHashSet {
    /// `LongOpenHashSet(int, float)`: a table for `expected` keys at load factor `f`. Panics for
    /// a load factor outside (0, 1) or a negative `expected`, where Java throws.
    pub fn new(expected: i32, f: f32) -> LongOpenHashSet {
        if f <= 0.0 || f >= 1.0 {
            panic!("Load factor must be greater than 0 and smaller than 1");
        }
        if expected < 0 {
            panic!("The expected number of elements must be nonnegative");
        }
        let n = HashCommon::array_size(expected, f);
        LongOpenHashSet {
            key: vec![0; n as usize + 1],
            mask: n - 1,
            contains_null: false,
            n,
            max_fill: HashCommon::max_fill(n, f),
            min_n: n,
            size: 0,
            f,
        }
    }

    /// Whether the key was new.
    pub fn add(&mut self, k: i64) -> bool {
        if k == 0 {
            if self.contains_null {
                return false;
            }
            self.contains_null = true;
        } else {
            let mut pos = HashCommon::mix_i64(k) as i32 & self.mask;
            loop {
                let current = self.key[pos as usize];
                if current == 0 {
                    break;
                }
                if current == k {
                    return false;
                }
                pos = (pos + 1) & self.mask;
            }
            self.key[pos as usize] = k;
        }

        let size = self.size;
        self.size += 1;
        if size >= self.max_fill {
            self.rehash(HashCommon::array_size(self.size + 1, self.f));
        }
        true
    }

    pub fn contains(&self, k: i64) -> bool {
        if k == 0 {
            return self.contains_null;
        }
        let mut pos = HashCommon::mix_i64(k) as i32 & self.mask;
        loop {
            let current = self.key[pos as usize];
            if current == 0 {
                return false;
            }
            if current == k {
                return true;
            }
            pos = (pos + 1) & self.mask;
        }
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        if self.size == 0 {
            return;
        }
        self.size = 0;
        self.contains_null = false;
        self.key.fill(0);
    }

    /// `trim(int)`: shrinks the table to the smallest that takes `n` keys, if that's smaller and
    /// takes the keys there are. Whether it could (Java's `false` is running out of memory).
    pub fn trim(&mut self, n: i32) -> bool {
        let l = HashCommon::next_power_of_two_i32(((n as f32 / self.f) as f64).ceil() as i32);
        if l >= self.n || self.size > HashCommon::max_fill(l, self.f) {
            return true;
        }
        self.rehash(l);
        true
    }

    /// Each key into a table of `new_n` slots, from the old table's last slot down.
    fn rehash(&mut self, new_n: i32) {
        let mask = new_n - 1;
        let mut new_key = vec![0i64; new_n as usize + 1];
        let mut i = self.n;
        let mut j = if self.contains_null {
            self.size - 1
        } else {
            self.size
        };
        while j != 0 {
            j -= 1;
            i -= 1;
            while self.key[i as usize] == 0 {
                i -= 1;
            }
            let mut pos = HashCommon::mix_i64(self.key[i as usize]) as i32 & mask;
            while new_key[pos as usize] != 0 {
                pos = (pos + 1) & mask;
            }
            new_key[pos as usize] = self.key[i as usize];
        }

        self.n = new_n;
        self.mask = mask;
        self.max_fill = HashCommon::max_fill(self.n, self.f);
        self.key = new_key;
    }

    /// `iterator()`: 0 first if it's there, then the table from its last slot down.
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        let null = self.contains_null.then_some(0);
        null.into_iter().chain(
            self.key[..self.n as usize]
                .iter()
                .rev()
                .copied()
                .filter(|&key| key != 0),
        )
    }

    /// `minN`, the table it was made with, which Java's `trim()` without a size shrinks to.
    pub fn min_n(&self) -> i32 {
        self.min_n
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::LongOpenHashSet;
    use crate::golden::{Decode, Golden};

    /// Programs of adds, clears, trims and iterations: every output and iteration order as
    /// fastutil's.
    #[test]
    fn programs_match_fastutil() {
        let golden = Golden::load("it.unimi.dsi.fastutil.longs.LongOpenHashSet");
        let programs = golden.cases("programs");
        assert!(programs.len() >= 200);
        for (number, (args, expected)) in programs.iter().enumerate() {
            let mut set = LongOpenHashSet::new(
                args[0].as_i64().expect("expected") as i32,
                f32::decode(&args[1]),
            );
            let ops = args[2].as_array().expect("ops");
            let expected = expected.as_array().expect("outputs");
            for (index, op) in ops.iter().enumerate() {
                let op = op.as_array().expect("an op");
                let long = |i: usize| op[i].as_i64().expect("a long");
                let output = match op[0].as_str().expect("its name") {
                    "add" => json!(set.add(long(1))),
                    "contains" => json!(set.contains(long(1))),
                    "clear" => {
                        set.clear();
                        Value::Null
                    }
                    "trim" => json!(set.trim(long(1) as i32)),
                    "size" => json!(set.size()),
                    "iterate" => json!(set.iter().collect::<Vec<_>>()),
                    other => panic!("no op {other}"),
                };
                assert_eq!(
                    output, expected[index],
                    "program {number}, op {index} {op:?}"
                );
            }
        }
    }
}
