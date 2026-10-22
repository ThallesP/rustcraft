//! net/minecraft/core/Direction.java
//!
//! Not ported yet: the codecs (`CODEC`, `VERTICAL_CODEC`, `LEGACY_ID_CODEC`, `LEGACY_ID_CODEC_2D`),
//! which wait for DataFixerUpper, and `orderedByNearest` and `getFacingAxis`, which wait for
//! `Entity`.
//!
//! Each constant's constructor arguments, and the vectors it makes of its normal, live in a
//! table indexed by the constant. `Axis`, `AxisDirection` and `Plane` are its nested enums.

use std::f64::consts::PI;
use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::r#enum::Enum;
use crate::java::lang::math::Math;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// One of the six faces of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    DOWN,
    UP,
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

/// What a constant's constructor takes, and the vectors it makes of the normal.
struct Data {
    data3d: i32,
    opposite_index: i32,
    data2d: i32,
    name: &'static str,
    axis: Axis,
    axis_direction: AxisDirection,
    normal: Vec3i,
    normal_vec3: Vec3,
    normal_vec3f: Vector3f,
}

impl Data {
    const fn new(
        data3d: i32,
        opposite_index: i32,
        data2d: i32,
        name: &'static str,
        axis_direction: AxisDirection,
        axis: Axis,
        normal: Vec3i,
    ) -> Data {
        Data {
            data3d,
            opposite_index,
            data2d,
            name,
            axis,
            axis_direction,
            normal,
            normal_vec3: Vec3::at_lower_corner_of(&normal),
            normal_vec3f: Vector3f::from_xyz(
                normal.get_x() as f32,
                normal.get_y() as f32,
                normal.get_z() as f32,
            ),
        }
    }
}

// One line per constant, like Java's.
#[rustfmt::skip]
static DATA: [Data; 6] = [
    Data::new(0, 1, -1, "down", AxisDirection::NEGATIVE, Axis::Y, Vec3i::new(0, -1, 0)),
    Data::new(1, 0, -1, "up", AxisDirection::POSITIVE, Axis::Y, Vec3i::new(0, 1, 0)),
    Data::new(2, 3, 2, "north", AxisDirection::NEGATIVE, Axis::Z, Vec3i::new(0, 0, -1)),
    Data::new(3, 2, 0, "south", AxisDirection::POSITIVE, Axis::Z, Vec3i::new(0, 0, 1)),
    Data::new(4, 5, 1, "west", AxisDirection::NEGATIVE, Axis::X, Vec3i::new(-1, 0, 0)),
    Data::new(5, 4, 3, "east", AxisDirection::POSITIVE, Axis::X, Vec3i::new(1, 0, 0)),
];

const YXZ_AXIS_ORDER: [Axis; 3] = [Axis::Y, Axis::X, Axis::Z];
const YZX_AXIS_ORDER: [Axis; 3] = [Axis::Y, Axis::Z, Axis::X];

static BY_3D_DATA: LazyLock<Vec<Direction>> = LazyLock::new(|| {
    let mut directions = Direction::VALUES.to_vec();
    directions.sort_by_key(|direction| direction.data().data3d);
    directions
});

static BY_2D_DATA: LazyLock<Vec<Direction>> = LazyLock::new(|| {
    let mut directions: Vec<Direction> = Direction::VALUES
        .into_iter()
        .filter(|direction| direction.get_axis().is_horizontal())
        .collect();
    directions.sort_by_key(|direction| direction.data().data2d);
    directions
});

// By 3D data value, wrapping around.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> Direction + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        Direction::get_3d_data_value,
        &Direction::VALUES,
        OutOfBoundsStrategy::WRAP,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, Direction> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::id_mapper(
        Direction::BY_ID.as_ref(),
        Direction::get_3d_data_value,
    ))
});

impl Direction {
    /// Java's `IntFunction<Direction>`.
    pub const BY_ID: &'static LazyLock<Box<dyn Fn(i32) -> Direction + Send + Sync>> = &BY_ID;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Direction> = &STREAM_CODEC;
    /// `values()`: the constants in declaration order, which is also their ordinal.
    pub const VALUES: [Direction; 6] = [
        Direction::DOWN,
        Direction::UP,
        Direction::NORTH,
        Direction::SOUTH,
        Direction::WEST,
        Direction::EAST,
    ];

