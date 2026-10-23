//! net/minecraft/world/phys/shapes/IndirectMerger.java

use std::any::Any;
use std::sync::{Arc, LazyLock};

use crate::it::unimi::dsi::fastutil::doubles::double_array_list::DoubleArrayList;
use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;

/// `DoubleLists.unmodifiable(DoubleArrayList.wrap(new double[]{0.0}))`.
static EMPTY: LazyLock<Arc<dyn DoubleList>> =
    LazyLock::new(|| Arc::new(DoubleArrayList::from_slice(&[0.0])));

/// Two overlapping lists, merged in order, with points closer than 1.0E-7 taken as one.
pub struct IndirectMerger {
    result: Arc<[f64]>,
    first_indices: Vec<i32>,
    second_indices: Vec<i32>,
    result_length: i32,
}

impl IndirectMerger {
    pub fn new(
        first: &dyn DoubleList,
        second: &dyn DoubleList,
        first_only_matters: bool,
        second_only_matters: bool,
    ) -> IndirectMerger {
        let mut last_value = f64::NAN;
        let first_size = first.size();
        let second_size = second.size();
        let capacity = (first_size + second_size) as usize;
        let mut result = vec![0.0; capacity];
        let mut first_indices = vec![0; capacity];
        let mut second_indices = vec![0; capacity];
        let can_skip_first = !first_only_matters;
        let can_skip_second = !second_only_matters;
        let mut result_index = 0;
        let mut first_index = 0;
        let mut second_index = 0;

        loop {
            let ran_out_of_first = first_index >= first_size;
            let ran_out_of_second = second_index >= second_size;
            if ran_out_of_first && ran_out_of_second {
                return IndirectMerger {
                    result: Arc::from(result),
                    first_indices,
                    second_indices,
                    result_length: 1.max(result_index as i32),
                };
            }

            let chose_first = !ran_out_of_first
                && (ran_out_of_second
                    || first.get_double(first_index) < second.get_double(second_index) + 1.0E-7);
            if chose_first {
                first_index += 1;
                if can_skip_first && (second_index == 0 || ran_out_of_second) {
                    continue;
                }
            } else {
                second_index += 1;
                if can_skip_second && (first_index == 0 || ran_out_of_first) {
                    continue;
                }
            }

            let current_first_index = first_index - 1;
            let current_second_index = second_index - 1;
            let next_value = if chose_first {
                first.get_double(current_first_index)
            } else {
                second.get_double(current_second_index)
            };
            // Negated like Java's, so the first value, after NaN, is always taken.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(last_value >= next_value - 1.0E-7) {
                first_indices[result_index] = current_first_index;
                second_indices[result_index] = current_second_index;
                result[result_index] = next_value;
                result_index += 1;
                last_value = next_value;
            } else {
                first_indices[result_index - 1] = current_first_index;
                second_indices[result_index - 1] = current_second_index;
            }
        }
    }
}

impl IndexMerger for IndirectMerger {
    fn for_merged_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool {
        let length = self.result_length - 1;
        for i in 0..length {
            if !consumer(
                self.first_indices[i as usize],
                self.second_indices[i as usize],
                i,
            ) {
                return false;
            }
        }
        true
    }

    fn size(&self) -> i32 {
        self.result_length
    }

    fn get_list(&self) -> Arc<dyn DoubleList> {
        if self.result_length <= 1 {
            Arc::clone(&EMPTY)
        } else {
            Arc::new(DoubleArrayList::wrap_length(
                Arc::clone(&self.result),
                self.result_length,
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
