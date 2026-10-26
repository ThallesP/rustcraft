//! it/unimi/dsi/fastutil/objects/Object2ObjectOpenHashMap.java
//!
//! From fastutil's bytecode (it publishes no sources with the game), what a chunk's block entities
//! go through (`ChunkAccess.blockEntities`): a table of keys placed by `HashCommon.mix` of their
//! `hashCode` and probed linearly, grown past its load factor and shrunk when a removal leaves it
//! a quarter full, a removal shifting the keys after it back (`shiftKeys`), and iterated from the
//! table's last slot down. So the order the game sees its entries in is fastutil's. The null key,
//! which Java keeps apart, can't be a Rust key, and removing while iterating isn't ported.

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
use crate::java::lang::object::HashCode;

/// A map in an open-addressed table: each slot a key and its value, or neither.
#[derive(Clone, Debug)]
pub struct Object2ObjectOpenHashMap<K, V> {
    /// `key` and `value`, one slot each; Java's tables have one more, the null key's.
    table: Vec<Option<(K, V)>>,
    mask: i32,
    n: i32,
    max_fill: i32,
    min_n: i32,
    size: i32,
    f: f32,
}

impl<K: HashCode + PartialEq, V> Default for Object2ObjectOpenHashMap<K, V> {
    fn default() -> Self {
        Object2ObjectOpenHashMap::new()
    }
}

impl<K: HashCode + PartialEq, V> Object2ObjectOpenHashMap<K, V> {
    /// `Object2ObjectOpenHashMap()`: for 16 entries at a load factor of 0.75.
    pub fn new() -> Object2ObjectOpenHashMap<K, V> {
        Object2ObjectOpenHashMap::with_capacity(16, 0.75)
    }

    /// `Object2ObjectOpenHashMap(int, float)`. Panics for a load factor outside (0, 1) or a
    /// negative `expected`, where Java throws.
    pub fn with_capacity(expected: i32, f: f32) -> Object2ObjectOpenHashMap<K, V> {
        if f <= 0.0 || f >= 1.0 {
            panic!("Load factor must be greater than 0 and smaller than 1");
        }
        if expected < 0 {
            panic!("The expected number of elements must be nonnegative");
        }
        let n = HashCommon::array_size(expected, f);
        Object2ObjectOpenHashMap {
            table: (0..n).map(|_| None).collect(),
            mask: n - 1,
            n,
            max_fill: HashCommon::max_fill(n, f),
            min_n: n,
            size: 0,
            f,
        }
    }

    fn slot(&self, k: &K) -> i32 {
        HashCommon::mix_i32(k.hash_code()) & self.mask
    }

    /// `find`: the key's slot, or `-(the free slot it would go in) - 1`.
    fn find(&self, k: &K) -> i32 {
        let mut pos = self.slot(k);
        loop {
            match &self.table[pos as usize] {
                None => return -(pos + 1),
                Some((curr, _)) if curr == k => return pos,
                Some(_) => pos = (pos + 1) & self.mask,
            }
        }
    }