    fn data(self) -> &'static Data {
        &DATA[self as usize]
    }

    pub fn values() -> [Direction; 6] {
        Direction::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Direction {
        match name {
            "DOWN" => Direction::DOWN,
            "UP" => Direction::UP,
            "NORTH" => Direction::NORTH,
            "SOUTH" => Direction::SOUTH,
            "WEST" => Direction::WEST,
            "EAST" => Direction::EAST,
            _ => panic!("IllegalArgumentException: no Direction constant {name}"),
        }
    }

    /// The direction a matrix turns `facing` to, as near as one of the six gets.
    pub fn rotate(matrix: &Matrix4f, facing: Direction) -> Direction {
        let mut vec = Vector3f::new();
        matrix.transform_direction_into(facing.get_unit_vec3f(), &mut vec);
        Direction::get_approximate_nearest_f32(vec.x, vec.y, vec.z)
    }

    pub fn all_shuffled(random: &mut dyn RandomSource) -> Vec<Direction> {
        Util::shuffled_copy(&Direction::values(), random)
    }

    pub fn stream() -> impl Iterator<Item = Direction> {
        Direction::VALUES.into_iter()
    }

    /// Panics for a vertical direction, where Java throws `IllegalStateException`.
    pub fn get_y_rot(direction: Direction) -> f32 {
        match direction {
            Direction::NORTH => 180.0,
            Direction::SOUTH => 0.0,
            Direction::WEST => 90.0,
            Direction::EAST => -90.0,
            _ => panic!("IllegalStateException: No y-Rot for vertical axis: {direction}"),
        }
    }

    pub fn get_rotation(&self) -> Quaternionf {
        let mut rotation = Quaternionf::new();
        match self {
            Direction::DOWN => rotation.rotation_x(PI as f32),
            Direction::UP => &mut rotation,
            Direction::NORTH => rotation.rotation_xyz((PI / 2.0) as f32, 0.0, PI as f32),
            Direction::SOUTH => rotation.rotation_x((PI / 2.0) as f32),
            Direction::WEST => rotation.rotation_xyz((PI / 2.0) as f32, 0.0, (PI / 2.0) as f32),
            Direction::EAST => rotation.rotation_xyz((PI / 2.0) as f32, 0.0, (-PI / 2.0) as f32),
        };
        rotation
    }

    pub fn get_3d_data_value(&self) -> i32 {
        self.data().data3d
    }

    pub fn get_2d_data_value(&self) -> i32 {
        self.data().data2d
    }

    pub fn get_axis_direction(&self) -> AxisDirection {
        self.data().axis_direction
    }

    pub fn get_opposite(&self) -> Direction {
        Direction::from_3d_data_value(self.data().opposite_index)
    }

    /// A quarter turn clockwise around `axis`, looking along it toward negative; a direction
    /// along the axis stays as it is.
    pub fn get_clock_wise_axis(&self, axis: Axis) -> Direction {
        match axis {
            Axis::X if *self != Direction::WEST && *self != Direction::EAST => {
                self.get_clock_wise_x()
            }
            Axis::Y if *self != Direction::UP && *self != Direction::DOWN => self.get_clock_wise(),
            Axis::Z if *self != Direction::NORTH && *self != Direction::SOUTH => {
                self.get_clock_wise_z()
            }
            _ => *self,
        }
    }

    pub fn get_counter_clock_wise_axis(&self, axis: Axis) -> Direction {
        match axis {
            Axis::X if *self != Direction::WEST && *self != Direction::EAST => {
                self.get_counter_clock_wise_x()
            }
            Axis::Y if *self != Direction::UP && *self != Direction::DOWN => {
                self.get_counter_clock_wise()
            }
            Axis::Z if *self != Direction::NORTH && *self != Direction::SOUTH => {
                self.get_counter_clock_wise_z()
            }
            _ => *self,
        }
    }

    /// A quarter turn clockwise seen from above. Panics for a vertical direction, where Java
    /// throws `IllegalStateException`.
    pub fn get_clock_wise(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::EAST,
            Direction::SOUTH => Direction::WEST,
            Direction::WEST => Direction::NORTH,
            Direction::EAST => Direction::SOUTH,
            _ => panic!("IllegalStateException: Unable to get Y-rotated facing of {self}"),
        }
    }

    fn get_clock_wise_x(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::SOUTH,
            Direction::UP => Direction::NORTH,
            Direction::NORTH => Direction::DOWN,
            Direction::SOUTH => Direction::UP,
            _ => panic!("IllegalStateException: Unable to get X-rotated facing of {self}"),
        }
    }

    fn get_counter_clock_wise_x(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::NORTH,
            Direction::UP => Direction::SOUTH,
            Direction::NORTH => Direction::UP,
            Direction::SOUTH => Direction::DOWN,
            _ => panic!("IllegalStateException: Unable to get X-rotated facing of {self}"),
        }
    }

    fn get_clock_wise_z(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::WEST,
            Direction::UP => Direction::EAST,
            Direction::WEST => Direction::UP,
            Direction::EAST => Direction::DOWN,
            _ => panic!("IllegalStateException: Unable to get Z-rotated facing of {self}"),
        }
    }

    fn get_counter_clock_wise_z(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::EAST,
            Direction::UP => Direction::WEST,
            Direction::WEST => Direction::DOWN,
            Direction::EAST => Direction::UP,
            _ => panic!("IllegalStateException: Unable to get Z-rotated facing of {self}"),
        }
    }

    /// Panics for a vertical direction, where Java throws `IllegalStateException`.
    pub fn get_counter_clock_wise(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::WEST,
            Direction::SOUTH => Direction::EAST,
            Direction::WEST => Direction::SOUTH,
            Direction::EAST => Direction::NORTH,
            _ => panic!("IllegalStateException: Unable to get CCW facing of {self}"),
        }
    }

    pub fn get_step_x(&self) -> i32 {
        self.data().normal.get_x()
    }

    pub fn get_step_y(&self) -> i32 {
        self.data().normal.get_y()
    }

    pub fn get_step_z(&self) -> i32 {
        self.data().normal.get_z()
    }

    pub fn get_step(&self) -> Vec3i {
        self.get_unit_vec3i()
    }

    /// A copy of the normal the caller can change, unlike `get_unit_vec3f`'s.
    pub fn step(&self) -> Vector3f {
        Vector3f::from_vector3f(&self.data().normal_vec3f)
    }

    pub fn get_name(&self) -> &'static str {
        self.data().name
    }

    pub fn get_axis(&self) -> Axis {
        self.data().axis
    }

    /// By serialized name (`"north"`), or `None`.
    pub fn by_name(name: &str) -> Option<Direction> {
        Direction::values()
            .into_iter()
            .find(|direction| direction.get_serialized_name() == name)
    }

    /// Any int, taken modulo 6, so a negative one counts back from the end.
    pub fn from_3d_data_value(data: i32) -> Direction {
        BY_3D_DATA[Mth::abs_i32(data % BY_3D_DATA.len() as i32) as usize]
    }

    /// Any int, taken modulo 4: south, west, north, east.
    pub fn from_2d_data_value(data: i32) -> Direction {
        BY_2D_DATA[Mth::abs_i32(data % BY_2D_DATA.len() as i32) as usize]
    }

    pub fn from_y_rot(y_rot: f64) -> Direction {
        Direction::from_2d_data_value(Mth::floor_f64(y_rot / 90.0 + 0.5) & 3)
    }

    pub fn from_axis_and_direction(axis: Axis, direction: AxisDirection) -> Direction {
        match axis {
            Axis::X if direction == AxisDirection::POSITIVE => Direction::EAST,
            Axis::X => Direction::WEST,
            Axis::Y if direction == AxisDirection::POSITIVE => Direction::UP,
            Axis::Y => Direction::DOWN,
            Axis::Z if direction == AxisDirection::POSITIVE => Direction::SOUTH,
            Axis::Z => Direction::NORTH,
        }
    }

    /// A vertical direction's 2D data value is -1, which gives 270.
    pub fn to_y_rot(&self) -> f32 {
        ((self.data().data2d & 3) * 90) as f32
    }

    pub fn get_random(random: &mut dyn RandomSource) -> Direction {
        *Util::get_random(&Direction::VALUES, random)
    }

    pub fn get_approximate_nearest_f64(dx: f64, dy: f64, dz: f64) -> Direction {
        Direction::get_approximate_nearest_f32(dx as f32, dy as f32, dz as f32)
    }

    /// The direction whose normal has the largest dot product with the vector, the first one
    /// on a tie. It has to beat `Float.MIN_VALUE`, which is the smallest positive float, so a
    /// vector without a positive dot product (zero, or NaN) gets `NORTH`.
    pub fn get_approximate_nearest_f32(dx: f32, dy: f32, dz: f32) -> Direction {
        let mut result = Direction::NORTH;
        let mut highest_dot = f32::from_bits(1);

        for direction in Direction::VALUES {
            let normal = direction.data().normal;
            let dot = dx * normal.get_x() as f32
                + dy * normal.get_y() as f32
                + dz * normal.get_z() as f32;
            if dot > highest_dot {
                highest_dot = dot;
                result = direction;
            }
        }

        result
    }

    pub fn get_approximate_nearest_vec3(vec: &Vec3) -> Direction {
        Direction::get_approximate_nearest_f64(vec.x, vec.y, vec.z)
    }

    /// The direction of the largest component, or `or_else` when two tie for it.
    pub fn get_nearest_xyz(
        x: i32,
        y: i32,
        z: i32,
        or_else: Option<Direction>,
    ) -> Option<Direction> {
        let abs_x = Math::abs_i32(x);
        let abs_y = Math::abs_i32(y);
        let abs_z = Math::abs_i32(z);
        if abs_x > abs_z && abs_x > abs_y {
            Some(if x < 0 {
                Direction::WEST
            } else {
                Direction::EAST
            })
        } else if abs_z > abs_x && abs_z > abs_y {
            Some(if z < 0 {
                Direction::NORTH
            } else {
                Direction::SOUTH
            })
        } else if abs_y > abs_x && abs_y > abs_z {
            Some(if y < 0 {
                Direction::DOWN
            } else {
                Direction::UP
            })
        } else {
            or_else
        }
    }

    pub fn get_nearest(vec: &Vec3i, or_else: Option<Direction>) -> Option<Direction> {
        Direction::get_nearest_xyz(vec.get_x(), vec.get_y(), vec.get_z(), or_else)
    }

    pub fn get_serialized_name(&self) -> &'static str {
        self.data().name
    }

    /// The direction along `axis` toward `axis_direction`; Java throws
    /// `IllegalArgumentException` if none is, which can't happen.
    pub fn get(axis_direction: AxisDirection, axis: Axis) -> Direction {
        for direction in Direction::VALUES {
            if direction.get_axis_direction() == axis_direction && direction.get_axis() == axis {
                return direction;
            }
        }

        panic!("IllegalArgumentException: No such direction: {axis_direction} {axis}");
    }

    /// The order `Entity.collide` tries the axes in: x before z unless the movement is mostly
    /// along z.
    pub fn axis_step_order(movement: &Vec3) -> [Axis; 3] {
        if movement.x.abs() < movement.z.abs() {
            YZX_AXIS_ORDER
        } else {
            YXZ_AXIS_ORDER
        }
    }

    pub fn get_unit_vec3i(&self) -> Vec3i {
        self.data().normal
    }

    pub fn get_unit_vec3(&self) -> Vec3 {
        self.data().normal_vec3
    }

    /// The shared normal, which nothing may change.
    pub fn get_unit_vec3f(&self) -> &'static Vector3f {
        &self.data().normal_vec3f
    }

    /// Whether a look at `y_angle` degrees (Minecraft's yaw) points this way at all.
    pub fn is_facing_angle(&self, y_angle: f32) -> bool {
        let radians = y_angle * Mth::DEG_TO_RAD;
        let dx = -Mth::sin(f64::from(radians));
        let dz = Mth::cos(f64::from(radians));
        let normal = self.data().normal;
        normal.get_x() as f32 * dx + normal.get_z() as f32 * dz > 0.0
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.data().name)
    }
}

