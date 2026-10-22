//! net/minecraft/util/CrudeIncrementalIntIdentityHashBiMap.java
//!
//! Java finds a key's slot by its identity hash (`System.identityHashCode`), which changes from
//! run to run, so no answer of the map may depend on where a key lands, and none does: lookups
//! probe until they find the key or an empty slot, ids come from `byId`, and `nextId` only skips
//! ids `byId` already has. The port hashes the key with Rust's (fixed) default hasher instead.
//! Keys compare with `==`, which is Java's reference comparison for the keys it holds (block
//! states, biome holders). The one place a hash shows is a key added twice: which copy `getId`
//! finds first can change when the table grows, which no palette does.
//!
//! `contains(K)` and `contains(int)` are `contains` and `contains_id`.

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::util::mth::Mth;

const NOT_FOUND: i32 = -1;
const LOADFACTOR: f32 = 0.8;

/// Keys numbered from 0 as they're added, found both ways: a hash table of keys to ids
/// (linear probing) and an array of keys by id.
#[derive(Clone, Debug)]
pub struct CrudeIncrementalIntIdentityHashBiMap<K> {
    keys: Vec<Option<K>>,
    values: Vec<i32>,
    by_id: Vec<Option<K>>,
    next_id: i32,
    size: i32,
}

impl<K: Clone + PartialEq + Hash> CrudeIncrementalIntIdentityHashBiMap<K> {
    fn with_capacity(capacity: i32) -> CrudeIncrementalIntIdentityHashBiMap<K> {
        assert!(capacity >= 0, "{capacity}");
        CrudeIncrementalIntIdentityHashBiMap {
            keys: vec![None; capacity as usize],
            values: vec![0; capacity as usize],
            by_id: vec![None; capacity as usize],
            next_id: 0,
            size: 0,
        }
    }

    /// A table big enough for `initial_capacity` keys at its load factor.
    pub fn create(initial_capacity: i32) -> CrudeIncrementalIntIdentityHashBiMap<K> {
        CrudeIncrementalIntIdentityHashBiMap::with_capacity(
            (initial_capacity as f32 / 0.8f32) as i32,
        )
    }

    pub fn get_id(&self, thing: &K) -> i32 {
        self.get_value(self.index_of(thing, self.hash(thing)))
    }

    pub fn by_id(&self, id: i32) -> Option<K> {
        if id >= 0 && (id as usize) < self.by_id.len() {
            self.by_id[id as usize].clone()
        } else {
            None
        }
    }

    fn get_value(&self, index: i32) -> i32 {
        if index == NOT_FOUND {
            NOT_FOUND
        } else {
            self.values[index as usize]
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get_id(key) != NOT_FOUND
    }

    pub fn contains_id(&self, id: i32) -> bool {
        self.by_id(id).is_some()
    }

    /// Adds the key under the lowest free id, which it returns. A key already in gets added again.
    pub fn add(&mut self, key: K) -> i32 {
        let value = self.next_id();
        self.add_mapping(key, value);
        value
    }

    fn next_id(&mut self) -> i32 {
        while (self.next_id as usize) < self.by_id.len()
            && self.by_id[self.next_id as usize].is_some()
        {
            self.next_id += 1;
        }

        self.next_id
    }

    fn grow(&mut self, new_size: i32) {
        let old_keys = std::mem::take(&mut self.keys);
        let old_values = std::mem::take(&mut self.values);
        let mut resized = CrudeIncrementalIntIdentityHashBiMap::with_capacity(new_size);

        for (key, value) in old_keys.into_iter().zip(old_values) {
            if let Some(key) = key {
                resized.add_mapping(key, value);
            }
        }

        *self = resized;
    }

    /// Panics where Java throws: an id past what the table grows to, or negative, after the key
    /// is in the table (but not in `byId`).
    pub fn add_mapping(&mut self, key: K, id: i32) {
        let min_size = id.max(self.size + 1);
        if min_size as f32 >= self.keys.len() as i32 as f32 * LOADFACTOR {
            let mut new_size = (self.keys.len() as i32) << 1;

            while new_size < id {
                new_size <<= 1;
            }

            self.grow(new_size);
        }

        let index = self.find_empty(self.hash(&key)) as usize;
        self.keys[index] = Some(key.clone());
        self.values[index] = id;
        let slot = usize::try_from(id)
            .ok()
            .and_then(|id| self.by_id.get_mut(id))
            .unwrap_or_else(|| panic!("Index {id} out of bounds"));
        *slot = Some(key);
        self.size += 1;
        if id == self.next_id {
            self.next_id += 1;
        }
    }

    /// Panics on an empty table, where Java's `%` throws `ArithmeticException`.
    fn hash(&self, key: &K) -> i32 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let identity_hash = hasher.finish() as i32;
        (Mth::murmur_hash3_mixer(identity_hash) & i32::MAX) % self.keys.len() as i32
    }

