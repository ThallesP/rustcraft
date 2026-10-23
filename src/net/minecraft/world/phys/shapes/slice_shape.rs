//! net/minecraft/world/phys/shapes/SliceShape.java

use std::sync::{Arc, LazyLock};

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::phys::shapes::cube_point_range::CubePointRange;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::DiscreteVoxelShape;
use crate::net::minecraft::world::phys::shapes::sub_shape::SubShape;
use crate::net::minecraft::world::phys::shapes::voxel_shape::{Kind, VoxelShape};

static SLICE_COORDS: LazyLock<Arc<dyn DoubleList>> =
    LazyLock::new(|| Arc::new(CubePointRange::new(1)));

/// One layer of another shape's voxels across an axis, stretched over the whole block along it.
pub struct SliceShape {
    delegate: VoxelShape,
    axis: Axis,
}

impl SliceShape {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(delegate: VoxelShape, axis: Axis, point: i32) -> VoxelShape {
        let shape = SliceShape::make_slice(Arc::clone(delegate.shape()), axis, point);
        VoxelShape::from_kind(Arc::new(shape), Kind::Slice(SliceShape { delegate, axis }))
    }

    fn make_slice(delegate: Arc<dyn DiscreteVoxelShape>, axis: Axis, point: i32) -> SubShape {
        let (x_size, y_size, z_size) = (delegate.x_size(), delegate.y_size(), delegate.z_size());
        SubShape::new(
            delegate,
            axis.choose_i32(point, 0, 0),
            axis.choose_i32(0, point, 0),
            axis.choose_i32(0, 0, point),
            axis.choose_i32(point + 1, x_size, x_size),
            axis.choose_i32(y_size, point + 1, y_size),
            axis.choose_i32(z_size, z_size, point + 1),
        )
    }

    pub(crate) fn get_coords(&self, axis: Axis) -> Arc<dyn DoubleList> {
        if axis == self.axis {
            Arc::clone(&SLICE_COORDS)
        } else {
            self.delegate.get_coords(axis)
        }
    }
}
