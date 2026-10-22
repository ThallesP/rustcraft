//! java/util/HashMap.java
//!
//! The map a `CompoundTag` keeps its tags in, whose order is part of the game's behaviour: NBT
//! writes a compound's tags in the order its map iterates them. That order goes by the map's
//! table: its bins in index order, and each bin's entries in the order they came, which a key's
//! hash and the table's length decide. So the port keeps Java's table: a `Vec` of bins, each a
//! `Vec` of the bin's entries in its linked list's order, grown and split the way `resize` does.
//! Java's `null` table is an empty `Vec`.
//!
//! Not ported yet: tree bins. A bin that reaches 9 entries in a table of 64 or more becomes a
//! red-black tree (`treeifyBin`), whose linked order then follows the tree (`TreeNode`), which
//! needs `TreeNode`'s insertion, removal, rotations and `split`. Only keys whose spread hashes agree
//! in their low six bits or more meet there, 9 of them in one compound, so the port panics there,
//! and the golden suite's programs that get there are pending. A bin of 9 in a smaller table makes
//! the table grow, which is ported.
//!
//! Only what `CompoundTag` calls is ported: the constructors (`new HashMap<>()` and
//! `new HashMap<>(map)`, which sizes its table for the map's size), `size`, `isEmpty`, `get`,
//! `containsKey`, `put`, `remove`, iterating it, and `AbstractMap`'s `equals` and `hashCode`.
//! The load factor is always the default 0.75. Keys are never null, which Rust can't have.

/*
 * Copyright (c) 1997, 2025, Oracle and/or its affiliates. All rights reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the "Classpath" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 */

use std::borrow::Borrow;

use crate::java::lang::object::HashCode;

const DEFAULT_INITIAL_CAPACITY: i32 = 1 << 4;
const MAXIMUM_CAPACITY: i32 = 1 << 30;
const DEFAULT_LOAD_FACTOR: f32 = 0.75;
const TREEIFY_THRESHOLD: i32 = 8;
const MIN_TREEIFY_CAPACITY: i32 = 64;

/// A hash table of keys and values, iterated bin by bin.
#[derive(Clone, Debug)]
pub struct HashMap<K, V> {
    table: Vec<Vec<Node<K, V>>>,
    size: i32,
    threshold: i32,
}

#[derive(Clone, Debug)]
struct Node<K, V> {
    hash: i32,
    key: K,
    value: V,
}

/// `HashMap.hash`: the key's `hashCode` with its high half folded into its low half, since a
/// table's index only takes the low bits.
fn hash<Q: HashCode + ?Sized>(key: &Q) -> i32 {
    let h = key.hash_code();
    h ^ ((h as u32) >> 16) as i32
}

/// The smallest power of two at least `cap`.
fn table_size_for(cap: i32) -> i32 {
    let n = (-1i32 as u32).wrapping_shr(cap.wrapping_sub(1).leading_zeros()) as i32;
    if n < 0 {
        1
    } else if n >= MAXIMUM_CAPACITY {
        MAXIMUM_CAPACITY
    } else {
        n + 1
    }
}

impl<K, V> HashMap<K, V> {
    /// `new HashMap<>()`: no table until the first `put`, which makes one of 16 bins.
    pub fn new() -> HashMap<K, V> {
        HashMap {
            table: Vec::new(),
            size: 0,
            threshold: 0,
        }
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// The entries in iteration order: bin by bin, each bin's in the order they came.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.table
            .iter()
            .flatten()
            .map(|node| (&node.key, &node.value))
    }

    /// `keySet()`'s order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    /// `values()`'s order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    /// Makes the table, or doubles it, moving each bin's entries to the same index or that index
    /// plus the old length, in the order they were.
    fn resize(&mut self) {
        let old_cap = self.table.len() as i32;
        let old_thr = self.threshold;
        let mut new_thr = 0;
        let new_cap;
        if old_cap > 0 {
            if old_cap >= MAXIMUM_CAPACITY {
                self.threshold = i32::MAX;
                return;
            }
            new_cap = old_cap << 1;
            if new_cap < MAXIMUM_CAPACITY && old_cap >= DEFAULT_INITIAL_CAPACITY {
                new_thr = old_thr << 1;
            }
        } else if old_thr > 0 {
            new_cap = old_thr;
        } else {
            new_cap = DEFAULT_INITIAL_CAPACITY;
            new_thr = (DEFAULT_LOAD_FACTOR * DEFAULT_INITIAL_CAPACITY as f32) as i32;
        }
        if new_thr == 0 {
            let ft = new_cap as f32 * DEFAULT_LOAD_FACTOR;
            new_thr = if new_cap < MAXIMUM_CAPACITY && ft < MAXIMUM_CAPACITY as f32 {
                ft as i32
            } else {
                i32::MAX
            };
        }
        self.threshold = new_thr;

        let old_tab = std::mem::take(&mut self.table);
        self.table = (0..new_cap).map(|_| Vec::new()).collect();
        for (j, bin) in old_tab.into_iter().enumerate() {
            for node in bin {
                let index = if node.hash & old_cap == 0 {
                    j
                } else {
                    j + old_cap as usize
                };
                self.table[index].push(node);
            }
        }
    }

    fn index_for(&self, hash: i32) -> usize {
        (hash & (self.table.len() as i32 - 1)) as usize
    }
}

