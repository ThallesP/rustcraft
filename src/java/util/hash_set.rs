//! java/util/HashSet.java
//!
//! A set whose order is part of the game's behaviour wherever it writes what it iterates:
//! `ClientboundUpdateEnabledFeaturesPacket` writes its features in its set's order. A `HashSet`
//! is a `HashMap` of its elements to Java's `PRESENT`, so it iterates like the map's keys, by its
//! table, and the port is the ported `HashMap` with `()` for `PRESENT`.
//!
//! Only what the game calls is ported: the constructors (`new HashSet<>()`, and
//! `new HashSet<>(initialCapacity)`, which codecs make their sets with), `size`, `isEmpty`,
//! `contains`, `add`, iterating it, `AbstractSet`'s `equals` and `hashCode`, and
//! `AbstractCollection`'s `toString`. Elements are never null, which Rust can't have.

/*
 * Copyright (c) 1997, 2024, Oracle and/or its affiliates. All rights reserved.
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
use std::fmt;

use crate::java::lang::object::HashCode;
use crate::java::util::hash_map::{self, HashMap};

/// A hash table of elements, iterated bin by bin.
#[derive(Clone, Debug)]
pub struct HashSet<E> {
    map: HashMap<E, ()>,
}

impl<E> HashSet<E> {
    /// `new HashSet<>()`: a map with no table until the first `add`, which makes one of 16 bins.
    pub fn new() -> HashSet<E> {
        HashSet {
            map: HashMap::new(),
        }
    }

    /// `new HashSet<>(initialCapacity)`: a map made with the capacity, whose first `add` makes a
    /// table of the smallest power of two at least the capacity. Panics on a negative capacity,
    /// where Java throws `IllegalArgumentException`.
    pub fn from_initial_capacity(initial_capacity: i32) -> HashSet<E> {
        HashSet {
            map: HashMap::from_initial_capacity(initial_capacity),
        }
    }

    pub fn size(&self) -> i32 {
        self.map.size()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// The elements in iteration order, the map's keys'.
    pub fn iter(&self) -> Iter<'_, E> {
        Iter {
            entries: self.map.iter(),
        }
    }
}

impl<E: HashCode + Eq> HashSet<E> {
    pub fn contains<Q>(&self, element: &Q) -> bool
    where
        E: Borrow<Q>,
        Q: HashCode + Eq + ?Sized,
    {
        self.map.contains_key(element)
    }

    /// Puts the element in the map, unless it has it, which leaves the one it has where it is.
    /// Returns whether it was new.
    pub fn add(&mut self, element: E) -> bool {
        self.map.put(element, ()).is_none()
    }

    /// `AbstractSet.hashCode`: the sum of its elements' hashes.
    pub fn hash_code(&self) -> i32 {
        self.iter().fold(0, |hash: i32, element| {
            hash.wrapping_add(element.hash_code())
        })
    }
}

impl<E> Default for HashSet<E> {
    fn default() -> HashSet<E> {
        HashSet::new()
    }
}

/// A set's elements in iteration order, as many as its `size()`, which a codec writes first.
pub struct Iter<'a, E> {
    entries: hash_map::Iter<'a, E, ()>,
}

impl<'a, E> Iterator for Iter<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<&'a E> {
        self.entries.next().map(|(element, _)| element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl<E> ExactSizeIterator for Iter<'_, E> {}

impl<'a, E> IntoIterator for &'a HashSet<E> {
    type Item = &'a E;
    type IntoIter = Iter<'a, E>;

    fn into_iter(self) -> Iter<'a, E> {
        self.iter()
    }
}

/// `add`s each element in turn, like a codec reading a collection (`ByteBufCodecs.collection`).
impl<E: HashCode + Eq> Extend<E> for HashSet<E> {
    fn extend<I: IntoIterator<Item = E>>(&mut self, elements: I) {
        for element in elements {
            self.add(element);
        }
    }
}

/// `AbstractSet.equals`: the same size, and every element of the other one in this one, whatever
/// their tables and order.
impl<E: HashCode + Eq> PartialEq for HashSet<E> {
    fn eq(&self, other: &HashSet<E>) -> bool {
        self.size() == other.size() && other.iter().all(|element| self.contains(element))
    }
}

/// `AbstractCollection.toString`: `[element, element]`, in iteration order.
impl<E: fmt::Display> fmt::Display for HashSet<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (index, element) in self.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{element}")?;
        }
        f.write_str("]")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::HashSet;
    use crate::golden::{Decode, Encode, elements};
    use crate::java::lang::object::HashCode;
    use crate::java::util::hash_map::HashMap;

    // [capacity, [element, …]]: its map's table length, and its elements in iteration order.
    impl<E: Decode + HashCode> Decode for HashSet<E> {
        fn decode(value: &Value) -> Self {
            let [capacity, elements] = elements(value);
            let entries = Vec::<E>::decode(elements)
                .into_iter()
                .map(|element| (element, ()))
                .collect();
            HashSet {
                map: HashMap::from_table(i32::decode(capacity), entries),
            }
        }
    }

    impl<E: Encode> Encode for HashSet<E> {
        fn encode(&self) -> Value {
            let elements: Vec<Value> = self.iter().map(Encode::encode).collect();
            Value::from(vec![
                Value::from(self.map.capacity()),
                Value::from(elements),
            ])
        }

        fn matches(&self, expected: &Value) -> bool {
            let Some([capacity, elements]) = expected.as_array().map(Vec::as_slice) else {
                return false;
            };
            let Some(elements) = elements.as_array() else {
                return false;
            };
            self.map.capacity().matches(capacity)
                && elements.len() == self.size() as usize
                && self
                    .iter()
                    .zip(elements)
                    .all(|(element, expected)| element.matches(expected))
        }
    }
}
