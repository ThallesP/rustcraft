//! net/minecraft/util/random/WeightedList.java
//!
//! Not ported yet: the codecs, which wait for DataFixerUpper's, and `equals`/`hashCode`.
//!
//! Java picks an entry through a `Flat` table of every weight's slot when the total is under 64,
//! and by walking the entries (`Compact`) otherwise. Both hand out the entry whose weights'
//! running sum first passes the drawn number, so the port walks the entries for both.

use crate::net::minecraft::util::random::weighted::Weighted;
use crate::net::minecraft::util::random_source::RandomSource;

#[derive(Clone, Debug)]
pub struct WeightedList<E> {
    total_weight: i32,
    items: Vec<Weighted<E>>,
}

impl<E> WeightedList<E> {
    /// Java throws `IllegalArgumentException` when the weights add up past an int.
    pub fn of(items: Vec<Weighted<E>>) -> WeightedList<E> {
        let total: i64 = items.iter().map(|item| i64::from(item.weight())).sum();
        assert!(
            total <= i64::from(i32::MAX),
            "IllegalArgumentException: Sum of weights must be <= 2147483647"
        );
        WeightedList {
            total_weight: total as i32,
            items,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_weight == 0
    }

    pub fn map<T>(&self, mut mapper: impl FnMut(&E) -> T) -> WeightedList<T> {
        WeightedList::of(
            self.items
                .iter()
                .map(|item| item.map(&mut mapper))
                .collect(),
        )
    }

    /// Java throws `IllegalStateException` for a list without weight.
    pub fn get_random_or_throw(&self, random: &mut dyn RandomSource) -> &E {
        assert!(
            self.total_weight != 0,
            "IllegalStateException: Weighted list has no elements"
        );
        let mut selection = random.next_int_bounded(self.total_weight);
        for entry in &self.items {
            selection -= entry.weight();
            if selection < 0 {
                return entry.value();
            }
        }
        panic!("IllegalStateException: {selection} exceeded total weight")
    }

    pub fn unwrap(&self) -> &[Weighted<E>] {
        &self.items
    }
}