    /// `put`: the value it replaced, if the key was there.
    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        let pos = self.find(&k);
        if pos < 0 {
            self.insert(-pos - 1, k, v);
            return None;
        }
        let entry = self.table[pos as usize].as_mut().expect("found");
        Some(std::mem::replace(&mut entry.1, v))
    }

    fn insert(&mut self, pos: i32, k: K, v: V) {
        self.table[pos as usize] = Some((k, v));
        let size = self.size;
        self.size += 1;
        if size >= self.max_fill {
            self.rehash(HashCommon::array_size(self.size + 1, self.f));
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let pos = self.find(k);
        (pos >= 0).then(|| &self.table[pos as usize].as_ref().expect("found").1)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let pos = self.find(k);
        if pos < 0 {
            return None;
        }
        Some(&mut self.table[pos as usize].as_mut().expect("found").1)
    }

    pub fn contains_key(&self, k: &K) -> bool {
        self.find(k) >= 0
    }

    /// `remove`: the key's value, if it was there; the table shrinks by half when that leaves
    /// it under a quarter of what it takes, down to the size it was made with.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        let pos = self.find(k);
        if pos < 0 {
            return None;
        }
        self.remove_entry(pos)
    }

    fn remove_entry(&mut self, pos: i32) -> Option<V> {
        let removed = self.table[pos as usize].take().map(|(_, value)| value);
        self.size -= 1;
        self.shift_keys(pos);
        if self.n > self.min_n && self.size < self.max_fill / 4 && self.n > 16 {
            self.rehash(self.n / 2);
        }
        removed
    }

    /// `shiftKeys`: the entries after a freed slot, up to the next free one, moved back into it
    /// where their probe sequences would find them.
    fn shift_keys(&mut self, mut pos: i32) {
        loop {
            let last = pos;
            pos = (last + 1) & self.mask;
            loop {
                let Some((curr, _)) = &self.table[pos as usize] else {
                    self.table[last as usize] = None;
                    return;
                };
                let slot = self.slot(curr);
                if if last <= pos {
                    last >= slot || slot > pos
                } else {
                    last >= slot && slot > pos
                } {
                    break;
                }
                pos = (pos + 1) & self.mask;
            }
            self.table[last as usize] = self.table[pos as usize].take();
        }
    }

    /// `rehash`: every entry into a table of `new_n` slots, from the old table's last slot down.
    fn rehash(&mut self, new_n: i32) {
        let mask = new_n - 1;
        let mut new_table: Vec<Option<(K, V)>> = (0..new_n).map(|_| None).collect();
        for entry in self.table.drain(..).rev().flatten() {
            let mut pos = HashCommon::mix_i32(entry.0.hash_code()) & mask;
            while new_table[pos as usize].is_some() {
                pos = (pos + 1) & mask;
            }
            new_table[pos as usize] = Some(entry);
        }

        self.n = new_n;
        self.mask = mask;
        self.max_fill = HashCommon::max_fill(self.n, self.f);
        self.table = new_table;
    }

    /// `clear()`: every slot emptied, the table as large as it was.
    pub fn clear(&mut self) {
        if self.size == 0 {
            return;
        }
        self.size = 0;
        self.table.iter_mut().for_each(|slot| *slot = None);
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// The entries in the order Java's iterators hand them out: the table from its last slot down.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.table
            .iter()
            .rev()
            .flatten()
            .map(|(key, value)| (key, value))
    }

    /// `values()`, in the entries' order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::Object2ObjectOpenHashMap;
    use crate::golden::Golden;
    use crate::net::minecraft::core::block_pos::BlockPos;

    /// Programs of puts, removes, gets, clears and iterations on a map of blocks: every output
    /// and iteration order as fastutil's, through its tables growing and shrinking.
    #[test]
    fn programs_match_fastutil() {
        let golden = Golden::load("it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap");
        let programs = golden.cases("programs");
        assert_eq!(programs.len(), 240);
        for (number, (args, expected)) in programs.iter().enumerate() {
            let mut map = Object2ObjectOpenHashMap::<BlockPos, i64>::new();
            let ops = args[0].as_array().expect("ops");
            let expected = expected.as_array().expect("outputs");
            assert_eq!(ops.len(), expected.len());
            for (index, op) in ops.iter().enumerate() {
                let op = op.as_array().expect("an op");
                let int = |i: usize| op[i].as_i64().expect("an int");
                let pos = || BlockPos::new(int(1) as i32, int(2) as i32, int(3) as i32);
                let output = match op[0].as_str().expect("its name") {
                    "put" => json!(map.put(pos(), int(4))),
                    "remove" => json!(map.remove(&pos())),
                    "get" => json!(map.get(&pos())),
                    "clear" => {
                        map.clear();
                        Value::Null
                    }
                    "size" => json!(map.size()),
                    "iterate" => map
                        .iter()
                        .map(|(pos, value)| json!([pos.get_x(), pos.get_y(), pos.get_z(), value]))
                        .collect(),
                    other => panic!("no op {other}"),
                };
                assert_eq!(
                    output, expected[index],
                    "program {number}, op {index} {op:?}"
                );
            }
        }
        golden.finish();
    }
}
