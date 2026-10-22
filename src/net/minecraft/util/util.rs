//! net/minecraft/util/Util.java
//!
//! A grab bag of helpers, ported as the classes calling them are. So far `getRandom` and
//! `shuffledCopy` of an array, and `shuffle`, which `Direction` uses: its golden cases check
//! them, since vanilla can't generate Java's generic arrays and lists.

use crate::net::minecraft::util::random_source::RandomSource;

pub struct Util;

impl Util {
    /// Covers the array and list overloads, which do the same.
    pub fn get_random<'a, T>(array: &'a [T], random: &mut dyn RandomSource) -> &'a T {
        &array[random.next_int_bounded(array.len() as i32) as usize]
    }

    pub fn shuffled_copy<T: Clone>(array: &[T], random: &mut dyn RandomSource) -> Vec<T> {
        let mut copy = array.to_vec();
        Util::shuffle(&mut copy, random);
        copy
    }

    /// Swaps each element from the last down to the second with one at or before it.
    pub fn shuffle<T>(list: &mut [T], random: &mut dyn RandomSource) {
        let size = list.len() as i32;
        for i in (2..=size).rev() {
            let swap_to = random.next_int_bounded(i);
            list.swap((i - 1) as usize, swap_to as usize);
        }
    }
}
