//! net/minecraft/world/phys/shapes/CubePointRange.java

use std::any::Any;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;

/// The points 0, 1/parts, 2/parts… 1: a cube's coordinates along an axis it splits in `parts`.
#[derive(Clone, Copy, Debug)]
pub struct CubePointRange {
    parts: i32,
}

impl CubePointRange {
    /// Throws `IllegalArgumentException` for fewer than 1 part.
    pub fn new(parts: i32) -> CubePointRange {
        assert!(parts > 0, "Need at least 1 part");
        CubePointRange { parts }
    }
}

impl DoubleList for CubePointRange {
    fn get_double(&self, index: i32) -> f64 {
        f64::from(index) / f64::from(self.parts)
    }

    fn size(&self) -> i32 {
        self.parts + 1
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
