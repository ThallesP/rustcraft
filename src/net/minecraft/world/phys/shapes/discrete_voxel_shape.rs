//! net/minecraft/world/phys/shapes/DiscreteVoxelShape.java
//!
//! An abstract class with two subclasses, `BitSetDiscreteVoxelShape` and `SubShape`, so it's a
//! trait whose provided methods are the abstract class's own, and `instanceof` is `as_any`. Its
//! `fill`, which only a `BitSetDiscreteVoxelShape` gets called with (a `SubShape` would fill its
//! parent, which nothing does), is that class's own. `IntLineConsumer` and `IntFaceConsumer` are
//! closures. `rotate`, which hands back the shape itself for the identity, is a function of the
//! shared shape (`rotate`).

use std::any::Any;
use std::sync::Arc;

use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::org::joml::vector3i::Vector3i;

use crate::net::minecraft::core::axis_cycle::AxisCycle;
use crate::net::minecraft::core::direction::{Axis, AxisDirection, Direction};
use crate::net::minecraft::world::phys::shapes::bit_set_discrete_voxel_shape::BitSetDiscreteVoxelShape;

/// A grid of voxels, each full or not, with the bounds of the full ones.
pub trait DiscreteVoxelShape: Any + Send + Sync {
    fn x_size(&self) -> i32;

    fn y_size(&self) -> i32;

    fn z_size(&self) -> i32;

    fn is_full(&self, x: i32, y: i32, z: i32) -> bool;

    fn first_full(&self, axis: Axis) -> i32;

    fn last_full(&self, axis: Axis) -> i32;

    fn as_any(&self) -> &dyn Any;

    /// `isFullWide(AxisCycle, int, int, int)`.
    fn is_full_wide_cycled(&self, transform: AxisCycle, x: i32, y: i32, z: i32) -> bool {
        self.is_full_wide(
            transform.cycle_i32(x, y, z, Axis::X),
            transform.cycle_i32(x, y, z, Axis::Y),
            transform.cycle_i32(x, y, z, Axis::Z),
        )
    }

    /// Whether the voxel is full, false outside the grid.
    fn is_full_wide(&self, x: i32, y: i32, z: i32) -> bool {
        if x < 0 || y < 0 || z < 0 {
            false
        } else if x < self.x_size() && y < self.y_size() && z < self.z_size() {
            self.is_full(x, y, z)
        } else {
            false
        }
    }

    /// `isFull(AxisCycle, int, int, int)`.
    fn is_full_cycled(&self, transform: AxisCycle, x: i32, y: i32, z: i32) -> bool {
        self.is_full(
            transform.cycle_i32(x, y, z, Axis::X),
            transform.cycle_i32(x, y, z, Axis::Y),
            transform.cycle_i32(x, y, z, Axis::Z),
        )
    }

    fn is_empty(&self) -> bool {
        Axis::VALUES
            .into_iter()
            .any(|axis| self.first_full(axis) >= self.last_full(axis))
    }

    /// `firstFull(Axis, int, int)`: the first full voxel along `a_axis` in the row at `b` and `c`
    /// along the next axes, or the size when there's none.
    fn first_full_at(&self, a_axis: Axis, b: i32, c: i32) -> i32 {
        let a_size = self.get_size(a_axis);
        if b >= 0 && c >= 0 {
            let b_axis = AxisCycle::FORWARD.cycle_axis(a_axis);
            let c_axis = AxisCycle::BACKWARD.cycle_axis(a_axis);
            if b < self.get_size(b_axis) && c < self.get_size(c_axis) {
                let transform = AxisCycle::between(Axis::X, a_axis);
                for a in 0..a_size {
                    if self.is_full_cycled(transform, a, b, c) {
                        return a;
                    }
                }
                a_size
            } else {
                a_size
            }
        } else {
            a_size
        }
    }

    /// `lastFull(Axis, int, int)`: one past the last full voxel in the row, or 0.
    fn last_full_at(&self, a_axis: Axis, b: i32, c: i32) -> i32 {
        if b >= 0 && c >= 0 {
            let b_axis = AxisCycle::FORWARD.cycle_axis(a_axis);
            let c_axis = AxisCycle::BACKWARD.cycle_axis(a_axis);
            if b < self.get_size(b_axis) && c < self.get_size(c_axis) {
                let a_size = self.get_size(a_axis);
                let transform = AxisCycle::between(Axis::X, a_axis);
                for a in (0..a_size).rev() {
                    if self.is_full_cycled(transform, a, b, c) {
                        return a + 1;
                    }
                }
                0
            } else {
                0
            }
        } else {
            0
        }
    }

    fn get_size(&self, axis: Axis) -> i32 {
        axis.choose_i32(self.x_size(), self.y_size(), self.z_size())
    }

    fn get_x_size(&self) -> i32 {
        self.get_size(Axis::X)
    }

    fn get_y_size(&self) -> i32 {
        self.get_size(Axis::Y)
    }

    fn get_z_size(&self) -> i32 {
        self.get_size(Axis::Z)
    }

