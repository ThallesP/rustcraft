//! net/minecraft/world/phys/shapes/SubShape.java
//!
//! Its `fill`, which would fill its parent, isn't ported: nothing fills a sub shape, and the parent
//! is shared and never changes once made.

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::{self, DiscreteVoxelShape};

/// A box of another shape's voxels, from its start (inclusive) to its end (exclusive).
pub struct SubShape {
    parent: Arc<dyn DiscreteVoxelShape>,
    start_x: i32,
    start_y: i32,
    start_z: i32,
    end_x: i32,
    end_y: i32,
    end_z: i32,
}

impl SubShape {
    pub(crate) fn new(
        parent: Arc<dyn DiscreteVoxelShape>,
        start_x: i32,
        start_y: i32,
        start_z: i32,
        end_x: i32,
        end_y: i32,
        end_z: i32,
    ) -> SubShape {
        discrete_voxel_shape::check_sizes(end_x - start_x, end_y - start_y, end_z - start_z);
        SubShape {
            parent,
            start_x,
            start_y,
            start_z,
            end_x,
            end_y,
            end_z,
        }
    }

    fn clamp_to_shape(&self, axis: Axis, parent_result: i32) -> i32 {
        let start = axis.choose_i32(self.start_x, self.start_y, self.start_z);
        let end = axis.choose_i32(self.end_x, self.end_y, self.end_z);
        Mth::clamp_i32(parent_result, start, end) - start
    }
}

impl DiscreteVoxelShape for SubShape {
    fn x_size(&self) -> i32 {
        self.end_x - self.start_x
    }

    fn y_size(&self) -> i32 {
        self.end_y - self.start_y
    }

    fn z_size(&self) -> i32 {
        self.end_z - self.start_z
    }

    fn is_full(&self, x: i32, y: i32, z: i32) -> bool {
        self.parent
            .is_full(self.start_x + x, self.start_y + y, self.start_z + z)
    }

    fn first_full(&self, axis: Axis) -> i32 {
        self.clamp_to_shape(axis, self.parent.first_full(axis))
    }

    fn last_full(&self, axis: Axis) -> i32 {
        self.clamp_to_shape(axis, self.parent.last_full(axis))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
