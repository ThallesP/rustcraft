//! net/minecraft/world/phys/shapes/DiscreteCubeMerger.java

use std::any::Any;
use std::sync::Arc;

use crate::com::google::common::math::int_math::IntMath;
use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::world::phys::shapes::cube_point_range::CubePointRange;
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;

/// Two cubes split in parts along an axis: the merged one is split in their least common
/// multiple.
pub struct DiscreteCubeMerger {
    result: CubePointRange,
    first_div: i32,
    second_div: i32,
}

impl DiscreteCubeMerger {
    pub(crate) fn new(first_size: i32, second_size: i32) -> DiscreteCubeMerger {
        let result = CubePointRange::new(Shapes::lcm(first_size, second_size) as i32);
        let gcd = IntMath::gcd(first_size, second_size);
        DiscreteCubeMerger {
            result,
            first_div: first_size / gcd,
            second_div: second_size / gcd,
        }
    }
}

impl IndexMerger for DiscreteCubeMerger {
    fn for_merged_indexes(&self, consumer: &mut dyn FnMut(i32, i32, i32) -> bool) -> bool {
        let size = self.result.size() - 1;
        for i in 0..size {
            if !consumer(i / self.second_div, i / self.first_div, i) {
                return false;
            }
        }
        true
    }

    fn size(&self) -> i32 {
        self.result.size()
    }

    fn get_list(&self) -> Arc<dyn DoubleList> {
        Arc::new(self.result)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
