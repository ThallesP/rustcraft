//! net/minecraft/world/phys/shapes/Shapes.java
//!
//! `BLOCK`, `EMPTY` and `INFINITY` are statics made on first use, and each hands out the same
//! shape, which the game compares with by reference (`shape == Shapes.block()`).
//!
//! The maps of rotated shapes (`rotateHorizontal`, `rotateAll`, the axes' maps) are Java's
//! `EnumMap`s, which iterate in their keys' order: here a `Vec` of the entries in that order,
//! which `get` searches; `rotateAttachFace`'s `Map.of`, which only `get` reads, is a `Vec` too.
//! `DoubleLineConsumer` is a closure.

use std::sync::{Arc, LazyLock};

use crate::com::google::common::math::double_math::DoubleMath;
use crate::com::google::common::math::int_math::IntMath;
use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::it::unimi::dsi::fastutil::doubles::double_array_list::DoubleArrayList;
use crate::it::unimi::dsi::fastutil::doubles::double_list::{self, DoubleList};
use crate::java::lang::math::Math;
use crate::net::minecraft::core::axis_cycle::AxisCycle;
use crate::net::minecraft::core::direction::{Axis, AxisDirection, Direction};
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::array_voxel_shape::ArrayVoxelShape;
use crate::net::minecraft::world::phys::shapes::bit_set_discrete_voxel_shape::BitSetDiscreteVoxelShape;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::cube_point_range::CubePointRange;
use crate::net::minecraft::world::phys::shapes::cube_voxel_shape::CubeVoxelShape;
use crate::net::minecraft::world::phys::shapes::discrete_cube_merger::DiscreteCubeMerger;
use crate::net::minecraft::world::phys::shapes::discrete_voxel_shape::{self, DiscreteVoxelShape};
use crate::net::minecraft::world::phys::shapes::identical_merger::IdenticalMerger;
use crate::net::minecraft::world::phys::shapes::index_merger::IndexMerger;
use crate::net::minecraft::world::phys::shapes::indirect_merger::IndirectMerger;
use crate::net::minecraft::world::phys::shapes::non_overlapping_merger::NonOverlappingMerger;
use crate::net::minecraft::world::phys::shapes::slice_shape::SliceShape;
use crate::net::minecraft::world::phys::shapes::voxel_shape::{Kind, VoxelShape};
use crate::net::minecraft::world::phys::vec3::Vec3;

static BLOCK: LazyLock<VoxelShape> = LazyLock::new(|| {
    let mut shape = BitSetDiscreteVoxelShape::new(1, 1, 1);
    shape.fill(0, 0, 0);
    CubeVoxelShape::new(Arc::new(shape))
});

static INFINITY: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::r#box(
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
    )
});

static EMPTY: LazyLock<VoxelShape> = LazyLock::new(|| {
    ArrayVoxelShape::from_arrays(
        Arc::new(BitSetDiscreteVoxelShape::new(0, 0, 0)),
        &[0.0],
        &[0.0],
        &[0.0],
    )
});

/// Making shapes and combining them.
pub struct Shapes;

impl Shapes {
    pub const EPSILON: f64 = 1.0E-7;
    pub const BIG_EPSILON: f64 = 1.0E-6;

    pub fn empty() -> VoxelShape {
        EMPTY.clone()
    }

    pub fn block() -> VoxelShape {
        BLOCK.clone()
    }