impl<K: HashCode + Eq, V> HashMap<K, V> {
    /// `new HashMap<>(map)`: a table sized for the map's size (the smallest power of two over
    /// size / 0.75), then its entries put in its order.
    pub fn from_map(map: &HashMap<K, V>) -> HashMap<K, V>
    where
        K: Clone,
        V: Clone,
    {
        let mut copy = HashMap::new();
        let s = map.size;
        if s > 0 {
            let dt = (f64::from(s) / f64::from(DEFAULT_LOAD_FACTOR)).ceil();
            let t = if dt < f64::from(MAXIMUM_CAPACITY) {
                dt as i32
            } else {
                MAXIMUM_CAPACITY
            };
            if t > copy.threshold {
                copy.threshold = table_size_for(t);
            }
            for (key, value) in map.iter() {
                copy.put(key.clone(), value.clone());
            }
        }
        copy
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        self.find(key)
            .map(|(index, position)| &self.table[index][position].value)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        self.find(key)
            .map(|(index, position)| &mut self.table[index][position].value)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        self.find(key).is_some()
    }

    /// Replaces the value of a key it has, where it is, or adds the key at the end of its bin. A
    /// bin of 9 makes a table under 64 grow; then, like any put of a new key, so does a size
    /// past the threshold. Returns the value it replaced.
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        let hash = hash(&key);
        if self.table.is_empty() {
            self.resize();
        }
        let index = self.index_for(hash);
        let bin = &mut self.table[index];
        if let Some(node) = bin
            .iter_mut()
            .find(|node| node.hash == hash && node.key == key)
        {
            return Some(std::mem::replace(&mut node.value, value));
        }
        bin.push(Node { hash, key, value });
        // Java counts the nodes it passed before the last: 7 or more means the bin has 9 now.
        if bin.len() as i32 - 2 >= TREEIFY_THRESHOLD - 1 {
            self.treeify_bin();
        }
        self.size += 1;
        if self.size > self.threshold {
            self.resize();
        }
        None
    }

    /// Takes the key's entry out of its bin, the others keeping their order. Returns its value.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        let (index, position) = self.find(key)?;
        self.size -= 1;
        Some(self.table[index].remove(position).value)
    }

    /// `treeifyBin`: grows a table under 64. A bigger one would make the bin a tree.
    fn treeify_bin(&mut self) {
        if (self.table.len() as i32) < MIN_TREEIFY_CAPACITY {
            self.resize();
        } else {
            panic!("HashMap's tree bins aren't ported");
        }
    }

    // The bin and the position in it of the key's entry.
    fn find<Q>(&self, key: &Q) -> Option<(usize, usize)>
    where
        K: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        if self.table.is_empty() {
            return None;
        }
        let hash = hash(key);
        let index = self.index_for(hash);
        self.table[index]
            .iter()
            .position(|node| node.hash == hash && node.key.borrow() == key)
            .map(|position| (index, position))
    }

    /// `AbstractMap.hashCode`: the sum of each entry's key's hash xor its value's.
    pub fn hash_code(&self) -> i32
    where
        V: HashCode,
    {
        self.iter().fold(0, |hash: i32, (key, value)| {
            hash.wrapping_add(key.hash_code() ^ value.hash_code())
        })
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> HashMap<K, V> {
        HashMap::new()
    }
}

/// `AbstractMap.equals`: the same size, and every key mapped to an equal value, in any order.
impl<K: HashCode + Eq, V: PartialEq> PartialEq for HashMap<K, V> {
    fn eq(&self, other: &HashMap<K, V>) -> bool {
        self.size == other.size
            && self
                .iter()
                .all(|(key, value)| other.get(key).is_some_and(|other| value == other))
    }
}

#[cfg(test)]
impl<K, V> HashMap<K, V> {
    /// The table's length, 0 before it's made, which the golden files record with the entries.
    pub fn capacity(&self) -> i32 {
        self.table.len() as i32
    }

    /// The map vanilla recorded: a table of `capacity` bins (none for 0), with the entries put
    /// in its bins in their order, as they were, without growing it.
    pub fn from_table(capacity: i32, entries: Vec<(K, V)>) -> HashMap<K, V>
    where
        K: HashCode,
    {
        let mut map = HashMap::new();
        if capacity > 0 {
            map.table = (0..capacity).map(|_| Vec::new()).collect();
            map.threshold = (capacity as f32 * DEFAULT_LOAD_FACTOR) as i32;
        }
        for (key, value) in entries {
            let hash = hash(&key);
            let index = map.index_for(hash);
            map.table[index].push(Node { hash, key, value });
            map.size += 1;
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::HashMap;
    use crate::golden::{Decode, Encode, Golden};

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.util.HashMap");
        golden.check("programs", |program: Value| run(&program));
        golden.pending(
            "treeified programs",
            "tree bins aren't ported: a bin of 9 in a table of 64 or more",
        );
        golden.finish();
    }

    // An op's output, and the map's table length and entries in order after it.
    type Step = (Box<dyn Encode>, (i32, Vec<(String, i32)>));

    // Mirrors HashMapSuite in oracle/golden/.
    fn run(program: &Value) -> Vec<Step> {
        let mut map: HashMap<String, i32> = HashMap::new();
        let mut steps = Vec::new();
        for op in program.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let key = || String::decode(&op[1]);
            let output: Box<dyn Encode> = match op[0].as_str().expect("ops start with their name") {
                "put" => Box::new(map.put(key(), i32::decode(&op[2]))),
                "remove" => Box::new(map.remove(key().as_str())),
                "get" => Box::new(map.get(key().as_str()).copied()),
                "containsKey" => Box::new(map.contains_key(key().as_str())),
                "copy" => {
                    map = HashMap::from_map(&map);
                    Box::new(())
                }
                name => panic!("unknown op {name}"),
            };
            let entries = map
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect();
            steps.push((output, (map.capacity(), entries)));
        }
        steps
    }
}
