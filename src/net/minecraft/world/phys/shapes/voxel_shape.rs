//! net/minecraft/world/phys/shapes/VoxelShape.java
//!
//! An abstract class whose subclasses (`ArrayVoxelShape`, `CubeVoxelShape`, `SliceShape`) only
//! say where the voxels' coordinates come from. The game shares shapes and compares them by
//! reference (`shape == Shapes.block()`, `first == second`), so a `VoxelShape` is an `Arc` of its
//! discrete shape, its subclass's fields and its cache of face shapes, and `==` is `Arc::ptr_eq`
//! (Java's `equals` is `Object`'s). Its methods that a subclass overrides (`getCoords`,
//! `findIndex`) match on the subclass.
//!
//! Not ported yet: `toString`, which prints `AABB.toString` and needs Java's `Double.toString`.
//! `DoubleLineConsumer` is a closure.

use std::sync::{Arc, OnceLock};

use crate::com::google::common::math::double_math::DoubleMath;
use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
use crate::java::lang::math::Math;
use crate::net::minecraft::core::axis_cycle::AxisCycle;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, AxisDirection, Direction};
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::block_hit_result::BlockHitResult;
use crate::net::minecraft::world::phys::shapes::array_voxel_shape::ArrayVoxelShape;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::cube_voxel_shape::CubeVoxelShape;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::DiscreteVoxelShape;
use crate::net::minecraft::world::phys::shapes::offset_double_list::OffsetDoubleList;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::slice_shape::SliceShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// A shape made of boxes: a grid of voxels, each full or not, and where the grid's lines are
/// along each axis.
#[derive(Clone)]
pub struct VoxelShape(Arc<Data>);

struct Data {
    shape: Arc<dyn DiscreteVoxelShape>,
    kind: Kind,
    faces: [OnceLock<VoxelShape>; 6],
}

/// The subclass, with its fields.
pub(crate) enum Kind {
    Array(ArrayVoxelShape),
    Cube(CubeVoxelShape),
    Slice(SliceShape),
}

impl PartialEq for VoxelShape {
    fn eq(&self, other: &VoxelShape) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for VoxelShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "EMPTY")
        } else {
            write!(f, "VoxelShape[{:?}]", self.bounds())
        }
    }
}

impl VoxelShape {
    pub(crate) fn from_kind(shape: Arc<dyn DiscreteVoxelShape>, kind: Kind) -> VoxelShape {
        VoxelShape(Arc::new(Data {
            shape,
            kind,
            faces: Default::default(),
        }))
    }

    /// The protected `shape` field: its voxels.
    pub fn shape(&self) -> &Arc<dyn DiscreteVoxelShape> {
        &self.0.shape
    }

    #[cfg(test)]
    pub(crate) fn kind(&self) -> &Kind {
        &self.0.kind
    }

    pub fn min(&self, axis: Axis) -> f64 {
        let i = self.0.shape.first_full(axis);
        if i >= self.0.shape.get_size(axis) {
            f64::INFINITY
        } else {
            self.get(axis, i)
        }
    }

    pub fn max(&self, axis: Axis) -> f64 {
        let i = self.0.shape.last_full(axis);
        if i <= 0 {
            f64::NEG_INFINITY
        } else {
            self.get(axis, i)
        }
    }

    /// Throws `UnsupportedOperationException` for an empty shape, a panic here.
    pub fn bounds(&self) -> AABB {
        assert!(!self.is_empty(), "No bounds for empty shape.");
        AABB::new(
            self.min(Axis::X),
            self.min(Axis::Y),
            self.min(Axis::Z),
            self.max(Axis::X),
            self.max(Axis::Y),
            self.max(Axis::Z),
        )
    }