    /// `block()` borrowed, to compare a shape with by reference without sharing it again.
    pub fn block_ref() -> &'static VoxelShape {
        &BLOCK
    }

    /// `empty()` borrowed.
    pub fn empty_ref() -> &'static VoxelShape {
        &EMPTY
    }

    /// The `INFINITY` field: the shape of all space.
    pub fn infinity() -> VoxelShape {
        INFINITY.clone()
    }

    /// Throws `IllegalArgumentException` when a minimum is past its maximum, a panic here. Negated
    /// like Java's, so a NaN side passes.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn r#box(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> VoxelShape {
        assert!(
            !(min_x > max_x) && !(min_y > max_y) && !(min_z > max_z),
            "The min values need to be smaller or equals to the max values"
        );
        Shapes::create(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// A box: empty when it's thinner than 1.0E-7 along an axis, the block itself, a cube shape
    /// when each side is on a block's halves, quarters or eighths, or else an array shape of one
    /// voxel. Negated like Java's, so a NaN side makes a shape.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn create(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> VoxelShape {
        if !(max_x - min_x < 1.0E-7) && !(max_y - min_y < 1.0E-7) && !(max_z - min_z < 1.0E-7) {
            let x_bits = Shapes::find_bits(min_x, max_x);
            let y_bits = Shapes::find_bits(min_y, max_y);
            let z_bits = Shapes::find_bits(min_z, max_z);
            if x_bits < 0 || y_bits < 0 || z_bits < 0 {
                return ArrayVoxelShape::from_arrays(
                    Arc::clone(BLOCK.shape()),
                    &[min_x, max_x],
                    &[min_y, max_y],
                    &[min_z, max_z],
                );
            }

            if x_bits == 0 && y_bits == 0 && z_bits == 0 {
                return Shapes::block();
            }

            let x_size = 1 << x_bits;
            let y_size = 1 << y_bits;
            let z_size = 1 << z_bits;
            let voxel_shape = BitSetDiscreteVoxelShape::with_filled_bounds(
                x_size,
                y_size,
                z_size,
                Math::round_f64(min_x * f64::from(x_size)) as i32,
                Math::round_f64(min_y * f64::from(y_size)) as i32,
                Math::round_f64(min_z * f64::from(z_size)) as i32,
                Math::round_f64(max_x * f64::from(x_size)) as i32,
                Math::round_f64(max_y * f64::from(y_size)) as i32,
                Math::round_f64(max_z * f64::from(z_size)) as i32,
            );
            CubeVoxelShape::new(Arc::new(voxel_shape))
        } else {
            Shapes::empty()
        }
    }

    /// `create(AABB)`.
    pub fn create_aabb(aabb: &AABB) -> VoxelShape {
        Shapes::create(
            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
        )
    }

    /// How many times the block has to be halved along an axis for both sides to be on a line,
    /// to 1.0E-7, up to 3 (eighths); -1 when they never are, or when the sides are outside the
    /// block.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub(crate) fn find_bits(min: f64, max: f64) -> i32 {
        if !(min < -1.0E-7) && !(max > 1.0000001) {
            for bits in 0..=3 {
                let intervals = 1 << bits;
                let sh_min = min * f64::from(intervals);
                let sh_max = max * f64::from(intervals);
                let found_min =
                    (sh_min - Math::round_f64(sh_min) as f64).abs() < 1.0E-7 * f64::from(intervals);
                let found_max =
                    (sh_max - Math::round_f64(sh_max) as f64).abs() < 1.0E-7 * f64::from(intervals);
                if found_min && found_max {
                    return bits;
                }
            }
            -1
        } else {
            -1
        }
    }

    /// The least common multiple, as a long; throws for a negative argument or two zeros.
    pub(crate) fn lcm(first: i32, second: i32) -> i64 {
        i64::from(first) * i64::from(second / IntMath::gcd(first, second))
    }

    pub fn or(first: &VoxelShape, second: &VoxelShape) -> VoxelShape {
        Shapes::join(first, second, BooleanOp::OR)
    }

    /// `or(VoxelShape, VoxelShape...)`.
    pub fn or_tail(first: &VoxelShape, tail: &[VoxelShape]) -> VoxelShape {
        tail.iter()
            .fold(first.clone(), |result, shape| Shapes::or(&result, shape))
    }

    pub fn join(first: &VoxelShape, second: &VoxelShape, op: BooleanOp) -> VoxelShape {
        Shapes::join_unoptimized(first, second, op).optimize()
    }

    /// The voxels where `op` holds, over the merged grid. Throws `IllegalArgumentException` for
    /// an `op` that holds of two empty voxels, which would fill all space.
    pub fn join_unoptimized(first: &VoxelShape, second: &VoxelShape, op: BooleanOp) -> VoxelShape {
        assert!(!op.apply(false, false), "IllegalArgumentException");

        if first == second {
            return if op.apply(true, true) {
                first.clone()
            } else {
                Shapes::empty()
            };
        }

        let first_only_matters = op.apply(true, false);
        let second_only_matters = op.apply(false, true);
        if first.is_empty() {
            return if second_only_matters {
                second.clone()
            } else {
                Shapes::empty()
            };
        }

        if second.is_empty() {
            return if first_only_matters {
                first.clone()
            } else {
                Shapes::empty()
            };
        }

        let x_merger = Shapes::create_index_merger(
            1,
            first.get_coords(Axis::X),
            second.get_coords(Axis::X),
            first_only_matters,
            second_only_matters,
        );
        let y_merger = Shapes::create_index_merger(
            x_merger.size() - 1,
            first.get_coords(Axis::Y),
            second.get_coords(Axis::Y),
            first_only_matters,
            second_only_matters,
        );
        let z_merger = Shapes::create_index_merger(
            (x_merger.size() - 1).wrapping_mul(y_merger.size() - 1),
            first.get_coords(Axis::Z),
            second.get_coords(Axis::Z),
            first_only_matters,
            second_only_matters,
        );
        let voxel_shape = BitSetDiscreteVoxelShape::join(
            &**first.shape(),
            &**second.shape(),
            &*x_merger,
            &*y_merger,
            &*z_merger,
            op,
        );
        let is_cube = |merger: &dyn IndexMerger| merger.as_any().is::<DiscreteCubeMerger>();
        if is_cube(&*x_merger) && is_cube(&*y_merger) && is_cube(&*z_merger) {
            CubeVoxelShape::new(Arc::new(voxel_shape))
        } else {
            ArrayVoxelShape::new(
                Arc::new(voxel_shape),
                x_merger.get_list(),
                y_merger.get_list(),
                z_merger.get_list(),
            )
        }
    }

    /// Whether `join_unoptimized` would be anything, without making it: shapes more than 1.0E-7
    /// apart along an axis can't meet.
    pub fn join_is_not_empty(first: &VoxelShape, second: &VoxelShape, op: BooleanOp) -> bool {
        assert!(!op.apply(false, false), "IllegalArgumentException");

        let first_empty = first.is_empty();
        let second_empty = second.is_empty();
        if !first_empty && !second_empty {
            if first == second {
                return op.apply(true, true);
            }

            let first_only_matters = op.apply(true, false);
            let second_only_matters = op.apply(false, true);

            for axis in AxisCycle::AXIS_VALUES {
                if first.max(axis) < second.min(axis) - 1.0E-7 {
                    return first_only_matters || second_only_matters;
                }

                if second.max(axis) < first.min(axis) - 1.0E-7 {
                    return first_only_matters || second_only_matters;
                }
            }

            let x_merger = Shapes::create_index_merger(
                1,
                first.get_coords(Axis::X),
                second.get_coords(Axis::X),
                first_only_matters,
                second_only_matters,
            );
            let y_merger = Shapes::create_index_merger(
                x_merger.size() - 1,
                first.get_coords(Axis::Y),
                second.get_coords(Axis::Y),
                first_only_matters,
                second_only_matters,
            );
            let z_merger = Shapes::create_index_merger(
                (x_merger.size() - 1).wrapping_mul(y_merger.size() - 1),
                first.get_coords(Axis::Z),
                second.get_coords(Axis::Z),
                first_only_matters,
                second_only_matters,
            );
            Shapes::join_is_not_empty_merged(
                &*x_merger,
                &*y_merger,
                &*z_merger,
                &**first.shape(),
                &**second.shape(),
                op,
            )
        } else {
            op.apply(!first_empty, !second_empty)
        }
    }

    fn join_is_not_empty_merged(
        x_merger: &dyn IndexMerger,
        y_merger: &dyn IndexMerger,
        z_merger: &dyn IndexMerger,
        first: &dyn DiscreteVoxelShape,
        second: &dyn DiscreteVoxelShape,
        op: BooleanOp,
    ) -> bool {
        !x_merger.for_merged_indexes(&mut |x1, x2, _| {
            y_merger.for_merged_indexes(&mut |y1, y2, _| {
                z_merger.for_merged_indexes(&mut |z1, z2, _| {
                    !op.apply(
                        first.is_full_wide(x1, y1, z1),
                        second.is_full_wide(x2, y2, z2),
                    )
                })
            })
        })
    }

    /// How far `moving` can go along `axis`, up to `distance`, before it hits one of the shapes;
    /// none once the distance left is under 1.0E-7.
    pub fn collide<'a>(
        axis: Axis,
        moving: &AABB,
        shapes: impl IntoIterator<Item = &'a VoxelShape>,
        mut distance: f64,
    ) -> f64 {
        for shape in shapes {
            if distance.abs() < 1.0E-7 {
                return 0.0;
            }

            distance = shape.collide(axis, moving, distance);
        }
        distance
    }

    /// Whether `occluder`, next to `shape` in `direction`, covers the face of it on that side.
    pub fn block_occludes(shape: &VoxelShape, occluder: &VoxelShape, direction: Direction) -> bool {
        if shape == Shapes::block_ref() && occluder == Shapes::block_ref() {
            return true;
        }

        if occluder.is_empty() {
            return false;
        }

        let axis = direction.get_axis();
        let sign = direction.get_axis_direction();
        let (first, second) = if sign == AxisDirection::POSITIVE {
            (shape, occluder)
        } else {
            (occluder, shape)
        };
        let op = if sign == AxisDirection::POSITIVE {
            BooleanOp::ONLY_FIRST
        } else {
            BooleanOp::ONLY_SECOND
        };
        DoubleMath::fuzzy_equals(first.max(axis), 1.0, 1.0E-7)
            && DoubleMath::fuzzy_equals(second.min(axis), 0.0, 1.0E-7)
            && !Shapes::join_is_not_empty(
                &SliceShape::new(first.clone(), axis, first.shape().get_size(axis) - 1),
                &SliceShape::new(second.clone(), axis, 0),
                op,
            )
    }

    /// Whether the two shapes' faces where they meet in `direction`, together, cover the block's
    /// face.
    pub fn merged_face_occludes(
        shape: &VoxelShape,
        occluder: &VoxelShape,
        direction: Direction,
    ) -> bool {
        if shape != Shapes::block_ref() && occluder != Shapes::block_ref() {
            let axis = direction.get_axis();
            let sign = direction.get_axis_direction();
            let (mut first, mut second) = if sign == AxisDirection::POSITIVE {
                (shape.clone(), occluder.clone())
            } else {
                (occluder.clone(), shape.clone())
            };
            if !DoubleMath::fuzzy_equals(first.max(axis), 1.0, 1.0E-7) {
                first = Shapes::empty();
            }

            if !DoubleMath::fuzzy_equals(second.min(axis), 0.0, 1.0E-7) {
                second = Shapes::empty();
            }

            let first_size = first.shape().get_size(axis);
            !Shapes::join_is_not_empty(
                &Shapes::block(),
                &Shapes::join_unoptimized(
                    &SliceShape::new(first, axis, first_size - 1),
                    &SliceShape::new(second, axis, 0),
                    BooleanOp::OR,
                ),
                BooleanOp::ONLY_FIRST,
            )
        } else {
            true
        }
    }

    /// Whether two face shapes together cover the block's face.
    pub fn face_shape_occludes(shape: &VoxelShape, occluder: &VoxelShape) -> bool {
        if shape == Shapes::block_ref() || occluder == Shapes::block_ref() {
            true
        } else if shape.is_empty() && occluder.is_empty() {
            false
        } else {
            !Shapes::join_is_not_empty(
                &Shapes::block(),
                &Shapes::join_unoptimized(shape, occluder, BooleanOp::OR),
                BooleanOp::ONLY_FIRST,
            )
        }
    }

    /// The merger for two shapes' coordinates along an axis: of cubes, when the merged grid has
    /// at most 256 voxels by `cost`; of lists one wholly below the other; of the same list; or
    /// any two.
    pub(crate) fn create_index_merger(
        cost: i32,
        first: Arc<dyn DoubleList>,
        second: Arc<dyn DoubleList>,
        first_only_matters: bool,
        second_only_matters: bool,
    ) -> Box<dyn IndexMerger> {
        let first_size = first.size() - 1;
        let second_size = second.size() - 1;
        if first.as_any().is::<CubePointRange>() && second.as_any().is::<CubePointRange>() {
            let size = Shapes::lcm(first_size, second_size);
            if i64::from(cost).wrapping_mul(size) <= 256 {
                return Box::new(DiscreteCubeMerger::new(first_size, second_size));
            }
        }

        if first.get_double(first_size) < second.get_double(0) - 1.0E-7 {
            Box::new(NonOverlappingMerger::new(first, second, false))
        } else if second.get_double(second_size) < first.get_double(0) - 1.0E-7 {
            Box::new(NonOverlappingMerger::new(second, first, true))
        } else if first_size == second_size && double_list::equals(&first, &second) {
            Box::new(IdenticalMerger::new(first))
        } else {
            Box::new(IndirectMerger::new(
                &*first,
                &*second,
                first_only_matters,
                second_only_matters,
            ))
        }
    }

    /// Whether the two shapes have the same voxels.
    pub fn equal(first: &VoxelShape, second: &VoxelShape) -> bool {
        !Shapes::join_is_not_empty(first, second, BooleanOp::NOT_SAME)
    }

    /// `BLOCK_CENTER`, which shapes turn about.
    pub const BLOCK_CENTER: Vec3 = Vec3::new(0.5, 0.5, 0.5);

    /// `rotate(VoxelShape, OctahedralGroup)`, about the block's center.
    pub fn rotate(shape: &VoxelShape, rotation: OctahedralGroup) -> VoxelShape {
        Shapes::rotate_about(shape, rotation, &Shapes::BLOCK_CENTER)
    }

    /// `rotate(VoxelShape, OctahedralGroup, Vec3)`: the voxels rotated, and each new axis's grid
    /// lines the old axis's, flipped about the point where the symmetry flips it. A cube turned
    /// about the block's center stays a cube; the identity hands the shape back.
    pub fn rotate_about(
        shape: &VoxelShape,
        rotation: OctahedralGroup,
        rotation_point: &Vec3,
    ) -> VoxelShape {
        if rotation == OctahedralGroup::IDENTITY {
            return shape.clone();
        }

        let new_discrete_shape = discrete_voxel_shape::rotate(shape.shape(), rotation);
        if matches!(shape.kind(), Kind::Cube(_)) && Shapes::BLOCK_CENTER == *rotation_point {
            return CubeVoxelShape::new(new_discrete_shape);
        }

        let permutation = rotation.permutation();
        let [new_x, new_y, new_z] = Axis::VALUES.map(|axis| permutation.permute_axis(axis));
        let flip = |new_axis: Axis, old_axis: Axis| {
            Shapes::flip_axis_if_needed(
                shape.get_coords(new_axis),
                rotation.inverts(old_axis),
                rotation_point.get(new_axis),
                rotation_point.get(old_axis),
            )
        };
        ArrayVoxelShape::new(
            new_discrete_shape,
            flip(new_x, Axis::X),
            flip(new_y, Axis::Y),
            flip(new_z, Axis::Z),
        )
    }

    /// The axis's grid lines moved from one point to the other, and reversed around it when the
    /// symmetry flips it: the same list when it does neither.
    pub fn flip_axis_if_needed(
        new_axis: Arc<dyn DoubleList>,
        flip: bool,
        new_relative: f64,
        old_relative: f64,
    ) -> Arc<dyn DoubleList> {
        if !flip && new_relative == old_relative {
            return new_axis;
        }

        let size = new_axis.size();
        let mut new_list = Vec::with_capacity(size as usize);
        if flip {
            for i in (0..size).rev() {
                new_list.push(-(new_axis.get_double(i) - new_relative) + old_relative);
            }
        } else {
            for i in 0..size {
                new_list.push(new_axis.get_double(i) - new_relative + old_relative);
            }
        }
        Arc::new(DoubleArrayList::wrap(Arc::from(new_list)))
    }

    /// `rotateHorizontalAxis(VoxelShape)`: the shape along z, and turned to x.
    pub fn rotate_horizontal_axis(z_axis: &VoxelShape) -> Vec<(Axis, VoxelShape)> {
        Shapes::rotate_horizontal_axis_about(z_axis, &Shapes::BLOCK_CENTER)
    }

    /// `rotateHorizontalAxis(VoxelShape, Vec3)`: turned about the point.
    pub fn rotate_horizontal_axis_about(
        z_axis: &VoxelShape,
        rotation_center: &Vec3,
    ) -> Vec<(Axis, VoxelShape)> {
        vec![
            (
                Axis::X,
                Shapes::rotate_about(z_axis, OctahedralGroup::BLOCK_ROT_Y_90, rotation_center),
            ),
            (Axis::Z, z_axis.clone()),
        ]
    }

    /// `rotateAllAxis(VoxelShape)`: the shape along z, and turned to x and to y.
    pub fn rotate_all_axis(north: &VoxelShape) -> Vec<(Axis, VoxelShape)> {
        Shapes::rotate_all_axis_about(north, &Shapes::BLOCK_CENTER)
    }

    /// `rotateAllAxis(VoxelShape, Vec3)`: turned about the point.
    pub fn rotate_all_axis_about(
        north: &VoxelShape,
        rotation_center: &Vec3,
    ) -> Vec<(Axis, VoxelShape)> {
        vec![
            (
                Axis::X,
                Shapes::rotate_about(north, OctahedralGroup::BLOCK_ROT_Y_90, rotation_center),
            ),
            (
                Axis::Y,
                Shapes::rotate_about(north, OctahedralGroup::BLOCK_ROT_X_90, rotation_center),
            ),
            (Axis::Z, north.clone()),
        ]
    }

    /// `rotateHorizontal(VoxelShape)`: the shape facing north, turned to face each way around.
    pub fn rotate_horizontal(north: &VoxelShape) -> Vec<(Direction, VoxelShape)> {
        Shapes::rotate_horizontal_from(north, OctahedralGroup::IDENTITY)
    }

    /// `rotateHorizontal(VoxelShape, OctahedralGroup)`: after the initial symmetry.
    pub fn rotate_horizontal_from(
        north: &VoxelShape,
        initial: OctahedralGroup,
    ) -> Vec<(Direction, VoxelShape)> {
        Shapes::rotate_horizontal_from_about(north, initial, &Shapes::BLOCK_CENTER)
    }

    /// `rotateHorizontal(VoxelShape, OctahedralGroup, Vec3)`: the other three turned about the
    /// point; north is the initial symmetry about the block's center.
    pub fn rotate_horizontal_from_about(
        north: &VoxelShape,
        initial: OctahedralGroup,
        rotation_center: &Vec3,
    ) -> Vec<(Direction, VoxelShape)> {
        vec![
            (Direction::NORTH, Shapes::rotate(north, initial)),
            (
                Direction::SOUTH,
                Shapes::rotate_about(
                    north,
                    OctahedralGroup::BLOCK_ROT_Y_180.compose(initial),
                    rotation_center,
                ),
            ),
            (
                Direction::WEST,
                Shapes::rotate_about(
                    north,
                    OctahedralGroup::BLOCK_ROT_Y_270.compose(initial),
                    rotation_center,
                ),
            ),
            (
                Direction::EAST,
                Shapes::rotate_about(
                    north,
                    OctahedralGroup::BLOCK_ROT_Y_90.compose(initial),
                    rotation_center,
                ),
            ),
        ]
    }

    /// `rotateAll(VoxelShape)`: the shape facing north, turned to face each of the six ways.
    pub fn rotate_all(north: &VoxelShape) -> Vec<(Direction, VoxelShape)> {
        Shapes::rotate_all_from_about(north, OctahedralGroup::IDENTITY, &Shapes::BLOCK_CENTER)
    }

    /// `rotateAll(VoxelShape, Vec3)`: turned about the point.
    pub fn rotate_all_about(
        north: &VoxelShape,
        rotation_center: &Vec3,
    ) -> Vec<(Direction, VoxelShape)> {
        Shapes::rotate_all_from_about(north, OctahedralGroup::IDENTITY, rotation_center)
    }

    /// `rotateAll(VoxelShape, OctahedralGroup, Vec3)`: after the initial symmetry, the other five
    /// turned about the point; north is the initial symmetry about the block's center.
    pub fn rotate_all_from_about(
        north: &VoxelShape,
        initial: OctahedralGroup,
        rotation_center: &Vec3,
    ) -> Vec<(Direction, VoxelShape)> {
        let turned = |rotation: OctahedralGroup| {
            Shapes::rotate_about(north, rotation.compose(initial), rotation_center)
        };
        vec![
            (Direction::DOWN, turned(OctahedralGroup::BLOCK_ROT_X_90)),
            (Direction::UP, turned(OctahedralGroup::BLOCK_ROT_X_270)),
            (Direction::NORTH, Shapes::rotate(north, initial)),
            (Direction::SOUTH, turned(OctahedralGroup::BLOCK_ROT_Y_180)),
            (Direction::WEST, turned(OctahedralGroup::BLOCK_ROT_Y_270)),
            (Direction::EAST, turned(OctahedralGroup::BLOCK_ROT_Y_90)),
        ]
    }

    /// `rotateAttachFace(VoxelShape)`: the shape on a wall facing north, turned onto the floor
    /// and the ceiling, each facing each way around.
    pub fn rotate_attach_face(
        north: &VoxelShape,
    ) -> Vec<(AttachFace, Vec<(Direction, VoxelShape)>)> {
        Shapes::rotate_attach_face_from(north, OctahedralGroup::IDENTITY)
    }

    /// `rotateAttachFace(VoxelShape, OctahedralGroup)`: after the initial symmetry.
    pub fn rotate_attach_face_from(
        north: &VoxelShape,
        initial: OctahedralGroup,
    ) -> Vec<(AttachFace, Vec<(Direction, VoxelShape)>)> {
        vec![
            (
                AttachFace::WALL,
                Shapes::rotate_horizontal_from(north, initial),
            ),
            (
                AttachFace::FLOOR,
                Shapes::rotate_horizontal_from(
                    north,
                    OctahedralGroup::BLOCK_ROT_X_270.compose(initial),
                ),
            ),
            (
                AttachFace::CEILING,
                Shapes::rotate_horizontal_from(
                    north,
                    OctahedralGroup::BLOCK_ROT_Y_180
                        .compose(OctahedralGroup::BLOCK_ROT_X_90)
                        .compose(initial),
                ),
            ),
        ]
    }

    /// What a map of rotated shapes has for the key: a shape, or the map of `rotateAttachFace`.
    pub fn get<K: PartialEq, V>(shapes: &[(K, V)], key: K) -> &V {
        &shapes
            .iter()
            .find(|(entry, _)| *entry == key)
            .expect("the map has every key")
            .1
    }
}