    fn index_of(&self, key: &K, start_from: i32) -> i32 {
        let start_from = start_from as usize;
        for i in (start_from..self.keys.len()).chain(0..start_from) {
            match &self.keys[i] {
                Some(candidate) if candidate == key => return i as i32,
                None => return NOT_FOUND,
                Some(_) => {}
            }
        }

        NOT_FOUND
    }

    fn find_empty(&self, start_from: i32) -> i32 {
        let start_from = start_from as usize;
        for i in (start_from..self.keys.len()).chain(0..start_from) {
            if self.keys[i].is_none() {
                return i as i32;
            }
        }

        panic!("Overflowed :(");
    }

    /// Every key, by id.
    pub fn iter(&self) -> impl Iterator<Item = &K> + '_ {
        self.by_id.iter().flatten()
    }

    pub fn clear(&mut self) {
        self.keys.fill(None);
        self.by_id.fill(None);
        self.next_id = 0;
        self.size = 0;
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn copy(&self) -> CrudeIncrementalIntIdentityHashBiMap<K> {
        self.clone()
    }
}

impl<K: Clone + PartialEq + Hash> IdMap<K> for CrudeIncrementalIntIdentityHashBiMap<K> {
    fn get_id(&self, thing: &K) -> i32 {
        CrudeIncrementalIntIdentityHashBiMap::get_id(self, thing)
    }

    fn by_id(&self, id: i32) -> Option<K> {
        CrudeIncrementalIntIdentityHashBiMap::by_id(self, id)
    }

    fn size(&self) -> i32 {
        CrudeIncrementalIntIdentityHashBiMap::size(self)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::CrudeIncrementalIntIdentityHashBiMap;
    use crate::golden::{Decode, Encode, Golden, Thrown, catch};
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap");
        golden.check("programs", |capacity: i32, ops: Value| {
            run(CrudeIncrementalIntIdentityHashBiMap::create(capacity), &ops)
        });
        golden.finish();
    }

    // Mirrors BiMapSuite.run in oracle/golden/: `[output, size]` after each op, a panic ending
    // them.
    fn run(
        mut map: CrudeIncrementalIntIdentityHashBiMap<BlockState>,
        ops: &Value,
    ) -> Vec<Box<dyn Encode>> {
        let mut steps: Vec<Box<dyn Encode>> = Vec::new();
        for op in ops.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let state = |index: usize| BlockState::decode(&op[index]);
            let int = |index: usize| i32::decode(&op[index]);
            let step = catch(|| -> Box<dyn Encode> {
                let output: Box<dyn Encode> =
                    match op[0].as_str().expect("ops start with their name") {
                        "add" => Box::new(map.add(state(1))),
                        "addMapping" => {
                            map.add_mapping(state(1), int(2));
                            Box::new(())
                        }
                        "getId" => Box::new(map.get_id(&state(1))),
                        "byId" => Box::new(map.by_id(int(1))),
                        "containsKey" => Box::new(map.contains(&state(1))),
                        "containsId" => Box::new(map.contains_id(int(1))),
                        "size" => Box::new(map.size()),
                        "clear" => {
                            map.clear();
                            Box::new(())
                        }
                        "copy" => {
                            map = map.copy();
                            Box::new(())
                        }
                        "iterator" => Box::new(map.iter().copied().collect::<Vec<_>>()),
                        name => panic!("unknown op {name}"),
                    };
                Box::new((output, map.size()))
            });
            match step {
                Ok(step) => steps.push(step),
                Err(_) => {
                    steps.push(Box::new(Thrown));
                    break;
                }
            }
        }
        steps
    }
}
