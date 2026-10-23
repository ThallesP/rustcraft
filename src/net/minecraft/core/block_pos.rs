//! net/minecraft/core/BlockPos.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs, and
//! `breadthFirstTraversal`, which only the server's blocks call (`SpongeBlock`,
//! `CreakingHeartBlockEntity`).
//!
//! `BlockPos` extends `Vec3i` without adding state, so it holds its `Vec3i` and derefs to it, and
//! the methods it overrides return a `BlockPos`. `MutableBlockPos` extends `BlockPos` the same
//! way. Java's iterables hand out one `MutableBlockPos` over and over, which callers use at once
//! or copy with `immutable()`; here they're iterators of copies of it.

use std::fmt;
use std::iter;
use std::ops::Deref;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::math::Math;
use crate::net::minecraft::core::axis_cycle::AxisCycle;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::position::Position;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::rotation::Rotation;
use crate::net::minecraft::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// A block's position. Java's `equals` and `hashCode` are `Vec3i`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    vec3i: Vec3i,
}

impl Deref for BlockPos {
    type Target = Vec3i;

    fn deref(&self) -> &Vec3i {
        &self.vec3i
    }
}

// Packed in a long, like `asLong`.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, BlockPos> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        FriendlyByteBuf::write_block_pos_to,
        FriendlyByteBuf::read_block_pos_from,
    ))
});

