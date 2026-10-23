//! net/minecraft/world/phys/shapes/ArrayVoxelShape.java

use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_array_list::DoubleArrayList;
use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::DiscreteVoxelShape;
use crate::net::minecraft::world::phys::shapes::voxel_shape::{Kind, VoxelShape};

/// A shape whose grid lines are given along each axis: one more than its voxels.
pub struct ArrayVoxelShape {
    xs: Arc<dyn DoubleList>,
    ys: Arc<dyn DoubleList>,
    zs: Arc<dyn DoubleList>,
}

impl ArrayVoxelShape {
    /// `new ArrayVoxelShape(DiscreteVoxelShape, double[], double[], double[])`: copies of the
    /// arrays, cut or padded with zeros to one more than the voxels.
    pub(crate) fn from_arrays(
        shape: Arc<dyn DiscreteVoxelShape>,
        xs: &[f64],
        ys: &[f64],
        zs: &[f64],
    ) -> VoxelShape {
        let copy_of = |array: &[f64], length: i32| -> Arc<dyn DoubleList> {
            let mut copy = vec![0.0; length as usize];
            let common = array.len().min(copy.len());
            copy[..common].copy_from_slice(&array[..common]);
            Arc::new(DoubleArrayList::wrap(Arc::from(copy)))
        };
        let x_size = shape.get_x_size() + 1;
        let y_size = shape.get_y_size() + 1;
        let z_size = shape.get_z_size() + 1;
        ArrayVoxelShape::new(
            shape,
            copy_of(xs, x_size),
            copy_of(ys, y_size),
            copy_of(zs, z_size),
        )
    }

    /// Throws `IllegalArgumentException` when a list's length isn't one more than the voxels along
    /// its axis.
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(
        shape: Arc<dyn DiscreteVoxelShape>,
        xs: Arc<dyn DoubleList>,
        ys: Arc<dyn DoubleList>,
        zs: Arc<dyn DoubleList>,
    ) -> VoxelShape {
        let x_size = shape.get_x_size() + 1;
        let y_size = shape.get_y_size() + 1;
        let z_size = shape.get_z_size() + 1;
        assert!(
            x_size == xs.size() && y_size == ys.size() && z_size == zs.size(),
            "Lengths of point arrays must be consistent with the size of the VoxelShape."
        );
        VoxelShape::from_kind(shape, Kind::Array(ArrayVoxelShape { xs, ys, zs }))
    }

    pub(crate) fn get_coords(&self, axis: Axis) -> Arc<dyn DoubleList> {
        Arc::clone(match axis {
            Axis::X => &self.xs,
            Axis::Y => &self.ys,
            Axis::Z => &self.zs,
        })
    }
}
