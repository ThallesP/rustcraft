//! net/minecraft/world/phys/Vec3.java
//!
//! Not ported yet: the codecs (`CODEC`, `STREAM_CODEC`, `LP_STREAM_CODEC`), which wait for
//! DataFixerUpper and the network buffers, and `toString`, which needs Java's
//! `Double.toString` (only logs and error messages print it).

use crate::java::lang::double::Double;
use crate::java::lang::math::Math;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::position::Position;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::phys::vec2::Vec2;
use crate::org::joml::vector3f::Vector3f;

/// A position, movement or direction in blocks.
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Java's `equals` compares the components with `Double.compare`, so -0.0 differs from 0.0 and
/// NaN equals NaN.
impl PartialEq for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        Double::double_to_long_bits(self.x) == Double::double_to_long_bits(other.x)
            && Double::double_to_long_bits(self.y) == Double::double_to_long_bits(other.y)
            && Double::double_to_long_bits(self.z) == Double::double_to_long_bits(other.z)
    }
}

impl Eq for Vec3 {}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const X_AXIS: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const Y_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const Z_AXIS: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    pub const fn at_lower_corner_of(pos: &Vec3i) -> Vec3 {
        Vec3::new(pos.get_x() as f64, pos.get_y() as f64, pos.get_z() as f64)
    }

    pub fn at_lower_corner_with_offset(pos: &Vec3i, x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(
            f64::from(pos.get_x()) + x,
            f64::from(pos.get_y()) + y,
            f64::from(pos.get_z()) + z,
        )
    }

    pub fn at_center_of(pos: &Vec3i) -> Vec3 {
        Vec3::at_lower_corner_with_offset(pos, 0.5, 0.5, 0.5)
    }

    pub fn at_center_of_with_y(pos: &Vec3i, y: f64) -> Vec3 {
        Vec3::new(
            f64::from(pos.get_x()) + 0.5,
            y,
            f64::from(pos.get_z()) + 0.5,
        )
    }

    pub fn at_bottom_center_of(pos: &Vec3i) -> Vec3 {
        Vec3::at_lower_corner_with_offset(pos, 0.5, 0.0, 0.5)
    }

    pub fn up_from_bottom_center_of(pos: &Vec3i, y_offset: f64) -> Vec3 {
        Vec3::at_lower_corner_with_offset(pos, 0.5, y_offset, 0.5)
    }

    pub const fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn from_vector3f(vec: &Vector3f) -> Vec3 {
        Vec3::new(f64::from(vec.x), f64::from(vec.y), f64::from(vec.z))
    }

    pub fn from_vec3i(vec: &Vec3i) -> Vec3 {
        Vec3::new(
            f64::from(vec.get_x()),
            f64::from(vec.get_y()),
            f64::from(vec.get_z()),
        )
    }

    pub fn vector_to(&self, vec: &Vec3) -> Vec3 {
        Vec3::new(vec.x - self.x, vec.y - self.y, vec.z - self.z)
    }

    /// `ZERO` when the length is below `1.0E-5F`, widened to a double.
    pub fn normalize(&self) -> Vec3 {
        let dist = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if dist < f64::from(1.0E-5f32) {
            Vec3::ZERO
        } else {
            Vec3::new(self.x / dist, self.y / dist, self.z / dist)
        }
    }

    pub fn dot(&self, vec: &Vec3) -> f64 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&self, vec: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * vec.z - self.z * vec.y,
            self.z * vec.x - self.x * vec.z,
            self.x * vec.y - self.y * vec.x,
        )
    }

    pub fn subtract(&self, vec: &Vec3) -> Vec3 {
        self.subtract_xyz(vec.x, vec.y, vec.z)
    }

    pub fn subtract_scalar(&self, value: f64) -> Vec3 {
        self.subtract_xyz(value, value, value)
    }

    pub fn subtract_xyz(&self, x: f64, y: f64, z: f64) -> Vec3 {
        self.add_xyz(-x, -y, -z)
    }

    pub fn add_scalar(&self, value: f64) -> Vec3 {
        self.add_xyz(value, value, value)
    }

    pub fn add(&self, vec: &Vec3) -> Vec3 {
        self.add_xyz(vec.x, vec.y, vec.z)
    }

    pub fn add_xyz(&self, x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn closer_than(&self, pos: &dyn Position, distance: f64) -> bool {
        self.distance_to_sqr_xyz(pos.x(), pos.y(), pos.z()) < distance * distance
    }

    pub fn distance_to(&self, vec: &Vec3) -> f64 {
        let xd = vec.x - self.x;
        let yd = vec.y - self.y;
        let zd = vec.z - self.z;
        (xd * xd + yd * yd + zd * zd).sqrt()
    }

    pub fn distance_to_sqr(&self, vec: &Vec3) -> f64 {
        let xd = vec.x - self.x;
        let yd = vec.y - self.y;
        let zd = vec.z - self.z;
        xd * xd + yd * yd + zd * zd
    }

    pub fn distance_to_sqr_xyz(&self, x: f64, y: f64, z: f64) -> f64 {
        let xd = x - self.x;
        let yd = y - self.y;
        let zd = z - self.z;
        xd * xd + yd * yd + zd * zd
    }

    /// Within a cylinder: closer than `distance_xz` horizontally and `distance_y` vertically.
    pub fn closer_than_xz_y(&self, vec: &Vec3, distance_xz: f64, distance_y: f64) -> bool {
        let dx = vec.x() - self.x;
        let dy = vec.y() - self.y;
        let dz = vec.z() - self.z;
        Mth::length_squared_f64(dx, dz) < Mth::square_f64(distance_xz) && dy.abs() < distance_y
    }

    pub fn scale(&self, scale: f64) -> Vec3 {
        self.multiply_xyz(scale, scale, scale)
    }

    pub fn reverse(&self) -> Vec3 {
        self.scale(-1.0)
    }

    pub fn multiply(&self, scale: &Vec3) -> Vec3 {
        self.multiply_xyz(scale.x, scale.y, scale.z)
    }

    pub fn multiply_xyz(&self, x_scale: f64, y_scale: f64, z_scale: f64) -> Vec3 {
        Vec3::new(self.x * x_scale, self.y * y_scale, self.z * z_scale)
    }

    pub fn horizontal(&self) -> Vec3 {
        Vec3::new(self.x, 0.0, self.z)
    }

    /// Moves by up to half of `offset` along each axis, drawing x, y and z in that order.
    pub fn offset_random(&self, random: &mut dyn RandomSource, offset: f32) -> Vec3 {
        self.add_xyz(
            f64::from((random.next_float() - 0.5) * offset),
            f64::from((random.next_float() - 0.5) * offset),
            f64::from((random.next_float() - 0.5) * offset),
        )
    }

    pub fn offset_random_xz(&self, random: &mut dyn RandomSource, offset: f32) -> Vec3 {
        self.add_xyz(
            f64::from((random.next_float() - 0.5) * offset),
            0.0,
            f64::from((random.next_float() - 0.5) * offset),
        )
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn length_sqr(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn horizontal_distance(&self) -> f64 {
        (self.x * self.x + self.z * self.z).sqrt()
    }

    pub fn horizontal_distance_sqr(&self) -> f64 {
        self.x * self.x + self.z * self.z
    }

    /// Each component's `Double.hashCode`, folded in by 31s.
    pub fn hash_code(&self) -> i32 {
        let result = Double::hash_code(self.x);
        let result = result
            .wrapping_mul(31)
            .wrapping_add(Double::hash_code(self.y));
        result
            .wrapping_mul(31)
            .wrapping_add(Double::hash_code(self.z))
    }

    pub fn lerp(&self, vec: &Vec3, a: f64) -> Vec3 {
        Vec3::new(
            Mth::lerp_f64(a, self.x, vec.x),
            Mth::lerp_f64(a, self.y, vec.y),
            Mth::lerp_f64(a, self.z, vec.z),
        )
    }

    /// Rotations take `Mth`'s table sine and cosine, as floats.
    pub fn x_rot(&self, radians: f32) -> Vec3 {
        let cos = f64::from(Mth::cos(f64::from(radians)));
        let sin = f64::from(Mth::sin(f64::from(radians)));
        let xx = self.x;
        let yy = self.y * cos + self.z * sin;
        let zz = self.z * cos - self.y * sin;
        Vec3::new(xx, yy, zz)
    }

    pub fn y_rot(&self, radians: f32) -> Vec3 {
        let cos = f64::from(Mth::cos(f64::from(radians)));
        let sin = f64::from(Mth::sin(f64::from(radians)));
        let xx = self.x * cos + self.z * sin;
        let yy = self.y;
        let zz = self.z * cos - self.x * sin;
        Vec3::new(xx, yy, zz)
    }

    pub fn z_rot(&self, radians: f32) -> Vec3 {
        let cos = f64::from(Mth::cos(f64::from(radians)));
        let sin = f64::from(Mth::sin(f64::from(radians)));
        let xx = self.x * cos + self.y * sin;
        let yy = self.y * cos - self.x * sin;
        let zz = self.z;
        Vec3::new(xx, yy, zz)
    }

    pub fn rotate_clockwise90(&self) -> Vec3 {
        Vec3::new(-self.z, self.y, self.x)
    }

    pub fn direction_from_rotation_vec2(rotation: &Vec2) -> Vec3 {
        Vec3::direction_from_rotation(rotation.x, rotation.y)
    }

    /// The unit look vector for an x rotation (pitch) and y rotation (yaw) in degrees, worked
    /// out in floats.
    pub fn direction_from_rotation(rot_x: f32, rot_y: f32) -> Vec3 {
        let y_cos = Mth::cos(f64::from(-rot_y * Mth::DEG_TO_RAD - Mth::PI));
        let y_sin = Mth::sin(f64::from(-rot_y * Mth::DEG_TO_RAD - Mth::PI));
        let x_cos = -Mth::cos(f64::from(-rot_x * Mth::DEG_TO_RAD));
        let x_sin = Mth::sin(f64::from(-rot_x * Mth::DEG_TO_RAD));
        Vec3::new(
            f64::from(y_sin * x_cos),
            f64::from(x_sin),
            f64::from(y_cos * x_cos),
        )
    }

    /// The look rotation in degrees as `(pitch, yaw)`, from the real `atan2` and `asin`.
    pub fn rotation(&self) -> Vec2 {
        let yaw = Math::atan2(-self.x, self.z) as f32 * (180.0 / Mth::PI);
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        let pitch = Math::asin(-self.y / length) as f32 * (180.0 / Mth::PI);
        Vec2::new(pitch, yaw)
    }

    /// Floors the components along the given axes, Java's `EnumSet<Direction.Axis>`.
    pub fn align(&self, axes: &[Axis]) -> Vec3 {
        let x = if axes.contains(&Axis::X) {
            f64::from(Mth::floor_f64(self.x))
        } else {
            self.x
        };
        let y = if axes.contains(&Axis::Y) {
            f64::from(Mth::floor_f64(self.y))
        } else {
            self.y
        };
        let z = if axes.contains(&Axis::Z) {
            f64::from(Mth::floor_f64(self.z))
        } else {
            self.z
        };
        Vec3::new(x, y, z)
    }

    pub fn get(&self, axis: Axis) -> f64 {
        axis.choose_f64(self.x, self.y, self.z)
    }

    pub fn with(&self, axis: Axis, value: f64) -> Vec3 {
        let x = if axis == Axis::X { value } else { self.x };
        let y = if axis == Axis::Y { value } else { self.y };
        let z = if axis == Axis::Z { value } else { self.z };
        Vec3::new(x, y, z)
    }

    pub fn relative(&self, direction: Direction, distance: f64) -> Vec3 {
        let normal = direction.get_unit_vec3i();
        Vec3::new(
            self.x + distance * f64::from(normal.get_x()),
            self.y + distance * f64::from(normal.get_y()),
            self.z + distance * f64::from(normal.get_z()),
        )
    }

    pub fn to_vector3f(&self) -> Vector3f {
        Vector3f::from_xyz(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn projected_on(&self, onto: &Vec3) -> Vec3 {
        if onto.length_sqr() == 0.0 {
            *onto
        } else {
            onto.scale(self.dot(onto)).scale(1.0 / onto.length_sqr())
        }
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl Position for Vec3 {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn z(&self) -> f64 {
        self.z
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::core::position::Position;
    use crate::net::minecraft::core::vec3i::Vec3i;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
    use crate::net::minecraft::world::phys::vec2::Vec2;
    use crate::org::joml::vector3f::Vector3f;

    encode_fields!(Vec3 {
        x: f64,
        y: f64,
        z: f64
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.Vec3");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending("STREAM_CODEC", "waits for the network buffers");
        golden.pending("LP_STREAM_CODEC", "waits for the network buffers");
        golden.pending(
            "toString()Ljava/lang/String;",
            "needs Java's Double.toString, and only logs and error messages print it",
        );
        golden.field("ZERO", Vec3::ZERO);
        golden.field("X_AXIS", Vec3::X_AXIS);
        golden.field("Y_AXIS", Vec3::Y_AXIS);
        golden.field("Z_AXIS", Vec3::Z_AXIS);
        golden.check(
            "atLowerCornerOf(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i| Vec3::at_lower_corner_of(&pos),
        );
        golden.check(
            "atLowerCornerWithOffset(Lnet/minecraft/core/Vec3i;DDD)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i, x, y, z| Vec3::at_lower_corner_with_offset(&pos, x, y, z),
        );
        golden.check(
            "atCenterOf(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i| Vec3::at_center_of(&pos),
        );
        golden.check(
            "atCenterOfWithY(Lnet/minecraft/core/Vec3i;D)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i, y| Vec3::at_center_of_with_y(&pos, y),
        );
        golden.check(
            "atBottomCenterOf(Lnet/minecraft/core/Vec3i;)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i| Vec3::at_bottom_center_of(&pos),
        );
        golden.check(
            "upFromBottomCenterOf(Lnet/minecraft/core/Vec3i;D)Lnet/minecraft/world/phys/Vec3;",
            |pos: Vec3i, y_offset| Vec3::up_from_bottom_center_of(&pos, y_offset),
        );
        golden.check("<init>(DDD)V", Vec3::new);
        golden.check("<init>(Lorg/joml/Vector3fc;)V", |vec: Vector3f| {
            Vec3::from_vector3f(&vec)
        });
        golden.check("<init>(Lnet/minecraft/core/Vec3i;)V", |vec: Vec3i| {
            Vec3::from_vec3i(&vec)
        });
        golden.check_method(
            "vectorTo(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, vec: Vec3| this.vector_to(&vec),
        );
        golden.check_method(
            "normalize()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3| this.normalize(),
        );
        golden.check_method(
            "dot(Lnet/minecraft/world/phys/Vec3;)D",
            |this: &mut Vec3, vec: Vec3| this.dot(&vec),
        );
        golden.check_method(
            "cross(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, vec: Vec3| this.cross(&vec),
        );
        golden.check_method(
            "subtract(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, vec: Vec3| this.subtract(&vec),
        );
        golden.check_method(
            "subtract(D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, value| this.subtract_scalar(value),
        );
        golden.check_method(
            "subtract(DDD)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, x, y, z| this.subtract_xyz(x, y, z),
        );
        golden.check_method(
            "add(D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, value| this.add_scalar(value),
        );
        golden.check_method(
            "add(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, vec: Vec3| this.add(&vec),
        );
        golden.check_method(
            "add(DDD)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, x, y, z| this.add_xyz(x, y, z),
        );
        golden.check_method(
            "closerThan(Lnet/minecraft/core/Position;D)Z",
            |this: &mut Vec3, pos: Vec3, distance| this.closer_than(&pos, distance),
        );
        golden.check_method(
            "distanceTo(Lnet/minecraft/world/phys/Vec3;)D",
            |this: &mut Vec3, vec: Vec3| this.distance_to(&vec),
        );
        golden.check_method(
            "distanceToSqr(Lnet/minecraft/world/phys/Vec3;)D",
            |this: &mut Vec3, vec: Vec3| this.distance_to_sqr(&vec),
        );
        golden.check_method("distanceToSqr(DDD)D", |this: &mut Vec3, x, y, z| {
            this.distance_to_sqr_xyz(x, y, z)
        });
        golden.check_method(
            "closerThan(Lnet/minecraft/world/phys/Vec3;DD)Z",
            |this: &mut Vec3, vec: Vec3, distance_xz, distance_y| {
                this.closer_than_xz_y(&vec, distance_xz, distance_y)
            },
        );
        golden.check_method(
            "scale(D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, scale| this.scale(scale),
        );
        golden.check_method(
            "reverse()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3| this.reverse(),
        );
        golden.check_method(
            "multiply(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, scale: Vec3| this.multiply(&scale),
        );
        golden.check_method(
            "multiply(DDD)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, x_scale, y_scale, z_scale| {
                this.multiply_xyz(x_scale, y_scale, z_scale)
            },
        );
        golden.check_method(
            "horizontal()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3| this.horizontal(),
        );
        golden.check_mut(
            "offsetRandom(Lnet/minecraft/util/RandomSource;F)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, random: &mut LegacyRandomSource, offset: &mut f32| {
                this.offset_random(random, *offset)
            },
        );
        golden.check_mut(
            "offsetRandomXZ(Lnet/minecraft/util/RandomSource;F)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, random: &mut LegacyRandomSource, offset: &mut f32| {
                this.offset_random_xz(random, *offset)
            },
        );
        golden.check_method("length()D", |this: &mut Vec3| this.length());
        golden.check_method("lengthSqr()D", |this: &mut Vec3| this.length_sqr());
        golden.check_method("horizontalDistance()D", |this: &mut Vec3| {
            this.horizontal_distance()
        });
        golden.check_method("horizontalDistanceSqr()D", |this: &mut Vec3| {
            this.horizontal_distance_sqr()
        });
        golden.check_method("equals(Ljava/lang/Object;)Z", |this: &mut Vec3, o: Vec3| {
            *this == o
        });
        golden.check_method("hashCode()I", |this: &mut Vec3| this.hash_code());
        golden.check_method(
            "lerp(Lnet/minecraft/world/phys/Vec3;D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, vec: Vec3, a| this.lerp(&vec, a),
        );
        golden.check_method(
            "xRot(F)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, radians| this.x_rot(radians),
        );
        golden.check_method(
            "yRot(F)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, radians| this.y_rot(radians),
        );
        golden.check_method(
            "zRot(F)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, radians| this.z_rot(radians),
        );
        golden.check_method(
            "rotateClockwise90()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3| this.rotate_clockwise90(),
        );
        golden.check(
            "directionFromRotation(Lnet/minecraft/world/phys/Vec2;)Lnet/minecraft/world/phys/Vec3;",
            |rotation: Vec2| Vec3::direction_from_rotation_vec2(&rotation),
        );
        golden.check(
            "directionFromRotation(FF)Lnet/minecraft/world/phys/Vec3;",
            Vec3::direction_from_rotation,
        );
        golden.check_method(
            "rotation()Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec3| this.rotation(),
        );
        golden.check_method(
            "align(Ljava/util/EnumSet;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, axes: Vec<Axis>| this.align(&axes),
        );
        golden.check_method(
            "get(Lnet/minecraft/core/Direction$Axis;)D",
            |this: &mut Vec3, axis| this.get(axis),
        );
        golden.check_method(
            "with(Lnet/minecraft/core/Direction$Axis;D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, axis, value| this.with(axis, value),
        );
        golden.check_method(
            "relative(Lnet/minecraft/core/Direction;D)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, direction: Direction, distance| this.relative(direction, distance),
        );
        golden.check_method("x()D", |this: &mut Vec3| this.x());
        golden.check_method("y()D", |this: &mut Vec3| this.y());
        golden.check_method("z()D", |this: &mut Vec3| this.z());
        golden.check_method("toVector3f()Lorg/joml/Vector3f;", |this: &mut Vec3| {
            this.to_vector3f()
        });
        golden.check_method(
            "projectedOn(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Vec3, onto: Vec3| this.projected_on(&onto),
        );
        golden.check_method("isFinite()Z", |this: &mut Vec3| this.is_finite());
        golden.finish();
    }
}
