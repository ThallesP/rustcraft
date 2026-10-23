//! net/minecraft/world/phys/shapes/BitSetDiscreteVoxelShape.java
//!
//! A shape is filled while it's made and never changes after, so the filling methods take
//! `&mut self` and a made shape is shared (`Arc<dyn DiscreteVoxelShape>`).

use std::any::Any;

use crate::java::util::bit_set::BitSet;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::{self, DiscreteVoxelShape};
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;

/// Voxels as bits, x outermost and z innermost, with the bounds of the full ones kept as they're
/// filled.
#[derive(Clone, Debug)]
pub struct BitSetDiscreteVoxelShape {
    x_size: i32,
    y_size: i32,
    z_size: i32,
    storage: BitSet,
    x_min: i32,
    y_min: i32,
    z_min: i32,
    x_max: i32,
    y_max: i32,
    z_max: i32,
}

impl BitSetDiscreteVoxelShape {
    /// An empty grid, whose bounds start inverted: each minimum at its size, each maximum at 0.
    pub fn new(x_size: i32, y_size: i32, z_size: i32) -> BitSetDiscreteVoxelShape {
        discrete_voxel_shape::check_sizes(x_size, y_size, z_size);
        BitSetDiscreteVoxelShape {
            x_size,
            y_size,
            z_size,
            storage: BitSet::new(x_size.wrapping_mul(y_size).wrapping_mul(z_size)),
            x_min: x_size,
            y_min: y_size,
            z_min: z_size,
            x_max: 0,
            y_max: 0,
            z_max: 0,
        }
    }

    /// A grid with the box between the bounds filled, whose bounds are the ones given.
    #[allow(clippy::too_many_arguments)]
    pub fn with_filled_bounds(
        x_size: i32,
        y_size: i32,
        z_size: i32,
        x_min: i32,
        y_min: i32,
        z_min: i32,
        x_max: i32,
        y_max: i32,
        z_max: i32,
    ) -> BitSetDiscreteVoxelShape {
        let mut shape = BitSetDiscreteVoxelShape::new(x_size, y_size, z_size);
        shape.x_min = x_min;
        shape.y_min = y_min;
        shape.z_min = z_min;
        shape.x_max = x_max;
        shape.y_max = y_max;
        shape.z_max = z_max;

        for x in x_min..x_max {
            for y in y_min..y_max {
                for z in z_min..z_max {
                    shape.fill_update_bounds(x, y, z, false);
                }
            }
        }
        shape
    }

    /// `new BitSetDiscreteVoxelShape(DiscreteVoxelShape)`: a copy of any shape, its bits copied
    /// from another `BitSetDiscreteVoxelShape` or read voxel by voxel, and its bounds the shape's.
    pub fn from_shape(voxel_shape: &dyn DiscreteVoxelShape) -> BitSetDiscreteVoxelShape {
        let x_size = voxel_shape.x_size();
        let y_size = voxel_shape.y_size();
        let z_size = voxel_shape.z_size();
        let storage = match voxel_shape
            .as_any()
            .downcast_ref::<BitSetDiscreteVoxelShape>()
        {
            Some(bit_set_discrete_voxel_shape) => bit_set_discrete_voxel_shape.storage.clone(),
            None => {
                let mut storage = BitSet::new(x_size.wrapping_mul(y_size).wrapping_mul(z_size));
                for x in 0..x_size {
                    for y in 0..y_size {
                        for z in 0..z_size {
                            if voxel_shape.is_full(x, y, z) {
                                storage.set((x * y_size + y) * z_size + z);
                            }
                        }
                    }
                }
                storage
            }
        };

        BitSetDiscreteVoxelShape {
            x_size,
            y_size,
            z_size,
            storage,
            x_min: voxel_shape.first_full(Axis::X),
            y_min: voxel_shape.first_full(Axis::Y),
            z_min: voxel_shape.first_full(Axis::Z),
            x_max: voxel_shape.last_full(Axis::X),
            y_max: voxel_shape.last_full(Axis::Y),
            z_max: voxel_shape.last_full(Axis::Z),
        }
    }

    fn get_index(&self, x: i32, y: i32, z: i32) -> i32 {
        (x * self.y_size + y) * self.z_size + z
    }

    fn fill_update_bounds(&mut self, x: i32, y: i32, z: i32, update_bounds: bool) {
        let index = self.get_index(x, y, z);
        self.storage.set(index);
        if update_bounds {
            self.x_min = self.x_min.min(x);
            self.y_min = self.y_min.min(y);
            self.z_min = self.z_min.min(z);
            self.x_max = self.x_max.max(x + 1);
            self.y_max = self.y_max.max(y + 1);
            self.z_max = self.z_max.max(z + 1);
        }
    }

    pub fn fill(&mut self, x: i32, y: i32, z: i32) {
        self.fill_update_bounds(x, y, z, true);
    }