impl Enum for Direction {
    const ENUM_CONSTANTS: &'static [Direction] = &Direction::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for Direction {
    fn get_serialized_name(&self) -> &'static str {
        Direction::get_serialized_name(self)
    }
}

/// A block state's `facing`.
impl PropertyEnum for Direction {
    const CLASS: &'static str = "net.minecraft.core.Direction";
}

/// `Direction.Axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const VALUES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    pub fn values() -> [Axis; 3] {
        Axis::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Axis {
        match name {
            "X" => Axis::X,
            "Y" => Axis::Y,
            "Z" => Axis::Z,
            _ => panic!("IllegalArgumentException: no Direction.Axis constant {name}"),
        }
    }

    /// By serialized name (`"x"`), or `None`.
    pub fn by_name(name: &str) -> Option<Axis> {
        Axis::values()
            .into_iter()
            .find(|axis| axis.get_serialized_name() == name)
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        }
    }

    pub fn is_vertical(&self) -> bool {
        *self == Axis::Y
    }

    pub fn is_horizontal(&self) -> bool {
        *self == Axis::X || *self == Axis::Z
    }

    pub fn get_positive(&self) -> Direction {
        match self {
            Axis::X => Direction::EAST,
            Axis::Y => Direction::UP,
            Axis::Z => Direction::SOUTH,
        }
    }

    pub fn get_negative(&self) -> Direction {
        match self {
            Axis::X => Direction::WEST,
            Axis::Y => Direction::DOWN,
            Axis::Z => Direction::NORTH,
        }
    }

    pub fn get_directions(&self) -> [Direction; 2] {
        [self.get_positive(), self.get_negative()]
    }

    pub fn get_random(random: &mut dyn RandomSource) -> Axis {
        *Util::get_random(&Axis::VALUES, random)
    }

    /// Java's `Predicate<Direction>`, which takes null.
    pub fn test(&self, input: Option<Direction>) -> bool {
        input.is_some_and(|input| input.get_axis() == *self)
    }

    pub fn get_plane(&self) -> Plane {
        match self {
            Axis::X | Axis::Z => Plane::HORIZONTAL,
            Axis::Y => Plane::VERTICAL,
        }
    }

    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    pub fn choose_i32(&self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn choose_f64(&self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn choose_bool(&self, x: bool, y: bool, z: bool) -> bool {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }
}

/// Java's `toString`, the name (`"x"`).
impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_name())
    }
}