#[cfg(test)]
mod tests {
    use super::Shapes;
    use crate::com::mojang::math::octahedral_group::OctahedralGroup;
    use crate::golden::Golden;
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::golden::{
        MergerEncoding, Structure,
    };
    use crate::net::minecraft::world::phys::vec3::Vec3;

    const SHAPE: &str = "Lnet/minecraft/world/phys/shapes/VoxelShape;";

    fn method(name: &str, args: &[&str], result: &str) -> String {
        format!("{name}({}){result}", args.concat())
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.shapes.Shapes");
        golden.field("EPSILON", Shapes::EPSILON);
        golden.field("BIG_EPSILON", Shapes::BIG_EPSILON);
        golden.field("INFINITY", Structure(Shapes::infinity()));
        golden.check(&method("block", &[], SHAPE), || Structure(Shapes::block()));
        golden.check(&method("empty", &[], SHAPE), || Structure(Shapes::empty()));
        golden.check(&method("box", &["DDDDDD"], SHAPE), Shapes::r#box);
        golden.check(&method("create", &["DDDDDD"], SHAPE), Shapes::create);
        golden.check(
            &method("create", &["Lnet/minecraft/world/phys/AABB;"], SHAPE),
            |aabb: AABB| Shapes::create_aabb(&aabb),
        );
        golden.check(&method("Block.box", &["DDDDDD"], SHAPE), Block::r#box);
        golden.check("findBits(DD)I", Shapes::find_bits);
        golden.check("lcm(II)J", Shapes::lcm);
        golden.check(
            &method("or", &[SHAPE, SHAPE], SHAPE),
            |first: VoxelShape, second: VoxelShape| Shapes::or(&first, &second),
        );
        golden.check(
            &method("or", &[SHAPE, "[", SHAPE], SHAPE),
            |first: VoxelShape, tail: Vec<VoxelShape>| Shapes::or_tail(&first, &tail),
        );
        golden.check(
            &method("equal", &[SHAPE, SHAPE], "Z"),
            |first: VoxelShape, second: VoxelShape| Shapes::equal(&first, &second),
        );
        let op = "Lnet/minecraft/world/phys/shapes/BooleanOp;";
        golden.check(
            &method("joinIsNotEmpty", &[SHAPE, SHAPE, op], "Z"),
            |first: VoxelShape, second: VoxelShape, op: BooleanOp| {
                Shapes::join_is_not_empty(&first, &second, op)
            },
        );
        golden.check(
            &method("joinUnoptimized", &[SHAPE, SHAPE, op], SHAPE),
            |first: VoxelShape, second: VoxelShape, op: BooleanOp| {
                Shapes::join_unoptimized(&first, &second, op)
            },
        );
        golden.check(
            &method("join", &[SHAPE, SHAPE, op], SHAPE),
            |first: VoxelShape, second: VoxelShape, op: BooleanOp| {
                Shapes::join(&first, &second, op)
            },
        );
        golden.check(
            "joinUnoptimized(itself)",
            |shape: VoxelShape, op: BooleanOp| Shapes::join_unoptimized(&shape, &shape, op),
        );
        golden.check(
            "joinIsNotEmpty(itself)",
            |shape: VoxelShape, op: BooleanOp| Shapes::join_is_not_empty(&shape, &shape, op),
        );
        let direction = "Lnet/minecraft/core/Direction;";
        golden.check(
            &method("blockOccludes", &[SHAPE, SHAPE, direction], "Z"),
            |shape: VoxelShape, occluder: VoxelShape, direction: Direction| {
                Shapes::block_occludes(&shape, &occluder, direction)
            },
        );
        golden.check(
            &method("mergedFaceOccludes", &[SHAPE, SHAPE, direction], "Z"),
            |shape: VoxelShape, occluder: VoxelShape, direction: Direction| {
                Shapes::merged_face_occludes(&shape, &occluder, direction)
            },
        );
        golden.check(
            &method("faceShapeOccludes", &[SHAPE, SHAPE], "Z"),
            |shape: VoxelShape, occluder: VoxelShape| {
                Shapes::face_shape_occludes(&shape, &occluder)
            },
        );
        golden.check(
            "createIndexMerger(ILit/unimi/dsi/fastutil/doubles/DoubleList;Lit/unimi/dsi/fastutil/doubles/DoubleList;ZZ)",
            |cost, first: VoxelShape, second: VoxelShape, axis: Axis, first_only, second_only| {
                MergerEncoding(Shapes::create_index_merger(
                    cost,
                    first.get_coords(axis),
                    second.get_coords(axis),
                    first_only,
                    second_only,
                ))
            },
        );
        golden.check(
            "collide(Lnet/minecraft/core/Direction$Axis;Lnet/minecraft/world/phys/AABB;Ljava/lang/Iterable;D)D",
            |axis: Axis, moving: AABB, shapes: Vec<VoxelShape>, distance| {
                Shapes::collide(axis, &moving, &shapes, distance)
            },
        );
        let group = "Lcom/mojang/math/OctahedralGroup;";
        golden.check(
            &method("rotate", &[SHAPE, group], SHAPE),
            |shape: VoxelShape, rotation: OctahedralGroup| Shapes::rotate(&shape, rotation),
        );
        golden.check(
            &method(
                "rotate",
                &[SHAPE, group, "Lnet/minecraft/world/phys/Vec3;"],
                SHAPE,
            ),
            |shape: VoxelShape, rotation: OctahedralGroup, point: Vec3| {
                Shapes::rotate_about(&shape, rotation, &point)
            },
        );
        golden.check(
            &method("rotateHorizontal", &[SHAPE], "Ljava/util/Map;"),
            |shape: VoxelShape| Shapes::rotate_horizontal(&shape),
        );
        golden.check(
            &method("rotateHorizontal", &[SHAPE, group], "Ljava/util/Map;"),
            |shape: VoxelShape, initial: OctahedralGroup| {
                Shapes::rotate_horizontal_from(&shape, initial)
            },
        );
        golden.check(
            &method("rotateAll", &[SHAPE], "Ljava/util/Map;"),
            |shape: VoxelShape| Shapes::rotate_all(&shape),
        );
        golden.check(
            &method("rotateHorizontalAxis", &[SHAPE], "Ljava/util/Map;"),
            |shape: VoxelShape| Shapes::rotate_horizontal_axis(&shape),
        );
        golden.check(
            &method("rotateAllAxis", &[SHAPE], "Ljava/util/Map;"),
            |shape: VoxelShape| Shapes::rotate_all_axis(&shape),
        );
        let vec3 = "Lnet/minecraft/world/phys/Vec3;";
        golden.check(
            &method("rotateHorizontalAxis", &[SHAPE, vec3], "Ljava/util/Map;"),
            |shape: VoxelShape, point: Vec3| Shapes::rotate_horizontal_axis_about(&shape, &point),
        );
        golden.check(
            &method("rotateAllAxis", &[SHAPE, vec3], "Ljava/util/Map;"),
            |shape: VoxelShape, point: Vec3| Shapes::rotate_all_axis_about(&shape, &point),
        );
        golden.check(
            &method("rotateAll", &[SHAPE, vec3], "Ljava/util/Map;"),
            |shape: VoxelShape, point: Vec3| Shapes::rotate_all_about(&shape, &point),
        );
        golden.check(
            &method("rotateHorizontal", &[SHAPE, group, vec3], "Ljava/util/Map;"),
            |shape: VoxelShape, initial: OctahedralGroup, point: Vec3| {
                Shapes::rotate_horizontal_from_about(&shape, initial, &point)
            },
        );
        golden.check(
            &method("rotateAll", &[SHAPE, group, vec3], "Ljava/util/Map;"),
            |shape: VoxelShape, initial: OctahedralGroup, point: Vec3| {
                Shapes::rotate_all_from_about(&shape, initial, &point)
            },
        );
        // In `AttachFace`'s order: Java's map is a `Map.of`.
        let attach_faces = |maps: Vec<(AttachFace, Vec<(Direction, VoxelShape)>)>| {
            AttachFace::VALUES
                .map(|face| {
                    let shapes = &maps
                        .iter()
                        .find(|(key, _)| *key == face)
                        .expect("every face")
                        .1;
                    (format!("{face:?}"), shapes.clone())
                })
                .to_vec()
        };
        golden.check(
            &method("rotateAttachFace", &[SHAPE], "Ljava/util/Map;"),
            |shape: VoxelShape| attach_faces(Shapes::rotate_attach_face(&shape)),
        );
        golden.check(
            &method("rotateAttachFace", &[SHAPE, group], "Ljava/util/Map;"),
            |shape: VoxelShape, initial: OctahedralGroup| {
                attach_faces(Shapes::rotate_attach_face_from(&shape, initial))
            },
        );
        golden.finish();
    }
}