    /// The voxels where `op` holds of the two shapes' voxels, over the merged grid the mergers
    /// make. Its bounds are those of the full voxels, `Integer.MAX_VALUE` and
    /// `Integer.MIN_VALUE + 1` when there are none.
    pub fn join(
        first: &dyn DiscreteVoxelShape,
        second: &dyn DiscreteVoxelShape,
        x_merger: &dyn IndexMerger,
        y_merger: &dyn IndexMerger,
        z_merger: &dyn IndexMerger,
        op: BooleanOp,
    ) -> BitSetDiscreteVoxelShape {
        let mut shape = BitSetDiscreteVoxelShape::new(
            x_merger.size() - 1,
            y_merger.size() - 1,
            z_merger.size() - 1,
        );
        let mut bounds = [i32::MAX, i32::MAX, i32::MAX, i32::MIN, i32::MIN, i32::MIN];
        x_merger.for_merged_indexes(&mut |x1, x2, xr| {
            let mut updated_slice = false;
            y_merger.for_merged_indexes(&mut |y1, y2, yr| {
                let mut updated_column = false;
                z_merger.for_merged_indexes(&mut |z1, z2, zr| {
                    if op.apply(
                        first.is_full_wide(x1, y1, z1),
                        second.is_full_wide(x2, y2, z2),
                    ) {
                        let index = shape.get_index(xr, yr, zr);
                        shape.storage.set(index);
                        bounds[2] = bounds[2].min(zr);
                        bounds[5] = bounds[5].max(zr);
                        updated_column = true;
                    }
                    true
                });
                if updated_column {
                    bounds[1] = bounds[1].min(yr);
                    bounds[4] = bounds[4].max(yr);
                    updated_slice = true;
                }
                true
            });
            if updated_slice {
                bounds[0] = bounds[0].min(xr);
                bounds[3] = bounds[3].max(xr);
            }
            true
        });
        shape.x_min = bounds[0];
        shape.y_min = bounds[1];
        shape.z_min = bounds[2];
        shape.x_max = bounds[3].wrapping_add(1);
        shape.y_max = bounds[4].wrapping_add(1);
        shape.z_max = bounds[5].wrapping_add(1);
        shape
    }

    /// Calls `consumer` with boxes that make up `voxel_shape`: each run of full voxels along z,
    /// grown along x and then y as far as the voxels it would take in are all full and not taken
    /// yet (a copy of the shape is cleared as it goes).
    pub(crate) fn for_all_boxes(
        voxel_shape: &dyn DiscreteVoxelShape,
        consumer: &mut dyn FnMut(i32, i32, i32, i32, i32, i32),
        merge_neighbors: bool,
    ) {
        let mut shape = BitSetDiscreteVoxelShape::from_shape(voxel_shape);

        for y in 0..shape.y_size {
            for x in 0..shape.x_size {
                let mut last_start_z = -1;

                for z in 0..=shape.z_size {
                    if shape.is_full_wide(x, y, z) {
                        if merge_neighbors {
                            if last_start_z == -1 {
                                last_start_z = z;
                            }
                        } else {
                            consumer(x, y, z, x + 1, y + 1, z + 1);
                        }
                    } else if last_start_z != -1 {
                        let mut end_x = x;
                        let mut end_y = y;
                        shape.clear_z_strip(last_start_z, z, x, y);

                        while shape.is_z_strip_full(last_start_z, z, end_x + 1, y) {
                            shape.clear_z_strip(last_start_z, z, end_x + 1, y);
                            end_x += 1;
                        }

                        while shape.is_xz_rectangle_full(x, end_x + 1, last_start_z, z, end_y + 1) {
                            for cx in x..=end_x {
                                shape.clear_z_strip(last_start_z, z, cx, end_y + 1);
                            }

                            end_y += 1;
                        }

                        consumer(x, y, last_start_z, end_x + 1, end_y + 1, z);
                        last_start_z = -1;
                    }
                }
            }
        }
    }

    fn is_z_strip_full(&self, start_z: i32, end_z: i32, x: i32, y: i32) -> bool {
        if x < self.x_size && y < self.y_size {
            self.storage.next_clear_bit(self.get_index(x, y, start_z))
                >= self.get_index(x, y, end_z)
        } else {
            false
        }
    }

    fn is_xz_rectangle_full(
        &self,
        start_x: i32,
        end_x: i32,
        start_z: i32,
        end_z: i32,
        y: i32,
    ) -> bool {
        (start_x..end_x).all(|x| self.is_z_strip_full(start_z, end_z, x, y))
    }

    fn clear_z_strip(&mut self, start_z: i32, end_z: i32, x: i32, y: i32) {
        let from = self.get_index(x, y, start_z);
        let to = self.get_index(x, y, end_z);
        self.storage.clear_range(from, to);
    }

    /// Whether the voxel is full, inside the grid, and its six neighbours are full too.
    pub fn is_interior(&self, x: i32, y: i32, z: i32) -> bool {
        let is_interior = x > 0
            && x < self.x_size - 1
            && y > 0
            && y < self.y_size - 1
            && z > 0
            && z < self.z_size - 1;
        is_interior
            && self.is_full(x, y, z)
            && self.is_full(x - 1, y, z)
            && self.is_full(x + 1, y, z)
            && self.is_full(x, y - 1, z)
            && self.is_full(x, y + 1, z)
            && self.is_full(x, y, z - 1)
            && self.is_full(x, y, z + 1)
    }
}

impl DiscreteVoxelShape for BitSetDiscreteVoxelShape {
    fn x_size(&self) -> i32 {
        self.x_size
    }

    fn y_size(&self) -> i32 {
        self.y_size
    }

    fn z_size(&self) -> i32 {
        self.z_size
    }

    fn is_full(&self, x: i32, y: i32, z: i32) -> bool {
        self.storage.get(self.get_index(x, y, z))
    }

    fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    fn first_full(&self, axis: Axis) -> i32 {
        axis.choose_i32(self.x_min, self.y_min, self.z_min)
    }

    fn last_full(&self, axis: Axis) -> i32 {
        axis.choose_i32(self.x_max, self.y_max, self.z_max)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
