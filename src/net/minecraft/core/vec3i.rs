//! net/minecraft/core/Vec3i.java
//!
//! Not ported yet: the codecs (`CODEC`, `STREAM_CODEC` and `offsetCodec`), which wait for
//! DataFixerUpper and the network buffers.

use std::fmt;

use crate::java::lang::math::Math;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::position::Position;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::vector3i::Vector3i;

/// A position or offset in whole blocks. Java's `equals` compares the coordinates, like the
/// derived `PartialEq`, and the arithmetic wraps like Java's ints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec3i {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3i {
    pub const ZERO: Vec3i = Vec3i::new(0, 0, 0);

    pub const fn new(x: i32, y: i32, z: i32) -> Vec3i {
        Vec3i { x, y, z }
    }

    pub fn hash_code(&self) -> i32 {
        self.get_y()
            .wrapping_add(self.get_z().wrapping_mul(31))
            .wrapping_mul(31)
            .wrapping_add(self.get_x())
    }

    /// By y, then z, then x, as the difference of the first coordinates that differ, which
    /// wraps when they're far apart.
    pub fn compare_to(&self, pos: &Vec3i) -> i32 {
        if self.get_y() == pos.get_y() {
            if self.get_z() == pos.get_z() {
                self.get_x().wrapping_sub(pos.get_x())
            } else {
                self.get_z().wrapping_sub(pos.get_z())
            }
        } else {
            self.get_y().wrapping_sub(pos.get_y())
        }
    }

    pub const fn get_x(&self) -> i32 {
        self.x
    }

    pub const fn get_y(&self) -> i32 {
        self.y
    }

    pub const fn get_z(&self) -> i32 {
        self.z
    }

    /// Java's protected setters, which only `MutableBlockPos` calls.
    pub(super) fn set_x(&mut self, x: i32) -> &mut Vec3i {
        self.x = x;
        self
    }

    pub(super) fn set_y(&mut self, y: i32) -> &mut Vec3i {
        self.y = y;
        self
    }

    pub(super) fn set_z(&mut self, z: i32) -> &mut Vec3i {
        self.z = z;
        self
    }

    pub fn offset_xyz(&self, x: i32, y: i32, z: i32) -> Vec3i {
        if x == 0 && y == 0 && z == 0 {
            *self
        } else {
            Vec3i::new(
                self.get_x().wrapping_add(x),
                self.get_y().wrapping_add(y),
                self.get_z().wrapping_add(z),
            )
        }
    }

    pub fn offset(&self, vec: &Vec3i) -> Vec3i {
        self.offset_xyz(vec.get_x(), vec.get_y(), vec.get_z())
    }

    pub fn subtract(&self, vec: &Vec3i) -> Vec3i {
        self.offset_xyz(
            vec.get_x().wrapping_neg(),
            vec.get_y().wrapping_neg(),
            vec.get_z().wrapping_neg(),
        )
    }

    pub fn multiply(&self, scale: i32) -> Vec3i {
        if scale == 1 {
            *self
        } else if scale == 0 {
            Vec3i::ZERO
        } else {
            Vec3i::new(
                self.get_x().wrapping_mul(scale),
                self.get_y().wrapping_mul(scale),
                self.get_z().wrapping_mul(scale),
            )
        }
    }

    pub fn multiply_xyz(&self, x_scale: i32, y_scale: i32, z_scale: i32) -> Vec3i {
        Vec3i::new(
            self.get_x().wrapping_mul(x_scale),
            self.get_y().wrapping_mul(y_scale),
            self.get_z().wrapping_mul(z_scale),
        )
    }

    pub fn above(&self) -> Vec3i {
        self.above_steps(1)
    }

    pub fn above_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::UP, steps)
    }

    pub fn below(&self) -> Vec3i {
        self.below_steps(1)
    }

    pub fn below_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::DOWN, steps)
    }

    pub fn north(&self) -> Vec3i {
        self.north_steps(1)
    }

    pub fn north_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::NORTH, steps)
    }

    pub fn south(&self) -> Vec3i {
        self.south_steps(1)
    }

    pub fn south_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::SOUTH, steps)
    }

    pub fn west(&self) -> Vec3i {
        self.west_steps(1)
    }

    pub fn west_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::WEST, steps)
    }

    pub fn east(&self) -> Vec3i {
        self.east_steps(1)
    }

    pub fn east_steps(&self, steps: i32) -> Vec3i {
        self.relative_steps(Direction::EAST, steps)
    }

    pub fn relative(&self, direction: Direction) -> Vec3i {
        self.relative_steps(direction, 1)
    }

    pub fn relative_steps(&self, direction: Direction, steps: i32) -> Vec3i {
        if steps == 0 {
            *self
        } else {
            Vec3i::new(
                self.get_x()
                    .wrapping_add(direction.get_step_x().wrapping_mul(steps)),
                self.get_y()
                    .wrapping_add(direction.get_step_y().wrapping_mul(steps)),
                self.get_z()
                    .wrapping_add(direction.get_step_z().wrapping_mul(steps)),
            )
        }
    }

    pub fn relative_axis(&self, axis: Axis, steps: i32) -> Vec3i {
        if steps == 0 {
            return *self;
        }

        let x_step = if axis == Axis::X { steps } else { 0 };
        let y_step = if axis == Axis::Y { steps } else { 0 };
        let z_step = if axis == Axis::Z { steps } else { 0 };
        Vec3i::new(
            self.get_x().wrapping_add(x_step),
            self.get_y().wrapping_add(y_step),
            self.get_z().wrapping_add(z_step),
        )
    }

    pub fn cross(&self, up_vector: &Vec3i) -> Vec3i {
        Vec3i::new(
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

    pub fn closer_than(&self, pos: &Vec3i, distance: f64) -> bool {
        self.dist_sqr(pos) < Mth::square_f64(distance)
    }

    pub fn closer_to_center_than(&self, pos: &dyn Position, distance: f64) -> bool {
        self.dist_to_center_sqr(pos) < Mth::square_f64(distance)
    }

    pub fn dist_sqr(&self, pos: &Vec3i) -> f64 {
        self.dist_to_low_corner_sqr(
            f64::from(pos.get_x()),
            f64::from(pos.get_y()),
            f64::from(pos.get_z()),
        )
    }

    pub fn dist_to_center_sqr(&self, pos: &dyn Position) -> f64 {
        self.dist_to_center_sqr_xyz(pos.x(), pos.y(), pos.z())
    }

    pub fn dist_to_center_sqr_xyz(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = f64::from(self.get_x()) + 0.5 - x;
        let dy = f64::from(self.get_y()) + 0.5 - y;
        let dz = f64::from(self.get_z()) + 0.5 - z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn dist_to_low_corner_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = f64::from(self.get_x()) - x;
        let dy = f64::from(self.get_y()) - y;
        let dz = f64::from(self.get_z()) - z;
        dx * dx + dy * dy + dz * dz
    }

    /// Sums the distances as floats, which round once they're past 2^24.
    pub fn dist_manhattan(&self, pos: &Vec3i) -> i32 {
        let xd = Math::abs_i32(pos.get_x().wrapping_sub(self.get_x())) as f32;
        let yd = Math::abs_i32(pos.get_y().wrapping_sub(self.get_y())) as f32;
        let zd = Math::abs_i32(pos.get_z().wrapping_sub(self.get_z())) as f32;
        (xd + yd + zd) as i32
    }

    pub fn dist_chessboard(&self, pos: &Vec3i) -> i32 {
        let xd = Math::abs_i32(self.get_x().wrapping_sub(pos.get_x()));
        let yd = Math::abs_i32(self.get_y().wrapping_sub(pos.get_y()));
        let zd = Math::abs_i32(self.get_z().wrapping_sub(pos.get_z()));
        xd.max(yd).max(zd)
    }

    pub fn differs_horizontally(&self, pos: &Vec3i) -> bool {
        self.x != pos.x || self.z != pos.z
    }

    pub fn get(&self, axis: Axis) -> i32 {
        axis.choose_i32(self.x, self.y, self.z)
    }

    pub fn to_mutable(&self) -> Vector3i {
        Vector3i::from_xyz(self.x, self.y, self.z)
    }

    pub fn to_short_string(&self) -> String {
        format!("{}, {}, {}", self.get_x(), self.get_y(), self.get_z())
    }

    /// Java's `toString`, `MoreObjects.toStringHelper`'s format with the simple name of the
    /// receiver's class, which subclasses (`BlockPos`) print instead of `Vec3i`.
    pub(super) fn fmt_with_class_name(
        &self,
        f: &mut fmt::Formatter<'_>,
        class_name: &str,
    ) -> fmt::Result {
        write!(
            f,
            "{class_name}{{x={}, y={}, z={}}}",
            self.get_x(),
            self.get_y(),
            self.get_z()
        )
    }
}

impl fmt::Display for Vec3i {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_class_name(f, "Vec3i")
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3i;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::world::phys::vec3::Vec3;

    encode_fields!(Vec3i {
        x: i32,
        y: i32,
        z: i32
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Vec3i");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending("STREAM_CODEC", "waits for the network buffers");
        golden.pending(
            "offsetCodec(I)Lcom/mojang/serialization/Codec;",
            "waits for DataFixerUpper's codecs",
        );
        golden.field("ZERO", Vec3i::ZERO);
        golden.check("<init>(III)V", Vec3i::new);
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Vec3i, o: Vec3i| *this == o,
        );
        golden.check_method("hashCode()I", |this: &mut Vec3i| this.hash_code());
        golden.check_method(
            "compareTo(Lnet/minecraft/core/Vec3i;)I",
            |this: &mut Vec3i, pos: Vec3i| this.compare_to(&pos),
        );
        golden.check_method("getX()I", |this: &mut Vec3i| this.get_x());
        golden.check_method("getY()I", |this: &mut Vec3i| this.get_y());
        golden.check_method("getZ()I", |this: &mut Vec3i| this.get_z());
        golden.check_method(
            "offset(III)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, x, y, z| this.offset_xyz(x, y, z),
        );
        golden.check_method(
            "offset(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, vec: Vec3i| this.offset(&vec),
        );
        golden.check_method(
            "subtract(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, vec: Vec3i| this.subtract(&vec),
        );
        golden.check_method(
            "multiply(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, scale| this.multiply(scale),
        );
        golden.check_method(
            "multiply(III)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, x_scale, y_scale, z_scale| {
                this.multiply_xyz(x_scale, y_scale, z_scale)
            },
        );
        golden.check_method("above()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.above()
        });
        golden.check_method(
            "above(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.above_steps(steps),
        );
        golden.check_method("below()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.below()
        });
        golden.check_method(
            "below(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.below_steps(steps),
        );
        golden.check_method("north()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.north()
        });
        golden.check_method(
            "north(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.north_steps(steps),
        );
        golden.check_method("south()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.south()
        });
        golden.check_method(
            "south(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.south_steps(steps),
        );
        golden.check_method("west()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.west()
        });
        golden.check_method(
            "west(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.west_steps(steps),
        );
        golden.check_method("east()Lnet/minecraft/core/Vec3i;", |this: &mut Vec3i| {
            this.east()
        });
        golden.check_method(
            "east(I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, steps| this.east_steps(steps),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, direction: Direction| this.relative(direction),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, direction: Direction, steps| this.relative_steps(direction, steps),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction$Axis;I)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, axis: Axis, steps| this.relative_axis(axis, steps),
        );
        golden.check_method(
            "cross(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/core/Vec3i;",
            |this: &mut Vec3i, up_vector: Vec3i| this.cross(&up_vector),
        );
        golden.check_method(
            "closerThan(Lnet/minecraft/core/Vec3i;D)Z",
            |this: &mut Vec3i, pos: Vec3i, distance| this.closer_than(&pos, distance),
        );
        golden.check_method(
            "closerToCenterThan(Lnet/minecraft/core/Position;D)Z",
            |this: &mut Vec3i, pos: Vec3, distance| this.closer_to_center_than(&pos, distance),
        );
        golden.check_method(
            "distSqr(Lnet/minecraft/core/Vec3i;)D",
            |this: &mut Vec3i, pos: Vec3i| this.dist_sqr(&pos),
        );
        golden.check_method(
            "distToCenterSqr(Lnet/minecraft/core/Position;)D",
            |this: &mut Vec3i, pos: Vec3| this.dist_to_center_sqr(&pos),
        );
        golden.check_method("distToCenterSqr(DDD)D", |this: &mut Vec3i, x, y, z| {
            this.dist_to_center_sqr_xyz(x, y, z)
        });
        golden.check_method("distToLowCornerSqr(DDD)D", |this: &mut Vec3i, x, y, z| {
            this.dist_to_low_corner_sqr(x, y, z)
        });
        golden.check_method(
            "distManhattan(Lnet/minecraft/core/Vec3i;)I",
            |this: &mut Vec3i, pos: Vec3i| this.dist_manhattan(&pos),
        );
        golden.check_method(
            "distChessboard(Lnet/minecraft/core/Vec3i;)I",
            |this: &mut Vec3i, pos: Vec3i| this.dist_chessboard(&pos),
        );
        golden.check_method(
            "differsHorizontally(Lnet/minecraft/core/Vec3i;)Z",
            |this: &mut Vec3i, pos: Vec3i| this.differs_horizontally(&pos),
        );
        golden.check_method(
            "get(Lnet/minecraft/core/Direction$Axis;)I",
            |this: &mut Vec3i, axis: Axis| this.get(axis),
        );
        golden.check_method("toMutable()Lorg/joml/Vector3i;", |this: &mut Vec3i| {
            this.to_mutable()
        });
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Vec3i| {
            this.to_string()
        });
        golden.check_method("toShortString()Ljava/lang/String;", |this: &mut Vec3i| {
            this.to_short_string()
        });
        golden.finish();
    }
}
