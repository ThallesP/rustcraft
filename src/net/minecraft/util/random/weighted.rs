//! net/minecraft/util/random/Weighted.java
//!
//! Not ported yet: `codec` and `streamCodec`, which wait for DataFixerUpper's codecs and their
//! callers.

/// A value and how likely it's picked against the others in its list.
#[derive(Clone, Debug, PartialEq)]
pub struct Weighted<T> {
    value: T,
    weight: i32,
}

impl<T> Weighted<T> {
    /// Java throws `IllegalArgumentException` for a negative weight, where this panics.
    pub fn new(value: T, weight: i32) -> Weighted<T> {
        assert!(
            weight >= 0,
            "IllegalArgumentException: Weight should be >= 0"
        );
        Weighted { value, weight }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }

    pub fn map<U>(&self, function: impl FnOnce(&T) -> U) -> Weighted<U> {
        Weighted::new(function(&self.value), self.weight)
    }
}
