//! com/google/common/collect/ImmutableMultimap.java
//!
//! Only what the game reaches through authlib's `PropertyMap`: `of()`, `builder()` with the
//! `Builder`'s `put` and `build`, `copyOf`, `size`, `isEmpty`, `values`, `equals`, `hashCode` and
//! `toString`. Each of them makes or is an `ImmutableListMultimap` (`ImmutableListMultimap.java`),
//! so that's what this is: an `ImmutableMap` from each key to the `ImmutableList` of its values.
//! The keys keep the order they were first put in, and each key's values the order they were put
//! in, so `values()` gives the first key's values, then the second's, and not the order they were
//! put in. The map is a `Vec` of the keys with their values, in that order, searched from the
//! start: the game's multimaps hold a handful of keys. Guava keeps the number of values in a field
//! of its own, which `size()` adds up here instead, so it can't disagree with the lists.
//! `ImmutableMultimap`'s own suite isn't needed: `PropertyMap`'s goldens check it, since a
//! `PropertyMap` forwards to its multimap.
//!
//! Guava rejects null keys and values, which Rust can't have.

/*
 * Copyright (C) 2008 The Guava Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust from Guava 33.6.0-jre, and changed: see above.
 */

use std::fmt;

use crate::java::lang::object::HashCode;

#[derive(Clone, Debug)]
pub struct ImmutableMultimap<K, V> {
    map: Vec<(K, Vec<V>)>,
}

impl<K, V> ImmutableMultimap<K, V> {
    /// `of()`: the empty multimap.
    pub const fn of() -> ImmutableMultimap<K, V> {
        ImmutableMultimap { map: Vec::new() }
    }

    pub fn builder() -> Builder<K, V> {
        Builder {
            builder_map: Vec::new(),
        }
    }

    /// `copyOf(Multimap)`: an immutable multimap that isn't a view of a bigger one is returned as
    /// it is, which every multimap the game makes is.
    pub fn copy_of(multimap: ImmutableMultimap<K, V>) -> ImmutableMultimap<K, V> {
        multimap
    }

    /// How many values there are, under every key.
    pub fn size(&self) -> i32 {
        self.map.iter().fold(0, |size: i32, (_, values)| {
            size.wrapping_add(values.len() as i32)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// The first key's values, then the second key's, and so on.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.map.iter().flat_map(|(_, values)| values)
    }

    // `asMap().get(key)`.
    fn values_of(&self, key: &K) -> Option<&Vec<V>>
    where
        K: PartialEq,
    {
        self.map
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, values)| values)
    }
}

impl<K: HashCode, V: HashCode> ImmutableMultimap<K, V> {
    /// `asMap().hashCode()`: the sum of its entries', each its key's hash xor its list's, so the
    /// keys' order doesn't change it.
    pub fn hash_code(&self) -> i32 {
        self.map.iter().fold(0, |hash_code, (key, values)| {
            hash_code.wrapping_add(key.hash_code() ^ list_hash_code(values))
        })
    }
}

// `ImmutableList.hashCode()`: each element's hash, added to 31 times the ones before, from 1.
fn list_hash_code<V: HashCode>(values: &[V]) -> i32 {
    values.iter().fold(1, |hash_code, value| {
        hash_code.wrapping_mul(31).wrapping_add(value.hash_code())
    })
}

/// `equals`: `asMap()`s with equal entry sets, so the same keys in any order, each with an equal
/// list of values, in the same order.
impl<K: PartialEq, V: PartialEq> PartialEq for ImmutableMultimap<K, V> {
    fn eq(&self, other: &ImmutableMultimap<K, V>) -> bool {
        self.map.len() == other.map.len()
            && other
                .map
                .iter()
                .all(|(key, values)| self.values_of(key) == Some(values))
    }
}

/// `asMap().toString()`: `{textures=[Property[name=textures, …]], …}`.
impl<K: fmt::Display, V: fmt::Display> fmt::Display for ImmutableMultimap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (index, (key, values)) in self.map.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{key}=[")?;
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{value}")?;
            }
            write!(f, "]")?;
        }
        write!(f, "}}")
    }
}

/// `ImmutableMultimap.Builder`: each key in the order it's first put, with its values in the
/// order they're put.
pub struct Builder<K, V> {
    builder_map: Vec<(K, Vec<V>)>,
}

impl<K: PartialEq, V> Builder<K, V> {
    pub fn put(&mut self, key: K, value: V) -> &mut Builder<K, V> {
        match self
            .builder_map
            .iter_mut()
            .find(|(candidate, _)| *candidate == key)
        {
            Some((_, values)) => values.push(value),
            None => self.builder_map.push((key, vec![value])),
        }
        self
    }

    /// An `ImmutableListMultimap` of what was put.
    pub fn build(self) -> ImmutableMultimap<K, V> {
        ImmutableMultimap {
            map: self.builder_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ImmutableMultimap;
    use crate::golden::{Decode, Encode, elements};

    // Vanilla writes a multimap as its `asMap()`'s entries, `[[key, [value, …]], …]`.
    impl<K: Decode, V: Decode> Decode for ImmutableMultimap<K, V> {
        fn decode(value: &Value) -> Self {
            let map = Vec::<Value>::decode(value)
                .iter()
                .map(|entry: &Value| {
                    let [key, values] = elements(entry);
                    (K::decode(key), Vec::decode(values))
                })
                .collect();
            ImmutableMultimap { map }
        }
    }

    impl<K: Encode, V: Encode> Encode for ImmutableMultimap<K, V> {
        fn encode(&self) -> Value {
            self.map
                .iter()
                .map(|(key, values)| Value::from(vec![key.encode(), values.encode()]))
                .collect()
        }

        fn matches(&self, expected: &Value) -> bool {
            expected.as_array().is_some_and(|expected| {
                expected.len() == self.map.len()
                    && self.map.iter().zip(expected).all(|((key, values), entry)| {
                        matches!(entry.as_array().map(Vec::as_slice), Some([expected_key, expected_values])
                            if key.matches(expected_key) && values.matches(expected_values))
                    })
            })
        }
    }
}