    pub fn single_encompassing(&self) -> VoxelShape {
        if self.is_empty() {
            Shapes::empty()
        } else {
            Shapes::r#box(
                self.min(Axis::X),
                self.min(Axis::Y),
                self.min(Axis::Z),
                self.max(Axis::X),
                self.max(Axis::Y),
                self.max(Axis::Z),
            )
        }
    }

    pub(crate) fn get(&self, axis: Axis, i: i32) -> f64 {
        self.get_coords(axis).get_double(i)
    }

    pub fn get_coords(&self, axis: Axis) -> Arc<dyn DoubleList> {
        match &self.0.kind {
            Kind::Array(array) => array.get_coords(axis),
            Kind::Cube(_) => CubeVoxelShape::get_coords(&*self.0.shape, axis),
            Kind::Slice(slice) => slice.get_coords(axis),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.shape.is_empty()
    }

    pub fn r#move(&self, delta: &Vec3) -> VoxelShape {
        self.move_xyz(delta.x, delta.y, delta.z)
    }

    pub fn move_vec3i(&self, delta: &Vec3i) -> VoxelShape {
        self.move_xyz(
            f64::from(delta.get_x()),
            f64::from(delta.get_y()),
            f64::from(delta.get_z()),
        )
    }

    pub fn move_xyz(&self, dx: f64, dy: f64, dz: f64) -> VoxelShape {
        if self.is_empty() {
            Shapes::empty()
        } else {
            ArrayVoxelShape::new(
                Arc::clone(&self.0.shape),
                Arc::new(OffsetDoubleList::new(self.get_coords(Axis::X), dx)),
                Arc::new(OffsetDoubleList::new(self.get_coords(Axis::Y), dy)),
                Arc::new(OffsetDoubleList::new(self.get_coords(Axis::Z), dz)),
            )
        }
    }

    /// The shape as the fewest boxes the greedy merge finds, each joined in turn.
    pub fn optimize(&self) -> VoxelShape {
        let mut result = Shapes::empty();
        self.for_all_boxes(&mut |x1, y1, z1, x2, y2, z2| {
            result = Shapes::join_unoptimized(
                &result,
                &Shapes::r#box(x1, y1, z1, x2, y2, z2),
                BooleanOp::OR,
            );
        });
        result
    }

    pub fn for_all_edges(&self, consumer: &mut dyn FnMut(f64, f64, f64, f64, f64, f64)) {
        self.0.shape.for_all_edges(
            &mut |xi1, yi1, zi1, xi2, yi2, zi2| {
                consumer(
                    self.get(Axis::X, xi1),
                    self.get(Axis::Y, yi1),
                    self.get(Axis::Z, zi1),
                    self.get(Axis::X, xi2),
                    self.get(Axis::Y, yi2),
                    self.get(Axis::Z, zi2),
                )
            },
            true,
        );
    }

    pub fn for_all_boxes(&self, consumer: &mut dyn FnMut(f64, f64, f64, f64, f64, f64)) {
        let x_coords = self.get_coords(Axis::X);
        let y_coords = self.get_coords(Axis::Y);
        let z_coords = self.get_coords(Axis::Z);
        self.0.shape.for_all_boxes(
            &mut |xi1, yi1, zi1, xi2, yi2, zi2| {
                consumer(
                    x_coords.get_double(xi1),
                    y_coords.get_double(yi1),
                    z_coords.get_double(zi1),
                    x_coords.get_double(xi2),
                    y_coords.get_double(yi2),
                    z_coords.get_double(zi2),
                )
            },
            true,
        );
    }

    pub fn to_aabbs(&self) -> Vec<AABB> {
        let mut list = Vec::new();
        self.for_all_boxes(&mut |x1, y1, z1, x2, y2, z2| {
            list.push(AABB::new(x1, y1, z1, x2, y2, z2))
        });
        list
    }

    /// `min(Axis, double, double)`: where the shape starts along `a_axis` in the row through `b`
    /// and `c` on the next axes, or infinity when the row is empty.
    pub fn min_at(&self, a_axis: Axis, b: f64, c: f64) -> f64 {
        let b_axis = AxisCycle::FORWARD.cycle_axis(a_axis);
        let c_axis = AxisCycle::BACKWARD.cycle_axis(a_axis);
        let bi = self.find_index(b_axis, b);
        let ci = self.find_index(c_axis, c);
        let i = self.0.shape.first_full_at(a_axis, bi, ci);
        if i >= self.0.shape.get_size(a_axis) {
            f64::INFINITY
        } else {
            self.get(a_axis, i)
        }
    }

    /// `max(Axis, double, double)`, like `min_at`.
    pub fn max_at(&self, a_axis: Axis, b: f64, c: f64) -> f64 {
        let b_axis = AxisCycle::FORWARD.cycle_axis(a_axis);
        let c_axis = AxisCycle::BACKWARD.cycle_axis(a_axis);
        let bi = self.find_index(b_axis, b);
        let ci = self.find_index(c_axis, c);
        let i = self.0.shape.last_full_at(a_axis, bi, ci);
        if i <= 0 {
            f64::NEG_INFINITY
        } else {
            self.get(a_axis, i)
        }
    }

    /// The voxel along `axis` that `coord` is in: -1 before the first, the size after the last.
    pub(crate) fn find_index(&self, axis: Axis, coord: f64) -> i32 {
        match &self.0.kind {
            Kind::Cube(_) => CubeVoxelShape::find_index(&*self.0.shape, axis, coord),
            _ => {
                Mth::binary_search(0, self.0.shape.get_size(axis) + 1, |index| {
                    coord < self.get(axis, index)
                }) - 1
            }
        }
    }

    /// Where the ray from `from` to `to` enters the shape at `pos`: inside it at once when the
    /// point 0.001 of the way along is in a full voxel.
    pub fn clip(&self, from: &Vec3, to: &Vec3, pos: &BlockPos) -> Option<BlockHitResult> {
        if self.is_empty() {
            return None;
        }

        let diff = to.subtract(from);
        if diff.length_sqr() < 1.0E-7 {
            return None;
        }

        let test_point = from.add(&diff.scale(0.001));
        if self.0.shape.is_full_wide(
            self.find_index(Axis::X, test_point.x - f64::from(pos.get_x())),
            self.find_index(Axis::Y, test_point.y - f64::from(pos.get_y())),
            self.find_index(Axis::Z, test_point.z - f64::from(pos.get_z())),
        ) {
            Some(BlockHitResult::new(
                &test_point,
                Direction::get_approximate_nearest_f64(diff.x, diff.y, diff.z).get_opposite(),
                pos,
                true,
            ))
        } else {
            AABB::clip_aabbs(&self.to_aabbs(), from, to, pos)
        }
    }

    /// The point of the shape closest to `point`: the first box's closest point, then each next
    /// box's that's strictly closer.
    pub fn closest_point_to(&self, point: &Vec3) -> Option<Vec3> {
        if self.is_empty() {
            return None;
        }

        let mut closest: Option<Vec3> = None;
        self.for_all_boxes(&mut |x1, y1, z1, x2, y2, z2| {
            let x = Mth::clamp_f64(point.x, x1, x2);
            let y = Mth::clamp_f64(point.y, y1, y2);
            let z = Mth::clamp_f64(point.z, z1, z2);
            if closest.is_none_or(|current_closest| {
                point.distance_to_sqr_xyz(x, y, z) < point.distance_to_sqr(&current_closest)
            }) {
                closest = Some(Vec3::new(x, y, z));
            }
        });
        Some(closest.expect("a shape that isn't empty has a box"))
    }

    /// The shape's face on the block's side in `direction`, cached: the shape itself when it
    /// fills the block across that axis, or the slice of voxels at that side.
    pub fn get_face_shape(&self, direction: Direction) -> VoxelShape {
        if !self.is_empty() && *self != Shapes::block() {
            self.0.faces[direction as usize]
                .get_or_init(|| self.calculate_face(direction))
                .clone()
        } else {
            self.clone()
        }
    }

    fn calculate_face(&self, direction: Direction) -> VoxelShape {
        let axis = direction.get_axis();
        if self.is_cube_like_along(axis) {
            self.clone()
        } else {
            let sign = direction.get_axis_direction();
            let index = self.find_index(
                axis,
                if sign == AxisDirection::POSITIVE {
                    0.9999999
                } else {
                    1.0E-7
                },
            );
            let slice = SliceShape::new(self.clone(), axis, index);
            if slice.is_empty() {
                Shapes::empty()
            } else if slice.is_cube_like() {
                Shapes::block()
            } else {
                slice
            }
        }
    }

    pub(crate) fn is_cube_like(&self) -> bool {
        Axis::VALUES
            .into_iter()
            .all(|axis| self.is_cube_like_along(axis))
    }

    fn is_cube_like_along(&self, axis: Axis) -> bool {
        let coords = self.get_coords(axis);
        coords.size() == 2
            && DoubleMath::fuzzy_equals(coords.get_double(0), 0.0, 1.0E-7)
            && DoubleMath::fuzzy_equals(coords.get_double(1), 1.0, 1.0E-7)
    }

    /// How far `moving` can go along `axis`, up to `distance`, before it hits the shape: a
    /// distance under 1.0E-7 is none, and the shape's sides within 1.0E-7 inside the box don't
    /// stop it.
    pub fn collide(&self, axis: Axis, moving: &AABB, distance: f64) -> f64 {
        self.collide_x(AxisCycle::between(axis, Axis::X), moving, distance)
    }

    pub(crate) fn collide_x(&self, transform: AxisCycle, moving: &AABB, mut distance: f64) -> f64 {
        if self.is_empty() {
            return distance;
        }

        if distance.abs() < 1.0E-7 {
            return 0.0;
        }

        let shape = &self.0.shape;
        let inverse = transform.inverse();
        let a_axis = inverse.cycle_axis(Axis::X);
        let b_axis = inverse.cycle_axis(Axis::Y);
        let c_axis = inverse.cycle_axis(Axis::Z);
        let max_a = moving.max(a_axis);
        let min_a = moving.min(a_axis);
        let a_min = self.find_index(a_axis, min_a + 1.0E-7);
        let a_max = self.find_index(a_axis, max_a - 1.0E-7);
        let b_min = 0.max(self.find_index(b_axis, moving.min(b_axis) + 1.0E-7));
        let b_max = shape
            .get_size(b_axis)
            .min(self.find_index(b_axis, moving.max(b_axis) - 1.0E-7) + 1);
        let c_min = 0.max(self.find_index(c_axis, moving.min(c_axis) + 1.0E-7));
        let c_max = shape
            .get_size(c_axis)
            .min(self.find_index(c_axis, moving.max(c_axis) - 1.0E-7) + 1);
        let a_size = shape.get_size(a_axis);
        if distance > 0.0 {
            for a in a_max + 1..a_size {
                for b in b_min..b_max {
                    for c in c_min..c_max {
                        if shape.is_full_wide_cycled(inverse, a, b, c) {
                            let new_distance = self.get(a_axis, a) - max_a;
                            if new_distance >= -1.0E-7 {
                                distance = Math::min_f64(distance, new_distance);
                            }

                            return distance;
                        }
                    }
                }
            }
        } else if distance < 0.0 {
            for a in (0..a_min).rev() {
                for b in b_min..b_max {
                    for c in c_min..c_max {
                        if shape.is_full_wide_cycled(inverse, a, b, c) {
                            let new_distance = self.get(a_axis, a + 1) - min_a;
                            if new_distance <= 1.0E-7 {
                                distance = Math::max_f64(distance, new_distance);
                            }

                            return distance;
                        }
                    }
                }
            }
        }

        distance
    }
}

