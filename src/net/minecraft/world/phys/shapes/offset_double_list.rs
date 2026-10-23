//! net/minecraft/world/phys/shapes/OffsetDoubleList.java

use std::any::Any;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;

/// Another list's values, each plus an offset: a moved shape's coordinates.
pub struct OffsetDoubleList {
    delegate: Arc<dyn DoubleList>,
    offset: f64,
}

impl OffsetDoubleList {
    pub fn new(delegate: Arc<dyn DoubleList>, offset: f64) -> OffsetDoubleList {
        OffsetDoubleList { delegate, offset }
    }
}

impl DoubleList for OffsetDoubleList {
    fn get_double(&self, index: i32) -> f64 {
        self.delegate.get_double(index) + self.offset
    }

    fn size(&self) -> i32 {
        self.delegate.size()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
