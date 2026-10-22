//! net/minecraft/core/IdMapper.java
//!
//! Java's `Reference2IntMap` maps things by reference, which is a `HashMap` of values that compare
//! like references (`BlockState`).

use std::collections::HashMap;
use std::hash::Hash;

/// Things numbered by id, both ways: `Block.BLOCK_STATE_REGISTRY` numbers every block state.
pub struct IdMapper<T> {
    next_id: i32,
    t_to_id: HashMap<T, i32>,
    id_to_t: Vec<Option<T>>,
}

impl<T: Copy + Eq + Hash> IdMapper<T> {
    /// `IdMapper(expectedSize)`.
    pub fn with_expected_size(expected_size: usize) -> IdMapper<T> {
        IdMapper {
            next_id: 0,
            t_to_id: HashMap::with_capacity(expected_size),
            id_to_t: Vec::with_capacity(expected_size),
        }
    }

    pub fn add_mapping(&mut self, thing: T, id: i32) {
        self.t_to_id.insert(thing, id);
        while self.id_to_t.len() <= id as usize {
            self.id_to_t.push(None);
        }
        self.id_to_t[id as usize] = Some(thing);
        if self.next_id <= id {
            self.next_id = id + 1;
        }
    }

    pub fn add(&mut self, thing: T) {
        self.add_mapping(thing, self.next_id);
    }

    /// The thing's id, or -1.
    pub fn get_id(&self, thing: T) -> i32 {
        self.t_to_id.get(&thing).copied().unwrap_or(-1)
    }

    pub fn by_id(&self, id: i32) -> Option<T> {
        if id >= 0 && (id as usize) < self.id_to_t.len() {
            self.id_to_t[id as usize]
        } else {
            None
        }
    }

    /// Every thing, by id, skipping ids nothing has.
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.id_to_t.iter().flatten().copied()
    }

    pub fn contains(&self, id: i32) -> bool {
        self.by_id(id).is_some()
    }

    pub fn size(&self) -> i32 {
        self.t_to_id.len() as i32
    }
}

impl<T: Copy + Eq + Hash> Default for IdMapper<T> {
    /// `IdMapper()`, which expects 512 things.
    fn default() -> IdMapper<T> {
        IdMapper::with_expected_size(512)
    }
}
