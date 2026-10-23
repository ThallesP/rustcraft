//! net/minecraft/world/phys/shapes/IdenticalMerger.java

use std::any::Any;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;

/// Two lists that are the same: each interval is the same in both.
pub struct IdenticalMerger {
    coords: Arc<dyn DoubleList>,
}

impl IdenticalMerger {
    pub fn new(coords: Arc<dyn DoubleList>) -> IdenticalMerger {
        IdenticalMerger { coords }
    }
}

impl IndexMerger for IdenticalMerger {
    fn for_merged_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool {
        let size = self.coords.size() - 1;
        for i in 0..size {
            if !consumer(i, i, i) {
                return false;
            }
        }
        true
    }

    fn size(&self) -> i32 {
        self.coords.size()
    }

    fn get_list(&self) -> Arc<dyn DoubleList> {
        Arc::clone(&self.coords)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