/// What the shapes' goldens encode (TESTING.md): a shape argument is the recipe that makes it,
/// which this follows with the port's own functions; a shape result is the constant's name, or its
/// class, coordinates and voxels.
#[cfg(test)]
pub(crate) mod golden {
    use std::sync::Arc;

    use serde_json::{Value, json};

    use super::{Kind, VoxelShape};
    use crate::golden::{Decode, Encode};
    use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::world::phys::shapes::bit_set_discrete_voxel_shape::BitSetDiscreteVoxelShape;
    use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
    use crate::net::minecraft::world::phys::shapes::cube_point_range::CubePointRange;
    use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::DiscreteVoxelShape;
    use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;
    use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
    use crate::net::minecraft::world::phys::shapes::slice_shape::SliceShape;

    impl Decode for BooleanOp {
        fn decode(value: &Value) -> Self {
            let name = value.as_str().expect("a BooleanOp is its field's name");
            *BooleanOp::VALUES
                .iter()
                .find(|op| op.name() == name)
                .unwrap_or_else(|| panic!("no BooleanOp {name}"))
        }
    }

    impl Decode for VoxelShape {
        fn decode(value: &Value) -> Self {
            if let Some(name) = value.as_str() {
                return match name {
                    "BLOCK" => Shapes::block(),
                    "EMPTY" => Shapes::empty(),
                    "INFINITY" => Shapes::infinity(),
                    _ => panic!("no shape constant {name}"),
                };
            }
            let recipe = value.as_array().expect("a shape's recipe is an array");
            let d = |index: usize| f64::decode(&recipe[index]);
            match recipe[0].as_str() {
                Some("box") => Shapes::r#box(d(1), d(2), d(3), d(4), d(5), d(6)),
                Some("create") => Shapes::create(d(1), d(2), d(3), d(4), d(5), d(6)),
                Some("move") => VoxelShape::decode(&recipe[1]).move_xyz(d(2), d(3), d(4)),
                Some("join") => Shapes::join(
                    &VoxelShape::decode(&recipe[1]),
                    &VoxelShape::decode(&recipe[2]),
                    BooleanOp::decode(&recipe[3]),
                ),
                Some("joinUnoptimized") => Shapes::join_unoptimized(
                    &VoxelShape::decode(&recipe[1]),
                    &VoxelShape::decode(&recipe[2]),
                    BooleanOp::decode(&recipe[3]),
                ),
                Some("face") => {
                    VoxelShape::decode(&recipe[1]).get_face_shape(Direction::decode(&recipe[2]))
                }
                Some("slice") => SliceShape::new(
                    VoxelShape::decode(&recipe[1]),
                    Axis::decode(&recipe[2]),
                    i32::decode(&recipe[3]),
                ),
                Some("optimize") => VoxelShape::decode(&recipe[1]).optimize(),
                _ => panic!("no shape recipe {value}"),
            }
        }
    }

