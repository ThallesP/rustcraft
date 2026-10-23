//! com/mojang/serialization/MapLike.java (DataFixerUpper 10.0.21)
//!
//! A map's entries as a codec reads them, which `DynamicOps.getMap` hands out over its input.
//! A value it hands out borrows the map, and `entries()` collects Java's stream.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::dynamic_ops::OpsValue;

pub trait MapLike<T>: fmt::Display {
    fn get(&self, key: &T) -> Option<&T>;

    /// `get(String)`.
    fn get_str(&self, key: &str) -> Option<&T>;

    fn entries(&self) -> Vec<Pair<T, T>>;
}

/// `MapLike.EMPTY`.
struct Empty;

impl<T> MapLike<T> for Empty {
    fn get(&self, _key: &T) -> Option<&T> {
        None
    }

    fn get_str(&self, _key: &str) -> Option<&T> {
        None
    }

    fn entries(&self) -> Vec<Pair<T, T>> {
        Vec::new()
    }
}

impl fmt::Display for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EmptyMapLike")
    }
}

pub fn empty<'a, T: 'a>() -> Box<dyn MapLike<T> + 'a> {
    Box::new(Empty)
}

/// `forMap`: the map's entries, in its order, looked up by `==`; `get(String)` looks up the key
/// `createString` makes. An empty map is `empty()`.
pub fn for_map<'a, T: OpsValue>(
    map: Vec<Pair<T, T>>,
    create_string: Box<dyn Fn(&str) -> T + 'a>,
) -> Box<dyn MapLike<T> + 'a> {
    if map.is_empty() {
        return empty();
    }
    Box::new(ForMap { map, create_string })
}

struct ForMap<'a, T> {
    map: Vec<Pair<T, T>>,
    create_string: Box<dyn Fn(&str) -> T + 'a>,
}

impl<T: OpsValue> MapLike<T> for ForMap<'_, T> {
    fn get(&self, key: &T) -> Option<&T> {
        self.map
            .iter()
            .find(|entry| entry.get_first() == key)
            .map(Pair::get_second)
    }

    fn get_str(&self, key: &str) -> Option<&T> {
        self.get(&(self.create_string)(key))
    }

    fn entries(&self) -> Vec<Pair<T, T>> {
        self.map.clone()
    }
}

/// `MapLike[{key=value, …}]`, `AbstractMap.toString` of the map.
impl<T: OpsValue> fmt::Display for ForMap<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapLike[{")?;
        for (index, entry) in self.map.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}={}", entry.get_first(), entry.get_second())?;
        }
        f.write_str("}]")
    }
}