impl Enum for Axis {
    const ENUM_CONSTANTS: &'static [Axis] = &Axis::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for Axis {
    fn get_serialized_name(&self) -> &'static str {
        Axis::get_serialized_name(self)
    }
}

/// A block state's `axis`.
impl PropertyEnum for Axis {
    const CLASS: &'static str = "net.minecraft.core.Direction$Axis";
}

/// `Direction.AxisDirection`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisDirection {
    POSITIVE,
    NEGATIVE,
}

impl AxisDirection {
    pub fn values() -> [AxisDirection; 2] {
        [AxisDirection::POSITIVE, AxisDirection::NEGATIVE]
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> AxisDirection {
        match name {
            "POSITIVE" => AxisDirection::POSITIVE,
            "NEGATIVE" => AxisDirection::NEGATIVE,
            _ => panic!("IllegalArgumentException: no Direction.AxisDirection constant {name}"),
        }
    }

    pub fn get_step(&self) -> i32 {
        match self {
            AxisDirection::POSITIVE => 1,
            AxisDirection::NEGATIVE => -1,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            AxisDirection::POSITIVE => "Towards positive",
            AxisDirection::NEGATIVE => "Towards negative",
        }
    }

    pub fn opposite(&self) -> AxisDirection {
        match self {
            AxisDirection::POSITIVE => AxisDirection::NEGATIVE,
            AxisDirection::NEGATIVE => AxisDirection::POSITIVE,
        }
    }
}

/// Java's `toString`, the name (`"Towards positive"`).
impl fmt::Display for AxisDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_name())
    }
}