    /// Calls `consumer` with each edge of the shape, from one corner's indexes to the other's,
    /// along x, then y, then z; with `merge_neighbors`, edges in a line are one.
    fn for_all_edges(
        &self,
        consumer: &mut dyn FnMut(i32, i32, i32, i32, i32, i32),
        merge_neighbors: bool,
    ) {
        self.for_all_axis_edges(consumer, AxisCycle::NONE, merge_neighbors);
        self.for_all_axis_edges(consumer, AxisCycle::FORWARD, merge_neighbors);
        self.for_all_axis_edges(consumer, AxisCycle::BACKWARD, merge_neighbors);
    }

    #[doc(hidden)]
    fn for_all_axis_edges(
        &self,
        consumer: &mut dyn FnMut(i32, i32, i32, i32, i32, i32),
        transform: AxisCycle,
        merge_neighbors: bool,
    ) {
        let inverse = transform.inverse();
        let a_size = self.get_size(inverse.cycle_axis(Axis::X));
        let b_size = self.get_size(inverse.cycle_axis(Axis::Y));
        let c_size = self.get_size(inverse.cycle_axis(Axis::Z));

        for a in 0..=a_size {
            for b in 0..=b_size {
                let mut last_start = -1;

                for c in 0..=c_size {
                    let mut full_sectors = 0;
                    let mut odd_sectors = 0;

                    for da in 0..=1 {
                        for db in 0..=1 {
                            if self.is_full_wide_cycled(inverse, a + da - 1, b + db - 1, c) {
                                full_sectors += 1;
                                odd_sectors ^= da ^ db;
                            }
                        }
                    }

                    if full_sectors == 1
                        || full_sectors == 3
                        || full_sectors == 2 && (odd_sectors & 1) == 0
                    {
                        if merge_neighbors {
                            if last_start == -1 {
                                last_start = c;
                            }
                        } else {
                            consumer(
                                inverse.cycle_i32(a, b, c, Axis::X),
                                inverse.cycle_i32(a, b, c, Axis::Y),
                                inverse.cycle_i32(a, b, c, Axis::Z),
                                inverse.cycle_i32(a, b, c + 1, Axis::X),
                                inverse.cycle_i32(a, b, c + 1, Axis::Y),
                                inverse.cycle_i32(a, b, c + 1, Axis::Z),
                            );
                        }
                    } else if last_start != -1 {
                        consumer(
                            inverse.cycle_i32(a, b, last_start, Axis::X),
                            inverse.cycle_i32(a, b, last_start, Axis::Y),
                            inverse.cycle_i32(a, b, last_start, Axis::Z),
                            inverse.cycle_i32(a, b, c, Axis::X),
                            inverse.cycle_i32(a, b, c, Axis::Y),
                            inverse.cycle_i32(a, b, c, Axis::Z),
                        );
                        last_start = -1;
                    }
                }
            }
        }
    }

    /// Calls `consumer` with each face of each full voxel that doesn't touch another, and the
    /// direction it faces.
    fn for_all_faces(&self, consumer: &mut dyn FnMut(Direction, i32, i32, i32)) {
        self.for_all_axis_faces(consumer, AxisCycle::NONE);
        self.for_all_axis_faces(consumer, AxisCycle::FORWARD);
        self.for_all_axis_faces(consumer, AxisCycle::BACKWARD);
    }

    #[doc(hidden)]
    fn for_all_axis_faces(
        &self,
        consumer: &mut dyn FnMut(Direction, i32, i32, i32),
        transform: AxisCycle,
    ) {
        let inverse = transform.inverse();
        let c_axis = inverse.cycle_axis(Axis::Z);
        let a_size = self.get_size(inverse.cycle_axis(Axis::X));
        let b_size = self.get_size(inverse.cycle_axis(Axis::Y));
        let c_size = self.get_size(c_axis);
        let negative = Direction::from_axis_and_direction(c_axis, AxisDirection::NEGATIVE);
        let positive = Direction::from_axis_and_direction(c_axis, AxisDirection::POSITIVE);

        for a in 0..a_size {
            for b in 0..b_size {
                let mut last_full = false;

                for c in 0..=c_size {
                    let full = c != c_size && self.is_full_cycled(inverse, a, b, c);
                    if !last_full && full {
                        consumer(
                            negative,
                            inverse.cycle_i32(a, b, c, Axis::X),
                            inverse.cycle_i32(a, b, c, Axis::Y),
                            inverse.cycle_i32(a, b, c, Axis::Z),
                        );
                    }

                    if last_full && !full {
                        consumer(
                            positive,
                            inverse.cycle_i32(a, b, c - 1, Axis::X),
                            inverse.cycle_i32(a, b, c - 1, Axis::Y),
                            inverse.cycle_i32(a, b, c - 1, Axis::Z),
                        );
                    }

                    last_full = full;
                }
            }
        }
    }
}