    impl Encode for VoxelShape {
        fn encode(&self) -> Value {
            if *self == Shapes::block() {
                Value::from("BLOCK")
            } else if *self == Shapes::empty() {
                Value::from("EMPTY")
            } else if *self == Shapes::infinity() {
                Value::from("INFINITY")
            } else {
                Structure(self.clone()).encode()
            }
        }
    }

    /// A shape encoded by what it holds, even when it's a constant.
    pub(crate) struct Structure(pub VoxelShape);

    impl Encode for Structure {
        fn encode(&self) -> Value {
            let shape = &self.0;
            let class = match shape.kind() {
                Kind::Array(_) => "ArrayVoxelShape",
                Kind::Cube(_) => "CubeVoxelShape",
                Kind::Slice(_) => "SliceShape",
            };
            json!([
                class,
                [
                    list(&*shape.get_coords(Axis::X)),
                    list(&*shape.get_coords(Axis::Y)),
                    list(&*shape.get_coords(Axis::Z)),
                ],
                discrete(&**shape.shape()),
            ])
        }
    }

    /// A coordinate list: `["cube", values]` for a `CubePointRange`, `["array", values]` for any
    /// other.
    pub(crate) fn list(list: &dyn DoubleList) -> Value {
        let kind = if list.as_any().is::<CubePointRange>() {
            "cube"
        } else {
            "array"
        };
        let values: Vec<Value> = (0..list.size())
            .map(|index| list.get_double(index).encode())
            .collect();
        json!([kind, values])
    }

