//! net/minecraft/world/level/levelgen/BitRandomSource.java
//!
//! In Java this interface fills in RandomSource's methods with defaults built on `next(bits)`.
//! A Rust trait can't provide another trait's methods, so they're functions here and each
//! implementor's `impl RandomSource` forwards to them.

use crate::net::minecraft::util::random_source::RandomSource;

pub const FLOAT_MULTIPLIER: f32 = 5.9604645E-8;
/// 2^-53. Vineflower prints it as `1.110223E-16F`, which reads as float math in `long * 1.110223E-16F`,
/// but the bytecode multiplies as doubles.
pub const DOUBLE_MULTIPLIER: f64 = 1.110223E-16_f32 as f64;

pub trait BitRandomSource: RandomSource {
    fn next(&mut self, bits: i32) -> i32;
}

pub fn next_int(source: &mut (impl BitRandomSource + ?Sized)) -> i32 {
    source.next(32)
}

pub fn next_int_bounded(source: &mut (impl BitRandomSource + ?Sized), bound: i32) -> i32 {
    assert!(bound > 0, "Bound must be positive");

    if bound & (bound - 1) == 0 {
        return ((i64::from(bound) * i64::from(source.next(31))) >> 31) as i32;
    }

    loop {
        let sample = source.next(31);
        let modulo = sample % bound;
        if sample.wrapping_sub(modulo).wrapping_add(bound - 1) >= 0 {
            return modulo;
        }
    }
}

pub fn next_long(source: &mut (impl BitRandomSource + ?Sized)) -> i64 {
    let upper = source.next(32);
    let lower = source.next(32);
    let shifted = i64::from(upper) << 32;
    shifted.wrapping_add(i64::from(lower))
}

pub fn next_boolean(source: &mut (impl BitRandomSource + ?Sized)) -> bool {
    source.next(1) != 0
}

pub fn next_float(source: &mut (impl BitRandomSource + ?Sized)) -> f32 {
    source.next(24) as f32 * FLOAT_MULTIPLIER
}

pub fn next_double(source: &mut (impl BitRandomSource + ?Sized)) -> f64 {
    let upper = source.next(26);
    let lower = source.next(27);
    let combined = (i64::from(upper) << 27) + i64::from(lower);
    combined as f64 * DOUBLE_MULTIPLIER
}