impl dyn DiscreteVoxelShape {
    /// Calls `consumer` with boxes that make up the shape, from one corner's indexes to the
    /// other's; with `merge_neighbors`, as few as the greedy merge makes.
    pub fn for_all_boxes(
        &self,
        consumer: &mut dyn FnMut(i32, i32, i32, i32, i32, i32),
        merge_neighbors: bool,
    ) {
        BitSetDiscreteVoxelShape::for_all_boxes(self, consumer, merge_neighbors);
    }
}

/// `DiscreteVoxelShape`'s constructor check: Java throws `IllegalArgumentException` for a negative
/// size.
pub(crate) fn check_sizes(x_size: i32, y_size: i32, z_size: i32) {
    assert!(
        x_size >= 0 && y_size >= 0 && z_size >= 0,
        "Need all positive sizes: x: {x_size}, y: {y_size}, z: {z_size}"
    );
}

/// `rotate(OctahedralGroup)`: a new shape of the voxels the symmetry takes these to, shifted back
/// into the grid along each axis it flips; the shape itself for the identity. Java reuses the
/// vector of the sizes for each voxel's position, which the port does too.
pub fn rotate(
    this: &Arc<dyn DiscreteVoxelShape>,
    rotation: OctahedralGroup,
) -> Arc<dyn DiscreteVoxelShape> {
    if rotation == OctahedralGroup::IDENTITY {
        return Arc::clone(this);
    }

    let mut v = Vector3i::from_xyz(this.x_size(), this.y_size(), this.z_size());
    rotation.rotate_vector3i(&mut v);
    let shift_x = fixup_coordinate(&mut v, 0);
    let shift_y = fixup_coordinate(&mut v, 1);
    let shift_z = fixup_coordinate(&mut v, 2);
    let mut new_shape = BitSetDiscreteVoxelShape::new(v.x, v.y, v.z);
    for x in 0..this.x_size() {
        for y in 0..this.y_size() {
            for z in 0..this.z_size() {
                if this.is_full(x, y, z) {
                    let new_pos = *rotation.rotate_vector3i(v.set_xyz(x, y, z));
                    new_shape.fill(
                        shift_x + new_pos.x,
                        shift_y + new_pos.y,
                        shift_z + new_pos.z,
                    );
                }
            }
        }
    }
    Arc::new(new_shape)
}

/// A flipped size made positive again, and how far its voxels shift back into the grid.
fn fixup_coordinate(v: &mut Vector3i, index: i32) -> i32 {
    let value = v.get(index);
    if value < 0 {
        v.set_component(index, value.wrapping_neg());
        value.wrapping_neg() - 1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::golden::Golden;
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::world::phys::shapes::bit_set_discrete_voxel_shape::BitSetDiscreteVoxelShape;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.shapes.DiscreteVoxelShape");
        golden.check("isEmpty()Z", |shape: VoxelShape| shape.shape().is_empty());
        golden.check(
            "forAllFaces(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntFaceConsumer;)V",
            |shape: VoxelShape| {
                let mut faces: Vec<(Direction, i32, i32, i32)> = Vec::new();
                shape
                    .shape()
                    .for_all_faces(&mut |direction, x, y, z| faces.push((direction, x, y, z)));
                faces
            },
        );
        golden.check(
            "forAllEdges(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntLineConsumer;Z)V",
            |shape: VoxelShape, merge: bool| {
                let mut lines = Vec::new();
                shape.shape().for_all_edges(
                    &mut |x1, y1, z1, x2, y2, z2| lines.push([x1, y1, z1, x2, y2, z2]),
                    merge,
                );
                lines
            },
        );
        golden.check(
            "forAllBoxes(Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape$IntLineConsumer;Z)V",
            |shape: VoxelShape, merge: bool| {
                let mut lines = Vec::new();
                shape.shape().for_all_boxes(
                    &mut |x1, y1, z1, x2, y2, z2| lines.push([x1, y1, z1, x2, y2, z2]),
                    merge,
                );
                lines
            },
        );
        golden.check(
            "firstFull(Lnet/minecraft/core/Direction$Axis;II)I",
            |shape: VoxelShape, axis: Axis, b, c| shape.shape().first_full_at(axis, b, c),
        );
        golden.check(
            "lastFull(Lnet/minecraft/core/Direction$Axis;II)I",
            |shape: VoxelShape, axis: Axis, b, c| shape.shape().last_full_at(axis, b, c),
        );
        golden.check("isFullWide(III)Z", |shape: VoxelShape, x, y, z| {
            shape.shape().is_full_wide(x, y, z)
        });
        golden.check(
            "BitSetDiscreteVoxelShape.isInterior(III)Z",
            |shape: VoxelShape, x, y, z| {
                shape
                    .shape()
                    .as_any()
                    .downcast_ref::<BitSetDiscreteVoxelShape>()
                    .expect("vanilla only asks a BitSetDiscreteVoxelShape")
                    .is_interior(x, y, z)
            },
        );
        golden.pending(
            "rotate(Lcom/mojang/math/OctahedralGroup;)Lnet/minecraft/world/phys/shapes/DiscreteVoxelShape;",
            "needs OctahedralGroup",
        );
        golden.finish();
    }
}
