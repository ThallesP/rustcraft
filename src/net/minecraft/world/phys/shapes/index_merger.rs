//! net/minecraft/world/phys/shapes/IndexMerger.java
//!
//! `IndexConsumer`, a functional interface, is a closure (PORTING.md), and `instanceof` is
//! `as_any`.

use std::any::Any;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;

/// Merges two shapes' coordinates along an axis: the merged list, and for each of its intervals,
/// the interval of each shape it lies in.
pub trait IndexMerger {
    fn get_list(&self) -> Arc<dyn DoubleList>;

    /// Calls `consumer` with each merged interval's index in the first list, in the second and in
    /// the merged one, until it returns false; returns whether it never did.
    fn for_merged_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool;

    fn size(&self) -> i32;

    fn as_any(&self) -> &dyn Any;
}