    /// `[class, xSize, ySize, zSize, firstFull, lastFull, bits]`.
    pub(crate) fn discrete(shape: &dyn DiscreteVoxelShape) -> Value {
        let class = if shape.as_any().is::<BitSetDiscreteVoxelShape>() {
            "BitSetDiscreteVoxelShape"
        } else {
            "SubShape"
        };
        let mut bits = String::new();
        for x in 0..shape.get_x_size() {
            for y in 0..shape.get_y_size() {
                for z in 0..shape.get_z_size() {
                    bits.push(if shape.is_full(x, y, z) { '1' } else { '0' });
                }
            }
        }
        json!([
            class,
            shape.get_x_size(),
            shape.get_y_size(),
            shape.get_z_size(),
            Axis::VALUES.map(|axis| shape.first_full(axis)),
            Axis::VALUES.map(|axis| shape.last_full(axis)),
            bits,
        ])
    }

    /// A coordinate list result.
    pub(crate) struct ListEncoding(pub Arc<dyn DoubleList>);

    impl Encode for ListEncoding {
        fn encode(&self) -> Value {
            list(&*self.0)
        }
    }

    /// An index merger: `[class, list, size, [[first, second, result], …]]`.
    pub(crate) struct MergerEncoding(pub Box<dyn IndexMerger>);

