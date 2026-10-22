//! net/minecraft/core/IdMap.java
//!
//! Java's extends `Iterable<T>`, which no caller of this trait iterates yet: each map that's
//! iterated has its own `iter` (`IdMapper::iter`). Things are handed in by reference and out by
//! value, a copy of Java's shared reference (a `BlockState`, a biome's `Holder`).

/// Things numbered by id, both ways: the block states' registry, a synchronized registry's
/// holders, a hash map palette's values.
pub trait IdMap<T> {
    /// The thing's id, or `DEFAULT` (-1).
    fn get_id(&self, thing: &T) -> i32;

    fn by_id(&self, id: i32) -> Option<T>;

    /// Panics for an id it doesn't have, where Java throws `IllegalArgumentException`.
    fn by_id_or_throw(&self, id: i32) -> T {
        self.by_id(id)
            .unwrap_or_else(|| panic!("No value with id {id}"))
    }

    /// Panics for a thing it doesn't have, where Java throws `IllegalArgumentException`.
    fn get_id_or_throw(&self, value: &T) -> i32 {
        let id = self.get_id(value);
        if id == DEFAULT {
            panic!("Can't find id for a value in the map");
        }
        id
    }

    fn size(&self) -> i32;
}

/// `IdMap.DEFAULT`: what `get_id` answers for a thing it doesn't have.
pub const DEFAULT: i32 = -1;

/// A borrowed map is a map, so a static one (`Block.BLOCK_STATE_REGISTRY`) can be shared like an
/// owned one.
impl<T, M: IdMap<T> + ?Sized> IdMap<T> for &M {
    fn get_id(&self, thing: &T) -> i32 {
        (**self).get_id(thing)
    }

    fn by_id(&self, id: i32) -> Option<T> {
        (**self).by_id(id)
    }

    fn size(&self) -> i32 {
        (**self).size()
    }
}