impl BlockPos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, BlockPos> = &STREAM_CODEC;
    pub const ZERO: BlockPos = BlockPos::new(0, 0, 0);
    pub const PACKED_HORIZONTAL_LENGTH: i32 =
        1 + Mth::log2(Mth::smallest_encompassing_power_of_two(30000000));
    pub const PACKED_Y_LENGTH: i32 = 64 - 2 * BlockPos::PACKED_HORIZONTAL_LENGTH;
    const PACKED_X_MASK: i64 = (1 << BlockPos::PACKED_HORIZONTAL_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1 << BlockPos::PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1 << BlockPos::PACKED_HORIZONTAL_LENGTH) - 1;
    const Y_OFFSET: i32 = 0;
    const Z_OFFSET: i32 = BlockPos::PACKED_Y_LENGTH;
    const X_OFFSET: i32 = BlockPos::PACKED_Y_LENGTH + BlockPos::PACKED_HORIZONTAL_LENGTH;
    pub const MAX_HORIZONTAL_COORDINATE: i32 = (1 << BlockPos::PACKED_HORIZONTAL_LENGTH) / 2 - 1;

    pub const fn new(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos {
            vec3i: Vec3i::new(x, y, z),
        }
    }

    pub fn offset_i64(block_node: i64, offset: Direction) -> i64 {
        BlockPos::offset_i64_xyz(
            block_node,
            offset.get_step_x(),
            offset.get_step_y(),
            offset.get_step_z(),
        )
    }

    pub fn offset_i64_xyz(block_node: i64, step_x: i32, step_y: i32, step_z: i32) -> i64 {
        BlockPos::as_long_xyz(
            BlockPos::get_x_i64(block_node).wrapping_add(step_x),
            BlockPos::get_y_i64(block_node).wrapping_add(step_y),
            BlockPos::get_z_i64(block_node).wrapping_add(step_z),
        )
    }

    /// Each coordinate of a packed block is sign-extended from its bits.
    pub fn get_x_i64(block_node: i64) -> i32 {
        ((block_node << (64 - BlockPos::X_OFFSET - BlockPos::PACKED_HORIZONTAL_LENGTH))
            >> (64 - BlockPos::PACKED_HORIZONTAL_LENGTH)) as i32
    }

    pub fn get_y_i64(block_node: i64) -> i32 {
        ((block_node << (64 - BlockPos::PACKED_Y_LENGTH)) >> (64 - BlockPos::PACKED_Y_LENGTH))
            as i32
    }

    pub fn get_z_i64(block_node: i64) -> i32 {
        ((block_node << (64 - BlockPos::Z_OFFSET - BlockPos::PACKED_HORIZONTAL_LENGTH))
            >> (64 - BlockPos::PACKED_HORIZONTAL_LENGTH)) as i32
    }

    pub fn of(block_node: i64) -> BlockPos {
        BlockPos::new(
            BlockPos::get_x_i64(block_node),
            BlockPos::get_y_i64(block_node),
            BlockPos::get_z_i64(block_node),
        )
    }

    pub fn containing_xyz(x: f64, y: f64, z: f64) -> BlockPos {
        BlockPos::new(Mth::floor_f64(x), Mth::floor_f64(y), Mth::floor_f64(z))
    }

    pub fn containing(pos: &dyn Position) -> BlockPos {
        BlockPos::containing_xyz(pos.x(), pos.y(), pos.z())
    }

    pub fn min(a: &BlockPos, b: &BlockPos) -> BlockPos {
        BlockPos::new(
            a.get_x().min(b.get_x()),
            a.get_y().min(b.get_y()),
            a.get_z().min(b.get_z()),
        )
    }

    pub fn max(a: &BlockPos, b: &BlockPos) -> BlockPos {
        BlockPos::new(
            a.get_x().max(b.get_x()),
            a.get_y().max(b.get_y()),
            a.get_z().max(b.get_z()),
        )
    }

    pub fn as_long(&self) -> i64 {
        BlockPos::as_long_xyz(self.get_x(), self.get_y(), self.get_z())
    }

    /// Packs x and z into 26 bits each and y into 12, dropping the bits that don't fit.
    pub fn as_long_xyz(x: i32, y: i32, z: i32) -> i64 {
        let mut node = 0;
        node |= (i64::from(x) & BlockPos::PACKED_X_MASK) << BlockPos::X_OFFSET;
        node |= (i64::from(y) & BlockPos::PACKED_Y_MASK) << BlockPos::Y_OFFSET;
        node | ((i64::from(z) & BlockPos::PACKED_Z_MASK) << BlockPos::Z_OFFSET)
    }

    pub fn get_flat_index(neighbor_block_node: i64) -> i64 {
        neighbor_block_node & -16
    }

    pub fn offset_xyz(&self, x: i32, y: i32, z: i32) -> BlockPos {
        if x == 0 && y == 0 && z == 0 {
            *self
        } else {
            BlockPos::new(
                self.get_x().wrapping_add(x),
                self.get_y().wrapping_add(y),
                self.get_z().wrapping_add(z),
            )
        }
    }

    pub fn offset(&self, vec: &Vec3i) -> BlockPos {
        self.offset_xyz(vec.get_x(), vec.get_y(), vec.get_z())
    }

    pub fn subtract(&self, vec: &Vec3i) -> BlockPos {
        self.offset_xyz(
            vec.get_x().wrapping_neg(),
            vec.get_y().wrapping_neg(),
            vec.get_z().wrapping_neg(),
        )
    }

    pub fn multiply(&self, scale: i32) -> BlockPos {
        if scale == 1 {
            *self
        } else if scale == 0 {
            BlockPos::ZERO
        } else {
            BlockPos::new(
                self.get_x().wrapping_mul(scale),
                self.get_y().wrapping_mul(scale),
                self.get_z().wrapping_mul(scale),
            )
        }
    }

    pub fn above(&self) -> BlockPos {
        self.relative(Direction::UP)
    }

    pub fn above_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::UP, steps)
    }

    pub fn below(&self) -> BlockPos {
        self.relative(Direction::DOWN)
    }

    pub fn below_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::DOWN, steps)
    }

    pub fn north(&self) -> BlockPos {
        self.relative(Direction::NORTH)
    }

    pub fn north_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::NORTH, steps)
    }

    pub fn south(&self) -> BlockPos {
        self.relative(Direction::SOUTH)
    }

    pub fn south_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::SOUTH, steps)
    }

    pub fn west(&self) -> BlockPos {
        self.relative(Direction::WEST)
    }

    pub fn west_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::WEST, steps)
    }

    pub fn east(&self) -> BlockPos {
        self.relative(Direction::EAST)
    }

    pub fn east_steps(&self, steps: i32) -> BlockPos {
        self.relative_steps(Direction::EAST, steps)
    }

    pub fn relative(&self, direction: Direction) -> BlockPos {
        BlockPos::new(
            self.get_x().wrapping_add(direction.get_step_x()),
            self.get_y().wrapping_add(direction.get_step_y()),
            self.get_z().wrapping_add(direction.get_step_z()),
        )
    }

    pub fn relative_steps(&self, direction: Direction, steps: i32) -> BlockPos {
        if steps == 0 {
            *self
        } else {
            BlockPos::new(
                self.get_x()
                    .wrapping_add(direction.get_step_x().wrapping_mul(steps)),
                self.get_y()
                    .wrapping_add(direction.get_step_y().wrapping_mul(steps)),
                self.get_z()
                    .wrapping_add(direction.get_step_z().wrapping_mul(steps)),
            )
        }
    }

    pub fn relative_axis(&self, axis: Axis, steps: i32) -> BlockPos {
        if steps == 0 {
            return *self;
        }

        let x_step = if axis == Axis::X { steps } else { 0 };
        let y_step = if axis == Axis::Y { steps } else { 0 };
        let z_step = if axis == Axis::Z { steps } else { 0 };
        BlockPos::new(
            self.get_x().wrapping_add(x_step),
            self.get_y().wrapping_add(y_step),
            self.get_z().wrapping_add(z_step),
        )
    }

    pub fn rotate(&self, rotation: Rotation) -> BlockPos {
        match rotation {
            Rotation::CLOCKWISE_90 => {
                BlockPos::new(self.get_z().wrapping_neg(), self.get_y(), self.get_x())
            }
            Rotation::CLOCKWISE_180 => BlockPos::new(
                self.get_x().wrapping_neg(),
                self.get_y(),
                self.get_z().wrapping_neg(),
            ),
            Rotation::COUNTERCLOCKWISE_90 => {
                BlockPos::new(self.get_z(), self.get_y(), self.get_x().wrapping_neg())
            }
            Rotation::NONE => *self,
        }
    }

    pub fn cross(&self, up_vector: &Vec3i) -> BlockPos {
        BlockPos::new(
            self.get_y()
                .wrapping_mul(up_vector.get_z())
                .wrapping_sub(self.get_z().wrapping_mul(up_vector.get_y())),
            self.get_z()
                .wrapping_mul(up_vector.get_x())
                .wrapping_sub(self.get_x().wrapping_mul(up_vector.get_z())),
            self.get_x()
                .wrapping_mul(up_vector.get_y())
                .wrapping_sub(self.get_y().wrapping_mul(up_vector.get_x())),
        )
    }

    pub fn at_y(&self, y: i32) -> BlockPos {
        BlockPos::new(self.get_x(), y, self.get_z())
    }

    pub fn immutable(&self) -> BlockPos {
        *self
    }

    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos::from_xyz(self.get_x(), self.get_y(), self.get_z())
    }

    /// The lower bound is a float sum (`getX() + 1.0E-5F`), which rounds far from the origin;
    /// the upper one is a double minus `1.0E-5F` widened.
    pub fn clamp_location_within(&self, location: &Vec3) -> Vec3 {
        Vec3::new(
            Mth::clamp_f64(
                location.x,
                f64::from(self.get_x() as f32 + 1.0E-5_f32),
                f64::from(self.get_x()) + 1.0 - f64::from(1.0E-5_f32),
            ),
            Mth::clamp_f64(
                location.y,
                f64::from(self.get_y() as f32 + 1.0E-5_f32),
                f64::from(self.get_y()) + 1.0 - f64::from(1.0E-5_f32),
            ),
            Mth::clamp_f64(
                location.z,
                f64::from(self.get_z() as f32 + 1.0E-5_f32),
                f64::from(self.get_z()) + 1.0 - f64::from(1.0E-5_f32),
            ),
        )
    }

    pub fn random_in_cube(
        random: &mut dyn RandomSource,
        limit: i32,
        center: &BlockPos,
        size_to_scan_in_all_directions: i32,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::random_between_closed(
            random,
            limit,
            center.get_x().wrapping_sub(size_to_scan_in_all_directions),
            center.get_y().wrapping_sub(size_to_scan_in_all_directions),
            center.get_z().wrapping_sub(size_to_scan_in_all_directions),
            center.get_x().wrapping_add(size_to_scan_in_all_directions),
            center.get_y().wrapping_add(size_to_scan_in_all_directions),
            center.get_z().wrapping_add(size_to_scan_in_all_directions),
        )
    }

    /// Draws as it goes, so the random source moves on only as far as the caller iterates.
    #[allow(clippy::too_many_arguments)]
    pub fn random_between_closed(
        random: &mut dyn RandomSource,
        limit: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> impl Iterator<Item = BlockPos> {
        let width = max_x.wrapping_sub(min_x).wrapping_add(1);
        let height = max_y.wrapping_sub(min_y).wrapping_add(1);
        let depth = max_z.wrapping_sub(min_z).wrapping_add(1);
        let mut next_pos = MutableBlockPos::new();
        let mut counter = limit;
        iter::from_fn(move || {
            if counter <= 0 {
                return None;
            }

            // The cursor, handed out as the BlockPos it is.
            let next = **next_pos.set_xyz(
                min_x.wrapping_add(random.next_int_bounded(width)),
                min_y.wrapping_add(random.next_int_bounded(height)),
                min_z.wrapping_add(random.next_int_bounded(depth)),
            );
            counter -= 1;
            Some(next)
        })
    }

    pub fn within_manhattan(origin: &BlockPos, reach: i32) -> impl Iterator<Item = BlockPos> {
        BlockPos::manhattan_ordered(origin, reach, reach, reach, reach)
    }

    pub fn within_clipped_manhattan(
        origin: &BlockPos,
        reach_x: i32,
        reach_y: i32,
        reach_z: i32,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::manhattan_ordered(
            origin,
            reach_x,
            reach_y,
            reach_z,
            reach_x.max(reach_y).max(reach_z),
        )
    }

    pub fn within_box_by_manhattan_distance(
        origin: &BlockPos,
        reach_x: i32,
        reach_y: i32,
        reach_z: i32,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::manhattan_ordered(
            origin,
            reach_x,
            reach_y,
            reach_z,
            reach_x.wrapping_add(reach_y).wrapping_add(reach_z),
        )
    }

    /// Blocks by their Manhattan distance from the origin, up to `max_depth`. Each depth starts
    /// from the reach's first x and y, and a block off the origin's z is followed by its mirror.
    /// The first depth looks at the origin only, whatever the reaches.
    fn manhattan_ordered(
        origin: &BlockPos,
        reach_x: i32,
        reach_y: i32,
        reach_z: i32,
        max_depth: i32,
    ) -> impl Iterator<Item = BlockPos> {
        let origin_x = origin.get_x();
        let origin_y = origin.get_y();
        let origin_z = origin.get_z();
        let mut cursor = MutableBlockPos::new();
        let mut current_depth: i32 = 0;
        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut z_mirror = false;
        iter::from_fn(move || {
            if z_mirror {
                z_mirror = false;
                cursor.set_z(origin_z.wrapping_sub(cursor.get_z().wrapping_sub(origin_z)));
                return Some(*cursor);
            }

            let mut found = None;
            while found.is_none() {
                if y > max_y {
                    x = x.wrapping_add(1);
                    if x > max_x {
                        current_depth = current_depth.wrapping_add(1);
                        if current_depth > max_depth {
                            return None;
                        }

                        max_x = reach_x.min(current_depth);
                        x = max_x.wrapping_neg();
                    }

                    max_y = reach_y.min(current_depth.wrapping_sub(Math::abs_i32(x)));
                    y = max_y.wrapping_neg();
                }

                let xx = x;
                let yy = y;
                let zz = current_depth
                    .wrapping_sub(Math::abs_i32(xx))
                    .wrapping_sub(Math::abs_i32(yy));
                if zz <= reach_z {
                    z_mirror = zz != 0;
                    found = Some(**cursor.set_xyz(
                        origin_x.wrapping_add(xx),
                        origin_y.wrapping_add(yy),
                        origin_z.wrapping_add(zz),
                    ));
                }
                // The for loop's update, which runs after the block is found too.
                y = y.wrapping_add(1);
            }

            found
        })
    }

    /// The blocks containing the box's corners and every block between them. A max side on a
    /// block's face takes in the block past it.
    pub fn between_closed_aabb(r#box: &AABB) -> impl Iterator<Item = BlockPos> {
        let start_pos = BlockPos::containing_xyz(r#box.min_x, r#box.min_y, r#box.min_z);
        let end_pos = BlockPos::containing_xyz(r#box.max_x, r#box.max_y, r#box.max_z);
        BlockPos::between_closed(&start_pos, &end_pos)
    }

    /// Holds on to neither block, so the corners can be temporaries (`between_closed_aabb`'s).
    pub fn between_closed(a: &BlockPos, b: &BlockPos) -> impl Iterator<Item = BlockPos> + use<> {
        BlockPos::between_closed_xyz(
            a.get_x().min(b.get_x()),
            a.get_y().min(b.get_y()),
            a.get_z().min(b.get_z()),
            a.get_x().max(b.get_x()),
            a.get_y().max(b.get_y()),
            a.get_z().max(b.get_z()),
        )
    }

    /// Java's `Stream` of `betweenClosed`.
    pub fn between_closed_stream(a: &BlockPos, b: &BlockPos) -> impl Iterator<Item = BlockPos> {
        BlockPos::between_closed(a, b)
    }

    /// Like `between_closed_aabb`, without sorting the corners, which a box has in order already.
    pub fn between_closed_stream_aabb(r#box: &AABB) -> impl Iterator<Item = BlockPos> {
        BlockPos::between_closed_stream_xyz(
            Mth::floor_f64(r#box.min_x),
            Mth::floor_f64(r#box.min_y),
            Mth::floor_f64(r#box.min_z),
            Mth::floor_f64(r#box.max_x),
            Mth::floor_f64(r#box.max_y),
            Mth::floor_f64(r#box.max_z),
        )
    }

    /// The box's blocks, its corners in order again (a box has them in order already).
    pub fn between_closed_stream_bounding_box(
        bounding_box: &BoundingBox,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::between_closed_stream_xyz(
            bounding_box.min_x().min(bounding_box.max_x()),
            bounding_box.min_y().min(bounding_box.max_y()),
            bounding_box.min_z().min(bounding_box.max_z()),
            bounding_box.min_x().max(bounding_box.max_x()),
            bounding_box.min_y().max(bounding_box.max_y()),
            bounding_box.min_z().max(bounding_box.max_z()),
        )
    }

    pub fn between_closed_stream_xyz(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::between_closed_xyz(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Along x, then y, then z. Visits nothing when a side is 0 long (the product of the sides
    /// is where it ends, so it ends at once), and wraps like Java's ints when a side is
    /// negative.
    pub fn between_closed_xyz(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> impl Iterator<Item = BlockPos> {
        let width = max_x.wrapping_sub(min_x).wrapping_add(1);
        let height = max_y.wrapping_sub(min_y).wrapping_add(1);
        let depth = max_z.wrapping_sub(min_z).wrapping_add(1);
        let end = width.wrapping_mul(height).wrapping_mul(depth);
        let mut cursor = MutableBlockPos::new();
        let mut index: i32 = 0;
        iter::from_fn(move || {
            if index == end {
                return None;
            }

            let x = index.wrapping_rem(width);
            let slice = index.wrapping_div(width);
            let y = slice.wrapping_rem(height);
            let z = slice.wrapping_div(height);
            index = index.wrapping_add(1);
            Some(**cursor.set_xyz(
                min_x.wrapping_add(x),
                min_y.wrapping_add(y),
                min_z.wrapping_add(z),
            ))
        })
    }

    /// The column from `start_y` to `end_y`, then the same next to it to the north, east,
    /// south and west.
    pub fn neighbor_column(
        start_x: i32,
        start_y: i32,
        start_z: i32,
        end_y: i32,
    ) -> impl Iterator<Item = BlockPos> {
        let y_direction = if end_y > start_y { 1 } else { -1 };
        let height = Math::abs_i32(end_y.wrapping_sub(start_y)).wrapping_add(1);
        let steps = [
            Vec3i::new(0, 0, 0),
            Direction::NORTH.get_unit_vec3i(),
            Direction::EAST.get_unit_vec3i(),
            Direction::SOUTH.get_unit_vec3i(),
            Direction::WEST.get_unit_vec3i(),
        ];
        let step_count = (steps.len() as i32).wrapping_mul(height);
        let mut cursor = MutableBlockPos::new();
        let mut index: i32 = 0;
        iter::from_fn(move || {
            if index == step_count {
                return None;
            }

            let y = index.wrapping_rem(height);
            let step_index = index.wrapping_div(height);
            let step = steps[step_index as usize];
            index = index.wrapping_add(1);
            Some(**cursor.set_xyz(
                start_x.wrapping_add(step.get_x()),
                start_y.wrapping_add(y.wrapping_mul(y_direction)),
                start_z.wrapping_add(step.get_z()),
            ))
        })
    }

    /// The center, then a square spiral around it, legs of 1, 1, 2, 2, 3… blocks, out to
    /// `radius`. Panics when both directions are on one axis, where Java throws
    /// `IllegalStateException`.
    pub fn spiral_around(
        center: &BlockPos,
        radius: i32,
        first_direction: Direction,
        second_direction: Direction,
    ) -> impl Iterator<Item = MutableBlockPos> {
        assert!(
            first_direction.get_axis() != second_direction.get_axis(),
            "The two directions cannot be on the same axis"
        );
        let directions = [
            first_direction,
            second_direction,
            first_direction.get_opposite(),
            second_direction.get_opposite(),
        ];
        let mut cursor = center.mutable();
        cursor.r#move(second_direction);
        let legs = radius.wrapping_mul(4);
        let mut leg: i32 = -1;
        let mut leg_size: i32 = 0;
        let mut leg_index: i32 = 0;
        let mut last_x = cursor.get_x();
        let mut last_y = cursor.get_y();
        let mut last_z = cursor.get_z();
        iter::from_fn(move || {
            cursor
                .set_xyz(last_x, last_y, last_z)
                .r#move(directions[(leg.wrapping_add(4) % 4) as usize]);
            last_x = cursor.get_x();
            last_y = cursor.get_y();
            last_z = cursor.get_z();
            if leg_index >= leg_size {
                if leg >= legs {
                    return None;
                }

                leg = leg.wrapping_add(1);
                leg_index = 0;
                leg_size = leg / 2 + 1;
            }

            leg_index = leg_index.wrapping_add(1);
            Some(cursor)
        })
    }

    /// The blocks containing the box's corners and every block between them, in the order
    /// `between_corners_in_direction_xyz` walks them.
    pub fn between_corners_in_direction_aabb(
        aabb: &AABB,
        direction: &Vec3,
    ) -> impl Iterator<Item = BlockPos> {
        let min_corner = aabb.get_min_position();
        let first_corner_x = Mth::floor_f64(min_corner.x());
        let first_corner_y = Mth::floor_f64(min_corner.y());
        let first_corner_z = Mth::floor_f64(min_corner.z());
        let max_corner = aabb.get_max_position();
        let second_corner_x = Mth::floor_f64(max_corner.x());
        let second_corner_y = Mth::floor_f64(max_corner.y());
        let second_corner_z = Mth::floor_f64(max_corner.z());
        BlockPos::between_corners_in_direction_xyz(
            first_corner_x,
            first_corner_y,
            first_corner_z,
            second_corner_x,
            second_corner_y,
            second_corner_z,
            direction,
        )
    }

    pub fn between_corners_in_direction(
        first_corner: &BlockPos,
        second_corner: &BlockPos,
        direction: &Vec3,
    ) -> impl Iterator<Item = BlockPos> {
        BlockPos::between_corners_in_direction_xyz(
            first_corner.get_x(),
            first_corner.get_y(),
            first_corner.get_z(),
            second_corner.get_x(),
            second_corner.get_y(),
            second_corner.get_z(),
            direction,
        )
    }

    /// Every block of the box, starting from the corner `direction` comes from and moving
    /// along the axes in `Direction.axisStepOrder`'s order, the last one fastest. A NaN or -0.0
    /// component counts as negative and positive respectively, as Java's `>= 0.0` has it.
    pub fn between_corners_in_direction_xyz(
        first_corner_x: i32,
        first_corner_y: i32,
        first_corner_z: i32,
        second_corner_x: i32,
        second_corner_y: i32,
        second_corner_z: i32,
        direction: &Vec3,
    ) -> impl Iterator<Item = BlockPos> {
        let min_corner_x = first_corner_x.min(second_corner_x);
        let min_corner_y = first_corner_y.min(second_corner_y);
        let min_corner_z = first_corner_z.min(second_corner_z);
        let max_corner_x = first_corner_x.max(second_corner_x);
        let max_corner_y = first_corner_y.max(second_corner_y);
        let max_corner_z = first_corner_z.max(second_corner_z);
        let diff_x = max_corner_x.wrapping_sub(min_corner_x);
        let diff_y = max_corner_y.wrapping_sub(min_corner_y);
        let diff_z = max_corner_z.wrapping_sub(min_corner_z);
        let start_corner_x = if direction.x >= 0.0 {
            min_corner_x
        } else {
            max_corner_x
        };
        let start_corner_y = if direction.y >= 0.0 {
            min_corner_y
        } else {
            max_corner_y
        };
        let start_corner_z = if direction.z >= 0.0 {
            min_corner_z
        } else {
            max_corner_z
        };
        let axes = Direction::axis_step_order(direction);
        let first_visit_axis = axes[0];
        let second_visit_axis = axes[1];
        let third_visit_axis = axes[2];
        let first_visit_dir = if direction.get(first_visit_axis) >= 0.0 {
            first_visit_axis.get_positive()
        } else {
            first_visit_axis.get_negative()
        };
        let second_visit_dir = if direction.get(second_visit_axis) >= 0.0 {
            second_visit_axis.get_positive()
        } else {
            second_visit_axis.get_negative()
        };
        let third_visit_dir = if direction.get(third_visit_axis) >= 0.0 {
            third_visit_axis.get_positive()
        } else {
            third_visit_axis.get_negative()
        };
        let first_max = first_visit_axis.choose_i32(diff_x, diff_y, diff_z);
        let second_max = second_visit_axis.choose_i32(diff_x, diff_y, diff_z);
        let third_max = third_visit_axis.choose_i32(diff_x, diff_y, diff_z);

        let mut cursor = MutableBlockPos::new();
        let mut first_index: i32 = 0;
        let mut second_index: i32 = 0;
        let mut third_index: i32 = 0;
        let mut end = false;
        let first_dir_x = first_visit_dir.get_step_x();
        let first_dir_y = first_visit_dir.get_step_y();
        let first_dir_z = first_visit_dir.get_step_z();
        let second_dir_x = second_visit_dir.get_step_x();
        let second_dir_y = second_visit_dir.get_step_y();
        let second_dir_z = second_visit_dir.get_step_z();
        let third_dir_x = third_visit_dir.get_step_x();
        let third_dir_y = third_visit_dir.get_step_y();
        let third_dir_z = third_visit_dir.get_step_z();
        iter::from_fn(move || {
            if end {
                return None;
            }

            cursor.set_xyz(
                start_corner_x
                    .wrapping_add(first_dir_x.wrapping_mul(first_index))
                    .wrapping_add(second_dir_x.wrapping_mul(second_index))
                    .wrapping_add(third_dir_x.wrapping_mul(third_index)),
                start_corner_y
                    .wrapping_add(first_dir_y.wrapping_mul(first_index))
                    .wrapping_add(second_dir_y.wrapping_mul(second_index))
                    .wrapping_add(third_dir_y.wrapping_mul(third_index)),
                start_corner_z
                    .wrapping_add(first_dir_z.wrapping_mul(first_index))
                    .wrapping_add(second_dir_z.wrapping_mul(second_index))
                    .wrapping_add(third_dir_z.wrapping_mul(third_index)),
            );
            if third_index < third_max {
                third_index += 1;
            } else if second_index < second_max {
                second_index += 1;
                third_index = 0;
            } else if first_index < first_max {
                first_index += 1;
                third_index = 0;
                second_index = 0;
            } else {
                end = true;
            }

            Some(*cursor)
        })
    }
}

/// Java's `toString`, with `BlockPos` as the class's name.
/// `Vec3i.hashCode`, for the Java maps of positions (a chunk's block entities).
impl crate::java::lang::object::HashCode for BlockPos {
    fn hash_code(&self) -> i32 {
        Vec3i::hash_code(self)
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.vec3i.fmt_with_class_name(f, "BlockPos")
    }
}

/// `BlockPos.MutableBlockPos`: a cursor the game moves around instead of making new positions.
/// The methods it overrides return a copy, never the cursor itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    block_pos: BlockPos,
}

impl Deref for MutableBlockPos {
    type Target = BlockPos;

    fn deref(&self) -> &BlockPos {
        &self.block_pos
    }
}

impl Default for MutableBlockPos {
    fn default() -> MutableBlockPos {
        MutableBlockPos::new()
    }
}

impl MutableBlockPos {
    pub const fn new() -> MutableBlockPos {
        MutableBlockPos::from_xyz(0, 0, 0)
    }

    pub const fn from_xyz(x: i32, y: i32, z: i32) -> MutableBlockPos {
        MutableBlockPos {
            block_pos: BlockPos::new(x, y, z),
        }
    }

    pub fn from_xyz_f64(x: f64, y: f64, z: f64) -> MutableBlockPos {
        MutableBlockPos::from_xyz(Mth::floor_f64(x), Mth::floor_f64(y), Mth::floor_f64(z))
    }

    pub fn offset_xyz(&self, x: i32, y: i32, z: i32) -> BlockPos {
        self.block_pos.offset_xyz(x, y, z).immutable()
    }

    pub fn multiply(&self, scale: i32) -> BlockPos {
        self.block_pos.multiply(scale).immutable()
    }

    pub fn relative_steps(&self, direction: Direction, steps: i32) -> BlockPos {
        self.block_pos.relative_steps(direction, steps).immutable()
    }

    pub fn relative_axis(&self, axis: Axis, steps: i32) -> BlockPos {
        self.block_pos.relative_axis(axis, steps).immutable()
    }

    pub fn rotate(&self, rotation: Rotation) -> BlockPos {
        self.block_pos.rotate(rotation).immutable()
    }

    pub fn set_xyz(&mut self, x: i32, y: i32, z: i32) -> &mut MutableBlockPos {
        self.set_x(x);
        self.set_y(y);
        self.set_z(z);
        self
    }

    pub fn set_xyz_f64(&mut self, x: f64, y: f64, z: f64) -> &mut MutableBlockPos {
        self.set_xyz(Mth::floor_f64(x), Mth::floor_f64(y), Mth::floor_f64(z))
    }

    pub fn set(&mut self, vec: &Vec3i) -> &mut MutableBlockPos {
        self.set_xyz(vec.get_x(), vec.get_y(), vec.get_z())
    }

    pub fn set_i64(&mut self, pos: i64) -> &mut MutableBlockPos {
        self.set_xyz(
            BlockPos::get_x_i64(pos),
            BlockPos::get_y_i64(pos),
            BlockPos::get_z_i64(pos),
        )
    }

    pub fn set_axis_cycle(
        &mut self,
        transform: AxisCycle,
        x: i32,
        y: i32,
        z: i32,
    ) -> &mut MutableBlockPos {
        self.set_xyz(
            transform.cycle_i32(x, y, z, Axis::X),
            transform.cycle_i32(x, y, z, Axis::Y),
            transform.cycle_i32(x, y, z, Axis::Z),
        )
    }

    pub fn set_with_offset(&mut self, pos: &Vec3i, direction: Direction) -> &mut MutableBlockPos {
        self.set_xyz(
            pos.get_x().wrapping_add(direction.get_step_x()),
            pos.get_y().wrapping_add(direction.get_step_y()),
            pos.get_z().wrapping_add(direction.get_step_z()),
        )
    }

    pub fn set_with_offset_xyz(
        &mut self,
        pos: &Vec3i,
        x: i32,
        y: i32,
        z: i32,
    ) -> &mut MutableBlockPos {
        self.set_xyz(
            pos.get_x().wrapping_add(x),
            pos.get_y().wrapping_add(y),
            pos.get_z().wrapping_add(z),
        )
    }

    pub fn set_with_offset_vec3i(&mut self, pos: &Vec3i, offset: &Vec3i) -> &mut MutableBlockPos {
        self.set_xyz(
            pos.get_x().wrapping_add(offset.get_x()),
            pos.get_y().wrapping_add(offset.get_y()),
            pos.get_z().wrapping_add(offset.get_z()),
        )
    }

    /// Java's `move`, a Rust keyword.
    pub fn r#move(&mut self, direction: Direction) -> &mut MutableBlockPos {
        self.move_steps(direction, 1)
    }

    pub fn move_steps(&mut self, direction: Direction, steps: i32) -> &mut MutableBlockPos {
        self.set_xyz(
            self.get_x()
                .wrapping_add(direction.get_step_x().wrapping_mul(steps)),
            self.get_y()
                .wrapping_add(direction.get_step_y().wrapping_mul(steps)),
            self.get_z()
                .wrapping_add(direction.get_step_z().wrapping_mul(steps)),
        )
    }

    pub fn move_xyz(&mut self, x: i32, y: i32, z: i32) -> &mut MutableBlockPos {
        self.set_xyz(
            self.get_x().wrapping_add(x),
            self.get_y().wrapping_add(y),
            self.get_z().wrapping_add(z),
        )
    }

    pub fn move_vec3i(&mut self, pos: &Vec3i) -> &mut MutableBlockPos {
        self.set_xyz(
            self.get_x().wrapping_add(pos.get_x()),
            self.get_y().wrapping_add(pos.get_y()),
            self.get_z().wrapping_add(pos.get_z()),
        )
    }

    pub fn clamp(&mut self, axis: Axis, minimum: i32, maximum: i32) -> &mut MutableBlockPos {
        match axis {
            Axis::X => self.set_xyz(
                Mth::clamp_i32(self.get_x(), minimum, maximum),
                self.get_y(),
                self.get_z(),
            ),
            Axis::Y => self.set_xyz(
                self.get_x(),
                Mth::clamp_i32(self.get_y(), minimum, maximum),
                self.get_z(),
            ),
            Axis::Z => self.set_xyz(
                self.get_x(),
                self.get_y(),
                Mth::clamp_i32(self.get_z(), minimum, maximum),
            ),
        }
    }

    pub fn set_x(&mut self, x: i32) -> &mut MutableBlockPos {
        self.block_pos.vec3i.set_x(x);
        self
    }

    pub fn set_y(&mut self, y: i32) -> &mut MutableBlockPos {
        self.block_pos.vec3i.set_y(y);
        self
    }

    pub fn set_z(&mut self, z: i32) -> &mut MutableBlockPos {
        self.block_pos.vec3i.set_z(z);
        self
    }

    pub fn immutable(&self) -> BlockPos {
        BlockPos::new(self.get_x(), self.get_y(), self.get_z())
    }
}

/// Java's `toString`, with `MutableBlockPos` as the class's name.
impl fmt::Display for MutableBlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.block_pos
            .vec3i
            .fmt_with_class_name(f, "MutableBlockPos")
    }
}

/// `BlockPos.TraversalNodeStatus`, what `breadthFirstTraversal`'s node processor answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TraversalNodeStatus {
    ACCEPT,
    SKIP,
    STOP,
}

impl TraversalNodeStatus {
    /// `values()`: the constants in declaration order, which is also their ordinal.
    pub const VALUES: [TraversalNodeStatus; 3] = [
        TraversalNodeStatus::ACCEPT,
        TraversalNodeStatus::SKIP,
        TraversalNodeStatus::STOP,
    ];

    pub fn values() -> [TraversalNodeStatus; 3] {
        TraversalNodeStatus::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> TraversalNodeStatus {
        match name {
            "ACCEPT" => TraversalNodeStatus::ACCEPT,
            "SKIP" => TraversalNodeStatus::SKIP,
            "STOP" => TraversalNodeStatus::STOP,
            _ => {
                panic!("IllegalArgumentException: no BlockPos.TraversalNodeStatus constant {name}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{BlockPos, MutableBlockPos, TraversalNodeStatus};
    use crate::golden::{Decode, Encode, Golden, STREAM_LIMIT, encode_names};
    use crate::net::minecraft::core::axis_cycle::AxisCycle;
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::core::vec3i::Vec3i;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::level::block::rotation::Rotation;
    use crate::net::minecraft::world::level::levelgen::structure::bounding_box::BoundingBox;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    // Both are [x, y, z], like the Vec3i they hold.
    impl Decode for BlockPos {
        fn decode(value: &Value) -> Self {
            BlockPos {
                vec3i: Vec3i::decode(value),
            }
        }
    }

    impl Encode for BlockPos {
        fn encode(&self) -> Value {
            self.vec3i.encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.vec3i.matches(expected)
        }
    }

    impl Decode for MutableBlockPos {
        fn decode(value: &Value) -> Self {
            MutableBlockPos {
                block_pos: BlockPos::decode(value),
            }
        }
    }

    impl Encode for MutableBlockPos {
        fn encode(&self) -> Value {
            self.block_pos.encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.block_pos.matches(expected)
        }
    }

    encode_names!(TraversalNodeStatus { ACCEPT, SKIP, STOP });

    /// What vanilla records of an iterable or a stream: its first elements, as they come.
    fn first<T>(elements: impl Iterator<Item = T>) -> Vec<T> {
        elements.take(STREAM_LIMIT).collect()
    }

    #[test]
    fn block_pos_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.BlockPos");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.check_codec("STREAM_CODEC", || BlockPos::STREAM_CODEC);
        golden.check(
            "betweenClosedStream(Lnet/minecraft/world/level/levelgen/structure/BoundingBox;)Ljava/util/stream/Stream;",
            |bounding_box: BoundingBox| {
                first(BlockPos::between_closed_stream_bounding_box(&bounding_box))
            },
        );
        golden.pending(
            "breadthFirstTraversal(Lnet/minecraft/core/BlockPos;IILjava/util/function/BiConsumer;Ljava/util/function/Function;)I",
            "only the server's blocks call it (SpongeBlock, CreakingHeartBlockEntity), and its callbacks have no generator",
        );
        golden.field("ZERO", BlockPos::ZERO);
        golden.field(
            "PACKED_HORIZONTAL_LENGTH",
            BlockPos::PACKED_HORIZONTAL_LENGTH,
        );
        golden.field("PACKED_Y_LENGTH", BlockPos::PACKED_Y_LENGTH);
        golden.field(
            "MAX_HORIZONTAL_COORDINATE",
            BlockPos::MAX_HORIZONTAL_COORDINATE,
        );
        golden.check("<init>(III)V", BlockPos::new);
        golden.check(
            "offset(JLnet/minecraft/core/Direction;)J",
            BlockPos::offset_i64,
        );
        golden.check("offset(JIII)J", BlockPos::offset_i64_xyz);
        golden.check("getX(J)I", BlockPos::get_x_i64);
        golden.check("getY(J)I", BlockPos::get_y_i64);
        golden.check("getZ(J)I", BlockPos::get_z_i64);
        golden.check("of(J)Lnet/minecraft/core/BlockPos;", BlockPos::of);
        golden.check(
            "containing(DDD)Lnet/minecraft/core/BlockPos;",
            BlockPos::containing_xyz,
        );
        golden.check(
            "containing(Lnet/minecraft/core/Position;)Lnet/minecraft/core/BlockPos;",
            |pos: Vec3| BlockPos::containing(&pos),
        );
        golden.check(
            "min(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/core/BlockPos;",
            |a: BlockPos, b: BlockPos| BlockPos::min(&a, &b),
        );
        golden.check(
            "max(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/core/BlockPos;",
            |a: BlockPos, b: BlockPos| BlockPos::max(&a, &b),
        );
        golden.check_method("asLong()J", |this: &mut BlockPos| this.as_long());
        golden.check("asLong(III)J", BlockPos::as_long_xyz);
        golden.check("getFlatIndex(J)J", BlockPos::get_flat_index);
        golden.check_method(
            "offset(III)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, x, y, z| this.offset_xyz(x, y, z),
        );
        golden.check_method(
            "offset(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, vec: Vec3i| this.offset(&vec),
        );
        golden.check_method(
            "subtract(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, vec: Vec3i| this.subtract(&vec),
        );
        golden.check_method(
            "multiply(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, scale| this.multiply(scale),
        );
        golden.check_method(
            "above()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.above(),
        );
        golden.check_method(
            "above(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.above_steps(steps),
        );
        golden.check_method(
            "below()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.below(),
        );
        golden.check_method(
            "below(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.below_steps(steps),
        );
        golden.check_method(
            "north()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.north(),
        );
        golden.check_method(
            "north(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.north_steps(steps),
        );
        golden.check_method(
            "south()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.south(),
        );
        golden.check_method(
            "south(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.south_steps(steps),
        );
        golden.check_method(
            "west()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.west(),
        );
        golden.check_method(
            "west(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.west_steps(steps),
        );
        golden.check_method(
            "east()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.east(),
        );
        golden.check_method(
            "east(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, steps| this.east_steps(steps),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, direction: Direction| this.relative(direction),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, direction: Direction, steps| {
                this.relative_steps(direction, steps)
            },
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction$Axis;I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, axis: Axis, steps| this.relative_axis(axis, steps),
        );
        golden.check_method(
            "rotate(Lnet/minecraft/world/level/block/Rotation;)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, rotation: Rotation| this.rotate(rotation),
        );
        golden.check_method(
            "cross(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, up_vector: Vec3i| this.cross(&up_vector),
        );
        golden.check_method(
            "atY(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos, y| this.at_y(y),
        );
        golden.check_method(
            "immutable()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockPos| this.immutable(),
        );
        golden.check_method(
            "mutable()Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut BlockPos| this.mutable(),
        );
        golden.check_method(
            "clampLocationWithin(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut BlockPos, location: Vec3| this.clamp_location_within(&location),
        );
        golden.check_random(
            "randomInCube(Lnet/minecraft/util/RandomSource;ILnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;",
            |random: &mut dyn RandomSource, limit, center: BlockPos, size| {
                first(BlockPos::random_in_cube(random, limit, &center, size))
            },
        );
        golden.check_random(
            "randomBetweenClosed(Lnet/minecraft/util/RandomSource;IIIIIII)Ljava/lang/Iterable;",
            |random: &mut dyn RandomSource, limit, min_x, min_y, min_z, max_x, max_y, max_z| {
                first(BlockPos::random_between_closed(
                    random, limit, min_x, min_y, min_z, max_x, max_y, max_z,
                ))
            },
        );
        golden.check(
            "withinManhattan(Lnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;",
            |origin: BlockPos, reach| first(BlockPos::within_manhattan(&origin, reach)),
        );
        golden.check(
            "withinClippedManhattan(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;",
            |origin: BlockPos, reach_x, reach_y, reach_z| {
                first(BlockPos::within_clipped_manhattan(
                    &origin, reach_x, reach_y, reach_z,
                ))
            },
        );
        golden.check(
            "withinBoxByManhattanDistance(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;",
            |origin: BlockPos, reach_x, reach_y, reach_z| {
                first(BlockPos::within_box_by_manhattan_distance(
                    &origin, reach_x, reach_y, reach_z,
                ))
            },
        );
        golden.check(
            "betweenClosed(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/lang/Iterable;",
            |a: BlockPos, b: BlockPos| first(BlockPos::between_closed(&a, &b)),
        );
        golden.check(
            "betweenClosed(Lnet/minecraft/world/phys/AABB;)Ljava/lang/Iterable;",
            |r#box: AABB| first(BlockPos::between_closed_aabb(&r#box)),
        );
        golden.check(
            "betweenClosedStream(Lnet/minecraft/world/phys/AABB;)Ljava/util/stream/Stream;",
            |r#box: AABB| first(BlockPos::between_closed_stream_aabb(&r#box)),
        );
        golden.check(
            "betweenCornersInDirection(Lnet/minecraft/world/phys/AABB;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;",
            |aabb: AABB, direction: Vec3| {
                first(BlockPos::between_corners_in_direction_aabb(&aabb, &direction))
            },
        );
        golden.check(
            "betweenClosedStream(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/util/stream/Stream;",
            |a: BlockPos, b: BlockPos| first(BlockPos::between_closed_stream(&a, &b)),
        );
        golden.check(
            "betweenClosedStream(IIIIII)Ljava/util/stream/Stream;",
            |min_x, min_y, min_z, max_x, max_y, max_z| {
                first(BlockPos::between_closed_stream_xyz(
                    min_x, min_y, min_z, max_x, max_y, max_z,
                ))
            },
        );
        golden.check(
            "betweenClosed(IIIIII)Ljava/lang/Iterable;",
            |min_x, min_y, min_z, max_x, max_y, max_z| {
                first(BlockPos::between_closed_xyz(
                    min_x, min_y, min_z, max_x, max_y, max_z,
                ))
            },
        );
        golden.check(
            "neighborColumn(IIII)Ljava/lang/Iterable;",
            |start_x, start_y, start_z, end_y| {
                first(BlockPos::neighbor_column(start_x, start_y, start_z, end_y))
            },
        );
        golden.check(
            "spiralAround(Lnet/minecraft/core/BlockPos;ILnet/minecraft/core/Direction;Lnet/minecraft/core/Direction;)Ljava/lang/Iterable;",
            |center: BlockPos, radius, first_direction, second_direction| {
                first(BlockPos::spiral_around(
                    &center,
                    radius,
                    first_direction,
                    second_direction,
                ))
            },
        );
        golden.check(
            "betweenCornersInDirection(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;",
            |first_corner: BlockPos, second_corner: BlockPos, direction: Vec3| {
                first(BlockPos::between_corners_in_direction(
                    &first_corner,
                    &second_corner,
                    &direction,
                ))
            },
        );
        golden.check(
            "betweenCornersInDirection(IIIIIILnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;",
            |first_corner_x,
             first_corner_y,
             first_corner_z,
             second_corner_x,
             second_corner_y,
             second_corner_z,
             direction: Vec3| {
                first(BlockPos::between_corners_in_direction_xyz(
                    first_corner_x,
                    first_corner_y,
                    first_corner_z,
                    second_corner_x,
                    second_corner_y,
                    second_corner_z,
                    &direction,
                ))
            },
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut BlockPos| {
            this.to_string()
        });
        golden.finish();
    }

    #[test]
    fn mutable_block_pos_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.BlockPos$MutableBlockPos");
        golden.check("<init>()V", MutableBlockPos::new);
        golden.check("<init>(III)V", MutableBlockPos::from_xyz);
        golden.check("<init>(DDD)V", MutableBlockPos::from_xyz_f64);
        golden.check_method(
            "offset(III)Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos, x, y, z| this.offset_xyz(x, y, z),
        );
        golden.check_method(
            "multiply(I)Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos, scale| this.multiply(scale),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;I)Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos, direction: Direction, steps| {
                this.relative_steps(direction, steps)
            },
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction$Axis;I)Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos, axis: Axis, steps| this.relative_axis(axis, steps),
        );
        golden.check_method(
            "rotate(Lnet/minecraft/world/level/block/Rotation;)Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos, rotation: Rotation| this.rotate(rotation),
        );
        golden.check_method(
            "set(III)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, x, y, z| *this.set_xyz(x, y, z),
        );
        golden.check_method(
            "set(DDD)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, x, y, z| *this.set_xyz_f64(x, y, z),
        );
        golden.check_method(
            "set(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, vec: Vec3i| *this.set(&vec),
        );
        golden.check_method(
            "set(J)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, pos| *this.set_i64(pos),
        );
        golden.check_method(
            "set(Lnet/minecraft/core/AxisCycle;III)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, transform: AxisCycle, x, y, z| {
                *this.set_axis_cycle(transform, x, y, z)
            },
        );
        golden.check_method(
            "setWithOffset(Lnet/minecraft/core/Vec3i;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, pos: Vec3i, direction: Direction| {
                *this.set_with_offset(&pos, direction)
            },
        );
        golden.check_method(
            "setWithOffset(Lnet/minecraft/core/Vec3i;III)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, pos: Vec3i, x, y, z| {
                *this.set_with_offset_xyz(&pos, x, y, z)
            },
        );
        golden.check_method(
            "setWithOffset(Lnet/minecraft/core/Vec3i;Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, pos: Vec3i, offset: Vec3i| {
                *this.set_with_offset_vec3i(&pos, &offset)
            },
        );
        golden.check_method(
            "move(Lnet/minecraft/core/Direction;)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, direction: Direction| *this.r#move(direction),
        );
        golden.check_method(
            "move(Lnet/minecraft/core/Direction;I)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, direction: Direction, steps| {
                *this.move_steps(direction, steps)
            },
        );
        golden.check_method(
            "move(III)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, x, y, z| *this.move_xyz(x, y, z),
        );
        golden.check_method(
            "move(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, pos: Vec3i| *this.move_vec3i(&pos),
        );
        golden.check_method(
            "clamp(Lnet/minecraft/core/Direction$Axis;II)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, axis: Axis, minimum, maximum| {
                *this.clamp(axis, minimum, maximum)
            },
        );
        golden.check_method(
            "setX(I)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, x| *this.set_x(x),
        );
        golden.check_method(
            "setY(I)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, y| *this.set_y(y),
        );
        golden.check_method(
            "setZ(I)Lnet/minecraft/core/BlockPos$MutableBlockPos;",
            |this: &mut MutableBlockPos, z| *this.set_z(z),
        );
        golden.check_method(
            "immutable()Lnet/minecraft/core/BlockPos;",
            |this: &mut MutableBlockPos| this.immutable(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut MutableBlockPos| this.to_string(),
        );
        golden.finish();
    }

    #[test]
    fn traversal_node_status_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.BlockPos$TraversalNodeStatus");
        for status in TraversalNodeStatus::VALUES {
            golden.field(&format!("{status:?}"), status);
        }
        golden.check(
            "values()[Lnet/minecraft/core/BlockPos$TraversalNodeStatus;",
            TraversalNodeStatus::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/BlockPos$TraversalNodeStatus;",
            |name: String| TraversalNodeStatus::value_of(&name),
        );
        golden.finish();
    }
}
