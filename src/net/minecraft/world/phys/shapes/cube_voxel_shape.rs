//! net/minecraft/world/phys/shapes/CubeVoxelShape.java

use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::shapes::cube_point_range::CubePointRange;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::DiscreteVoxelShape;
use crate::net::minecraft::world::phys::shapes::voxel_shape::{Kind, VoxelShape};

/// A shape within the unit cube whose grid splits each axis evenly: a block's shape in halves,
/// quarters or eighths.
pub struct CubeVoxelShape;

impl CubeVoxelShape {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(shape: Arc<dyn DiscreteVoxelShape>) -> VoxelShape {
        VoxelShape::from_kind(shape, Kind::Cube(CubeVoxelShape))
    }

    /// A new `CubePointRange` of the voxels along the axis each time, like Java's.
    pub(crate) fn get_coords(shape: &dyn DiscreteVoxelShape, axis: Axis) -> Arc<dyn DoubleList> {
        Arc::new(CubePointRange::new(shape.get_size(axis)))
    }

    pub(crate) fn find_index(shape: &dyn DiscreteVoxelShape, axis: Axis, coord: f64) -> i32 {
        let size = shape.get_size(axis);
        Mth::floor_f64(Mth::clamp_f64(
            coord * f64::from(size),
            -1.0,
            f64::from(size),
        ))
    }
}