    impl Encode for MergerEncoding {
        fn encode(&self) -> Value {
            use crate::net::minecraft::world::phys::shapes::discrete_cube_merger::DiscreteCubeMerger;
            use crate::net::minecraft::world::phys::shapes::identical_merger::IdenticalMerger;
            use crate::net::minecraft::world::phys::shapes::indirect_merger::IndirectMerger;

            let merger = self.0.as_any();
            let class = if merger.is::<DiscreteCubeMerger>() {
                "DiscreteCubeMerger"
            } else if merger.is::<IdenticalMerger>() {
                "IdenticalMerger"
            } else if merger.is::<IndirectMerger>() {
                "IndirectMerger"
            } else {
                "NonOverlappingMerger"
            };
            let mut triples = Vec::new();
            self.0.for_merged_indexes(&mut |first, second, result| {
                triples.push(json!([first, second, result]));
                true
            });
            json!([class, list(&*self.0.get_list()), self.0.size(), triples])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VoxelShape;
    use super::golden::ListEncoding;
    use crate::golden::Golden;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::core::vec3i::Vec3i;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.shapes.VoxelShape");
        golden.check(
            "min(Lnet/minecraft/core/Direction$Axis;)D",
            |shape: VoxelShape, axis| shape.min(axis),
        );
        golden.check(
            "max(Lnet/minecraft/core/Direction$Axis;)D",
            |shape: VoxelShape, axis| shape.max(axis),
        );
        golden.check(
            "getCoords(Lnet/minecraft/core/Direction$Axis;)Lit/unimi/dsi/fastutil/doubles/DoubleList;",
            |shape: VoxelShape, axis| ListEncoding(shape.get_coords(axis)),
        );
        golden.check(
            "min(Lnet/minecraft/core/Direction$Axis;DD)D",
            |shape: VoxelShape, axis, b, c| shape.min_at(axis, b, c),
        );
        golden.check(
            "max(Lnet/minecraft/core/Direction$Axis;DD)D",
            |shape: VoxelShape, axis, b, c| shape.max_at(axis, b, c),
        );
        golden.check(
            "findIndex(Lnet/minecraft/core/Direction$Axis;D)I",
            |shape: VoxelShape, axis: Axis, coord| shape.find_index(axis, coord),
        );
        golden.check(
            "bounds()Lnet/minecraft/world/phys/AABB;",
            |shape: VoxelShape| shape.bounds(),
        );
        golden.check(
            "singleEncompassing()Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape| shape.single_encompassing(),
        );
        golden.check("isEmpty()Z", |shape: VoxelShape| shape.is_empty());
        golden.check(
            "optimize()Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape| shape.optimize(),
        );
        golden.check("toAabbs()Ljava/util/List;", |shape: VoxelShape| {
            shape.to_aabbs()
        });
        golden.check(
            "forAllEdges(Lnet/minecraft/world/phys/shapes/Shapes$DoubleLineConsumer;)V",
            |shape: VoxelShape| {
                let mut lines = Vec::new();
                shape.for_all_edges(&mut |x1, y1, z1, x2, y2, z2| {
                    lines.push([x1, y1, z1, x2, y2, z2])
                });
                lines
            },
        );
        golden.check(
            "forAllBoxes(Lnet/minecraft/world/phys/shapes/Shapes$DoubleLineConsumer;)V",
            |shape: VoxelShape| {
                let mut lines = Vec::new();
                shape.for_all_boxes(&mut |x1, y1, z1, x2, y2, z2| {
                    lines.push([x1, y1, z1, x2, y2, z2])
                });
                lines
            },
        );
        golden.check(
            "getFaceShape(Lnet/minecraft/core/Direction;)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape, direction: Direction| shape.get_face_shape(direction),
        );
        golden.check(
            "move(DDD)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape, dx, dy, dz| shape.move_xyz(dx, dy, dz),
        );
        golden.check(
            "move(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape, delta: Vec3| shape.r#move(&delta),
        );
        golden.check(
            "move(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |shape: VoxelShape, delta: Vec3i| shape.move_vec3i(&delta),
        );
        golden.check(
            "closestPointTo(Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;",
            |shape: VoxelShape, point: Vec3| shape.closest_point_to(&point),
        );
        golden.check(
            "clip(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
            |shape: VoxelShape, from: Vec3, to: Vec3, pos: BlockPos| shape.clip(&from, &to, &pos),
        );
        golden.check(
            "collide(Lnet/minecraft/core/Direction$Axis;Lnet/minecraft/world/phys/AABB;D)D",
            |shape: VoxelShape, axis, moving: AABB, distance| {
                shape.collide(axis, &moving, distance)
            },
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints AABB.toString, which needs Java's Double.toString",
        );
        golden.pending(
            "equals(Ljava/lang/Object;)Z",
            "Object's, by reference, which is VoxelShape's PartialEq",
        );
        golden.finish();
    }
}
