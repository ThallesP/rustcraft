//! java/util/ArrayList.java
//!
//! An `ArrayList` is a `Vec`. Only its constructor taking a capacity is here, which codecs pass to
//! make their lists (`ByteBufCodecs.collection(ArrayList::new, …)`), since it throws on a negative
//! capacity, which a length read from a buffer can be.

pub struct ArrayList;

impl ArrayList {
    /// `new ArrayList<>(initialCapacity)`. Panics on a negative capacity, where Java throws
    /// `IllegalArgumentException`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new<E>(initial_capacity: i32) -> Vec<E> {
        assert!(
            initial_capacity >= 0,
            "Illegal Capacity: {initial_capacity}"
        );
        Vec::with_capacity(initial_capacity as usize)
    }
}
