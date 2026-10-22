//! net/minecraft/world/phys/Vec2.java
//!
//! `CODEC` waits for DataFixerUpper.

use crate::java::lang::float::Float;
use crate::net::minecraft::util::mth::Mth;

/// Two floats: a rotation (`x` up and down, `y` around) or a point on a plane.
///
/// Java's `equals` compares the components with `==`, like the derived `PartialEq`, so -0.0
/// equals 0.0 and NaN equals nothing. Java also returns true for the same object, which only
/// differs for a vector with a NaN in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
    pub const ONE: Vec2 = Vec2::new(1.0, 1.0);
    pub const UNIT_X: Vec2 = Vec2::new(1.0, 0.0);
    pub const NEG_UNIT_X: Vec2 = Vec2::new(-1.0, 0.0);
    pub const UNIT_Y: Vec2 = Vec2::new(0.0, 1.0);
    pub const NEG_UNIT_Y: Vec2 = Vec2::new(0.0, -1.0);
    pub const MAX: Vec2 = Vec2::new(f32::MAX, f32::MAX);
    /// `Float.MIN_VALUE`, the smallest positive float, in both.
    pub const MIN: Vec2 = Vec2::new(f32::from_bits(1), f32::from_bits(1));

    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn scale(&self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    pub fn dot(&self, v: &Vec2) -> f32 {
        self.x * v.x + self.y * v.y
    }

    pub fn add(&self, rhs: &Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }

    pub fn add_scalar(&self, v: f32) -> Vec2 {
        Vec2::new(self.x + v, self.y + v)
    }

    /// `Objects.hash(x, y)`: 31 times (31 plus the hash of x), plus the hash of y.
    pub fn hash_code(&self) -> i32 {
        31i32
            .wrapping_add(Float::hash_code(self.x))
            .wrapping_mul(31)
            .wrapping_add(Float::hash_code(self.y))
    }

    pub fn normalized(&self) -> Vec2 {
        let dist = Mth::sqrt(self.x * self.x + self.y * self.y);
        if dist < 1.0E-4 {
            Vec2::ZERO
        } else {
            Vec2::new(self.x / dist, self.y / dist)
        }
    }

    pub fn length(&self) -> f32 {
        Mth::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn distance_to_sqr(&self, p: &Vec2) -> f32 {
        let xd = p.x - self.x;
        let yd = p.y - self.y;
        xd * xd + yd * yd
    }

    pub fn negated(&self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }

    /// Counterclockwise, with `Mth`'s table sine and cosine.
    pub fn rotate(&self, angle_radians: f64) -> Vec2 {
        let cosine = Mth::cos(angle_radians);
        let sine = Mth::sin(angle_radians);
        Vec2::new(
            self.x * cosine - self.y * sine,
            self.y * cosine + self.x * sine,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Vec2;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(Vec2 { x: f32, y: f32 });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.Vec2");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.field("ZERO", Vec2::ZERO);
        golden.field("ONE", Vec2::ONE);
        golden.field("UNIT_X", Vec2::UNIT_X);
        golden.field("NEG_UNIT_X", Vec2::NEG_UNIT_X);
        golden.field("UNIT_Y", Vec2::UNIT_Y);
        golden.field("NEG_UNIT_Y", Vec2::NEG_UNIT_Y);
        golden.field("MAX", Vec2::MAX);
        golden.field("MIN", Vec2::MIN);
        golden.check("<init>(FF)V", Vec2::new);
        golden.check_method(
            "scale(F)Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2, s| this.scale(s),
        );
        golden.check_method(
            "dot(Lnet/minecraft/world/phys/Vec2;)F",
            |this: &mut Vec2, v: Vec2| this.dot(&v),
        );
        golden.check_method(
            "add(Lnet/minecraft/world/phys/Vec2;)Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2, rhs: Vec2| this.add(&rhs),
        );
        golden.check_method(
            "add(F)Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2, v| this.add_scalar(v),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Vec2, rhs: Vec2| *this == rhs,
        );
        golden.check_method("hashCode()I", |this: &mut Vec2| this.hash_code());
        golden.check_method(
            "normalized()Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2| this.normalized(),
        );
        golden.check_method("length()F", |this: &mut Vec2| this.length());
        golden.check_method("lengthSquared()F", |this: &mut Vec2| this.length_squared());
        golden.check_method(
            "distanceToSqr(Lnet/minecraft/world/phys/Vec2;)F",
            |this: &mut Vec2, p: Vec2| this.distance_to_sqr(&p),
        );
        golden.check_method(
            "negated()Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2| this.negated(),
        );
        golden.check_method(
            "rotate(D)Lnet/minecraft/world/phys/Vec2;",
            |this: &mut Vec2, angle_radians| this.rotate(angle_radians),
        );
        golden.finish();
    }
}