/// `Direction.Plane`: the horizontal directions, in clockwise order from north, or the
/// vertical ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Plane {
    HORIZONTAL,
    VERTICAL,
}

impl Plane {
    pub fn values() -> [Plane; 2] {
        [Plane::HORIZONTAL, Plane::VERTICAL]
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Plane {
        match name {
            "HORIZONTAL" => Plane::HORIZONTAL,
            "VERTICAL" => Plane::VERTICAL,
            _ => panic!("IllegalArgumentException: no Direction.Plane constant {name}"),
        }
    }

    const fn faces(&self) -> &'static [Direction] {
        match self {
            Plane::HORIZONTAL => &[
                Direction::NORTH,
                Direction::EAST,
                Direction::SOUTH,
                Direction::WEST,
            ],
            Plane::VERTICAL => &[Direction::UP, Direction::DOWN],
        }
    }

    const fn axis(&self) -> &'static [Axis] {
        match self {
            Plane::HORIZONTAL => &[Axis::X, Axis::Z],
            Plane::VERTICAL => &[Axis::Y],
        }
    }

    pub fn get_random_direction(&self, random: &mut dyn RandomSource) -> Direction {
        *Util::get_random(self.faces(), random)
    }

    pub fn get_random_axis(&self, random: &mut dyn RandomSource) -> Axis {
        *Util::get_random(self.axis(), random)
    }

    /// Java's `Predicate<Direction>`, which takes null.
    pub fn test(&self, input: Option<Direction>) -> bool {
        input.is_some_and(|input| input.get_axis().get_plane() == *self)
    }

    /// Java's `Iterable<Direction>`.
    pub fn iterator(&self) -> impl Iterator<Item = Direction> {
        self.faces().iter().copied()
    }

    pub fn stream(&self) -> impl Iterator<Item = Direction> {
        self.faces().iter().copied()
    }

    pub fn shuffled_copy(&self, random: &mut dyn RandomSource) -> Vec<Direction> {
        Util::shuffled_copy(self.faces(), random)
    }

    pub fn length(&self) -> i32 {
        self.faces().len() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::{Axis, AxisDirection, Direction, Plane};
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::vec3i::Vec3i;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::org::joml::matrix4f::Matrix4f;

    encode_names!(Direction {
        DOWN,
        UP,
        NORTH,
        SOUTH,
        WEST,
        EAST
    });
    encode_names!(Axis { X, Y, Z });
    encode_names!(AxisDirection { POSITIVE, NEGATIVE });
    encode_names!(Plane {
        HORIZONTAL,
        VERTICAL
    });

    #[test]
    fn direction_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Direction");
        for codec in [
            "CODEC",
            "VERTICAL_CODEC",
            "LEGACY_ID_CODEC",
            "LEGACY_ID_CODEC_2D",
        ] {
            golden.pending(codec, "waits for DataFixerUpper's codecs");
        }
        golden.check_codec("STREAM_CODEC", || Direction::STREAM_CODEC);
        golden.check_function("BY_ID", || Direction::BY_ID.as_ref());
        golden.pending(
            "orderedByNearest(Lnet/minecraft/world/entity/Entity;)[Lnet/minecraft/core/Direction;",
            "waits for Entity",
        );
        golden.pending(
            "getFacingAxis(Lnet/minecraft/world/entity/Entity;Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction;",
            "waits for Entity",
        );
        for direction in Direction::VALUES {
            golden.field(&format!("{direction:?}"), direction);
        }
        golden.check("values()[Lnet/minecraft/core/Direction;", Direction::values);
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction;",
            |name: String| Direction::value_of(&name),
        );
        golden.check(
            "rotate(Lorg/joml/Matrix4fc;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;",
            |matrix: Matrix4f, facing| Direction::rotate(&matrix, facing),
        );
        golden.check_random(
            "allShuffled(Lnet/minecraft/util/RandomSource;)Ljava/util/Collection;",
            |random: &mut dyn RandomSource| Direction::all_shuffled(random),
        );
        golden.check("stream()Ljava/util/stream/Stream;", || {
            Direction::stream().collect::<Vec<_>>()
        });
        golden.check(
            "getYRot(Lnet/minecraft/core/Direction;)F",
            Direction::get_y_rot,
        );
        golden.check_method(
            "getRotation()Lorg/joml/Quaternionf;",
            |this: &mut Direction| this.get_rotation(),
        );
        golden.check_method("get3DDataValue()I", |this: &mut Direction| {
            this.get_3d_data_value()
        });
        golden.check_method("get2DDataValue()I", |this: &mut Direction| {
            this.get_2d_data_value()
        });
        golden.check_method(
            "getAxisDirection()Lnet/minecraft/core/Direction$AxisDirection;",
            |this: &mut Direction| this.get_axis_direction(),
        );
        golden.check_method(
            "getOpposite()Lnet/minecraft/core/Direction;",
            |this: &mut Direction| this.get_opposite(),
        );
        golden.check_method(
            "getClockWise(Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction;",
            |this: &mut Direction, axis| this.get_clock_wise_axis(axis),
        );
        golden.check_method(
            "getCounterClockWise(Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction;",
            |this: &mut Direction, axis| this.get_counter_clock_wise_axis(axis),
        );
        golden.check_method(
            "getClockWise()Lnet/minecraft/core/Direction;",
            |this: &mut Direction| this.get_clock_wise(),
        );
        golden.check_method(
            "getCounterClockWise()Lnet/minecraft/core/Direction;",
            |this: &mut Direction| this.get_counter_clock_wise(),
        );
        golden.check_method("getStepX()I", |this: &mut Direction| this.get_step_x());
        golden.check_method("getStepY()I", |this: &mut Direction| this.get_step_y());
        golden.check_method("getStepZ()I", |this: &mut Direction| this.get_step_z());
        golden.check_method(
            "getStep()Lnet/minecraft/core/Vec3i;",
            |this: &mut Direction| this.get_step(),
        );
        golden.check_method("step()Lorg/joml/Vector3f;", |this: &mut Direction| {
            this.step()
        });
        golden.check_method("getName()Ljava/lang/String;", |this: &mut Direction| {
            this.get_name().to_owned()
        });
        golden.check_method(
            "getAxis()Lnet/minecraft/core/Direction$Axis;",
            |this: &mut Direction| this.get_axis(),
        );
        golden.check(
            "byName(Ljava/lang/String;)Lnet/minecraft/core/Direction;",
            |name: String| Direction::by_name(&name),
        );
        golden.check(
            "from3DDataValue(I)Lnet/minecraft/core/Direction;",
            Direction::from_3d_data_value,
        );
        golden.check(
            "from2DDataValue(I)Lnet/minecraft/core/Direction;",
            Direction::from_2d_data_value,
        );
        golden.check(
            "fromYRot(D)Lnet/minecraft/core/Direction;",
            Direction::from_y_rot,
        );
        golden.check(
            "fromAxisAndDirection(Lnet/minecraft/core/Direction$Axis;Lnet/minecraft/core/Direction$AxisDirection;)Lnet/minecraft/core/Direction;",
            Direction::from_axis_and_direction,
        );
        golden.check_method("toYRot()F", |this: &mut Direction| this.to_y_rot());
        golden.check_random(
            "getRandom(Lnet/minecraft/util/RandomSource;)Lnet/minecraft/core/Direction;",
            |random: &mut dyn RandomSource| Direction::get_random(random),
        );
        golden.check(
            "getApproximateNearest(DDD)Lnet/minecraft/core/Direction;",
            Direction::get_approximate_nearest_f64,
        );
        golden.check(
            "getApproximateNearest(FFF)Lnet/minecraft/core/Direction;",
            Direction::get_approximate_nearest_f32,
        );
        golden.check(
            "getApproximateNearest(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/core/Direction;",
            |vec: Vec3| Direction::get_approximate_nearest_vec3(&vec),
        );
        golden.check(
            "getNearest(IIILnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;",
            Direction::get_nearest_xyz,
        );
        golden.check(
            "getNearest(Lnet/minecraft/core/Vec3i;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;",
            |vec: Vec3i, or_else| Direction::get_nearest(&vec, or_else),
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Direction| {
            this.to_string()
        });
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Direction| this.get_serialized_name().to_owned(),
        );
        golden.check(
            "get(Lnet/minecraft/core/Direction$AxisDirection;Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction;",
            Direction::get,
        );
        golden.check(
            "axisStepOrder(Lnet/minecraft/world/phys/Vec3;)Lcom/google/common/collect/ImmutableList;",
            |movement: Vec3| Direction::axis_step_order(&movement),
        );
        golden.check_method(
            "getUnitVec3i()Lnet/minecraft/core/Vec3i;",
            |this: &mut Direction| this.get_unit_vec3i(),
        );
        golden.check_method(
            "getUnitVec3()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Direction| this.get_unit_vec3(),
        );
        golden.check_method(
            "getUnitVec3f()Lorg/joml/Vector3fc;",
            |this: &mut Direction| *this.get_unit_vec3f(),
        );
        golden.check_method("isFacingAngle(F)Z", |this: &mut Direction, y_angle| {
            this.is_facing_angle(y_angle)
        });
        golden.finish();
    }

    #[test]
    fn axis_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Direction$Axis");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.field("X", Axis::X);
        golden.field("Y", Axis::Y);
        golden.field("Z", Axis::Z);
        golden.field("VALUES", Axis::VALUES);
        golden.check("values()[Lnet/minecraft/core/Direction$Axis;", Axis::values);
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;",
            |name: String| Axis::value_of(&name),
        );
        golden.check(
            "byName(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;",
            |name: String| Axis::by_name(&name),
        );
        golden.check_method("getName()Ljava/lang/String;", |this: &mut Axis| {
            this.get_name().to_owned()
        });
        golden.check_method("isVertical()Z", |this: &mut Axis| this.is_vertical());
        golden.check_method("isHorizontal()Z", |this: &mut Axis| this.is_horizontal());
        golden.check_method(
            "getPositive()Lnet/minecraft/core/Direction;",
            |this: &mut Axis| this.get_positive(),
        );
        golden.check_method(
            "getNegative()Lnet/minecraft/core/Direction;",
            |this: &mut Axis| this.get_negative(),
        );
        golden.check_method(
            "getDirections()[Lnet/minecraft/core/Direction;",
            |this: &mut Axis| this.get_directions(),
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Axis| {
            this.to_string()
        });
        golden.check_random(
            "getRandom(Lnet/minecraft/util/RandomSource;)Lnet/minecraft/core/Direction$Axis;",
            |random: &mut dyn RandomSource| Axis::get_random(random),
        );
        golden.check_method(
            "test(Lnet/minecraft/core/Direction;)Z",
            |this: &mut Axis, input| this.test(input),
        );
        golden.check_method(
            "getPlane()Lnet/minecraft/core/Direction$Plane;",
            |this: &mut Axis| this.get_plane(),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Axis| this.get_serialized_name().to_owned(),
        );
        golden.check_method("choose(III)I", |this: &mut Axis, x, y, z| {
            this.choose_i32(x, y, z)
        });
        golden.check_method("choose(DDD)D", |this: &mut Axis, x, y, z| {
            this.choose_f64(x, y, z)
        });
        golden.check_method("choose(ZZZ)Z", |this: &mut Axis, x, y, z| {
            this.choose_bool(x, y, z)
        });
        golden.finish();
    }

    #[test]
    fn axis_direction_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Direction$AxisDirection");
        golden.field("POSITIVE", AxisDirection::POSITIVE);
        golden.field("NEGATIVE", AxisDirection::NEGATIVE);
        golden.check(
            "values()[Lnet/minecraft/core/Direction$AxisDirection;",
            AxisDirection::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$AxisDirection;",
            |name: String| AxisDirection::value_of(&name),
        );
        golden.check_method("getStep()I", |this: &mut AxisDirection| this.get_step());
        golden.check_method("getName()Ljava/lang/String;", |this: &mut AxisDirection| {
            this.get_name().to_owned()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut AxisDirection| this.to_string(),
        );
        golden.check_method(
            "opposite()Lnet/minecraft/core/Direction$AxisDirection;",
            |this: &mut AxisDirection| this.opposite(),
        );
        golden.finish();
    }

    #[test]
    fn plane_matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Direction$Plane");
        golden.field("HORIZONTAL", Plane::HORIZONTAL);
        golden.field("VERTICAL", Plane::VERTICAL);
        golden.check(
            "values()[Lnet/minecraft/core/Direction$Plane;",
            Plane::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Plane;",
            |name: String| Plane::value_of(&name),
        );
        golden.check_mut(
            "getRandomDirection(Lnet/minecraft/util/RandomSource;)Lnet/minecraft/core/Direction;",
            |this: &mut Plane, random: &mut LegacyRandomSource| this.get_random_direction(random),
        );
        golden.check_mut(
            "getRandomAxis(Lnet/minecraft/util/RandomSource;)Lnet/minecraft/core/Direction$Axis;",
            |this: &mut Plane, random: &mut LegacyRandomSource| this.get_random_axis(random),
        );
        golden.check_method(
            "test(Lnet/minecraft/core/Direction;)Z",
            |this: &mut Plane, input| this.test(input),
        );
        golden.check_method("iterator()Ljava/util/Iterator;", |this: &mut Plane| {
            this.iterator().collect::<Vec<_>>()
        });
        golden.check_method("stream()Ljava/util/stream/Stream;", |this: &mut Plane| {
            this.stream().collect::<Vec<_>>()
        });
        golden.check_mut(
            "shuffledCopy(Lnet/minecraft/util/RandomSource;)Ljava/util/List;",
            |this: &mut Plane, random: &mut LegacyRandomSource| this.shuffled_copy(random),
        );
        golden.check_method("length()I", |this: &mut Plane| this.length());
        golden.finish();
    }
}
