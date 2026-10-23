//! net/minecraft/world/phys/shapes/NonOverlappingMerger.java
//!
//! Java's is its own list (`getList()` returns `this`); here `get_list` hands out a copy, which
//! holds the same lists, so it reads the same.

use std::any::Any;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;

/// Two lists, one wholly below the other: the merged list is the lower one, then the upper one.
#[derive(Clone)]
pub struct NonOverlappingMerger {
    lower: Arc<dyn DoubleList>,
    upper: Arc<dyn DoubleList>,
    swap: bool,
}

impl NonOverlappingMerger {
    pub(crate) fn new(
        lower: Arc<dyn DoubleList>,
        upper: Arc<dyn DoubleList>,
        swap: bool,
    ) -> NonOverlappingMerger {
        NonOverlappingMerger { lower, upper, swap }
    }

    fn for_non_swapped_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool {
        let lower_size = self.lower.size();
        for i in 0..lower_size {
            if !consumer(i, -1, i) {
                return false;
            }
        }

        let upper_size = self.upper.size() - 1;
        for i in 0..upper_size {
            if !consumer(lower_size - 1, i, lower_size + i) {
                return false;
            }
        }
        true
    }
}

impl DoubleList for NonOverlappingMerger {
    fn get_double(&self, index: i32) -> f64 {
        if index < self.lower.size() {
            self.lower.get_double(index)
        } else {
            self.upper.get_double(index - self.lower.size())
        }
    }

    fn size(&self) -> i32 {
        self.lower.size() + self.upper.size()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IndexMerger for NonOverlappingMerger {
    fn for_merged_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool {
        if self.swap {
            self.for_non_swapped_indexes(&mut |first_index, second_index, result_index| {
                consumer(second_index, first_index, result_index)
            })
        } else {
            self.for_non_swapped_indexes(consumer)
        }
    }

    fn size(&self) -> i32 {
        DoubleList::size(self)
    }

    fn get_list(&self) -> Arc<dyn DoubleList> {
        Arc::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
