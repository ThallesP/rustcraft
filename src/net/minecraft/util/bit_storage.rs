//! net/minecraft/util/BitStorage.java
//!
//! `copy()` is `Clone`, which both storages are: `SimpleBitStorage`'s copies its longs like
//! Java's, and `ZeroBitStorage`'s has nothing to share. `getRaw()` hands out the array Java's
//! callers write into (`PalettedContainer.read` reads longs straight into it), so it comes as
//! `get_raw` and `get_raw_mut`. `getAll`'s `IntConsumer` is an `FnMut(i32)`.

/// Ints of a fixed number of bits, packed into longs: a palette's ids, one per block or biome.
pub trait BitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32;

    fn set(&mut self, index: i32, value: i32);

    fn fill(&mut self, value: i32);

    fn get(&self, index: i32) -> i32;

    fn get_raw(&self) -> &[i64];

    fn get_raw_mut(&mut self) -> &mut [i64];

    fn get_size(&self) -> i32;

    fn get_bits(&self) -> i32;

    fn get_all(&self, output: &mut dyn FnMut(i32));

    fn unpack(&self, output: &mut [i32]);
}

/// Apache Commons' `Validate.inclusiveBetween(long, long, long)`, which throws
/// `IllegalArgumentException`.
pub(crate) fn validate_inclusive_between(start: i64, end: i64, value: i64) {
    if value < start || value > end {
        panic!("The value {value} is not in the specified inclusive range of {start} to {end}");
    }
}
