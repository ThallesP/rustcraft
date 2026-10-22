//! com/mojang/math/Axis.java
//!
//! An interface in Java, implemented by its six constants and by what `Axis.of(axis)` returns.
//! Nothing in the game calls `of`, so it isn't ported, and the six constants become an enum.

use std::f64::consts::PI;

use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;

// `(float) (Math.PI / 180.0)`, which each `*Degrees` method multiplies by.
const DEGREES_TO_RADIANS: f32 = (PI / 180.0) as f32;

/// A rotation around -X, +X, -Y, +Y, -Z or +Z.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    XN,
    XP,
    YN,
    YP,
    ZN,
    ZP,
}

impl Axis {
    pub fn rotation(&self, angle: f32) -> Quaternionf {
        let mut rotation = Quaternionf::new();
        match self {
            Axis::XN => rotation.rotation_x(-angle),
            Axis::XP => rotation.rotation_x(angle),
            Axis::YN => rotation.rotation_y(-angle),
            Axis::YP => rotation.rotation_y(angle),
            Axis::ZN => rotation.rotation_z(-angle),
            Axis::ZP => rotation.rotation_z(angle),
        };
        rotation
    }

    pub fn rotation_degrees(&self, angle: f32) -> Quaternionf {
        self.rotation(angle * DEGREES_TO_RADIANS)
    }

    pub fn rotate_matrix3f<'a>(&self, matrix: &'a mut Matrix3f, angle: f32) -> &'a mut Matrix3f {
        match self {
            Axis::XN => matrix.rotate_x(-angle),
            Axis::XP => matrix.rotate_x(angle),
            Axis::YN => matrix.rotate_y(-angle),
            Axis::YP => matrix.rotate_y(angle),
            Axis::ZN => matrix.rotate_z(-angle),
            Axis::ZP => matrix.rotate_z(angle),
        }
    }

    pub fn rotate_degrees_matrix3f<'a>(
        &self,
        matrix: &'a mut Matrix3f,
        angle: f32,
    ) -> &'a mut Matrix3f {
        self.rotate_matrix3f(matrix, angle * DEGREES_TO_RADIANS)
    }

    pub fn rotate_matrix4f<'a>(&self, matrix: &'a mut Matrix4f, angle: f32) -> &'a mut Matrix4f {
        match self {
            Axis::XN => matrix.rotate_x(-angle),
            Axis::XP => matrix.rotate_x(angle),
            Axis::YN => matrix.rotate_y(-angle),
            Axis::YP => matrix.rotate_y(angle),
            Axis::ZN => matrix.rotate_z(-angle),
            Axis::ZP => matrix.rotate_z(angle),
        }
    }

    pub fn rotate_degrees_matrix4f<'a>(
        &self,
        matrix: &'a mut Matrix4f,
        angle: f32,
    ) -> &'a mut Matrix4f {
        self.rotate_matrix4f(matrix, angle * DEGREES_TO_RADIANS)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Axis;
    use crate::golden::{Decode, Encode, Golden};
    use crate::org::joml::matrix3f::Matrix3f;
    use crate::org::joml::matrix4f::Matrix4f;

    /// Vanilla encodes the constants by name.
    impl Decode for Axis {
        fn decode(value: &Value) -> Self {
            match value.as_str() {
                Some("XN") => Axis::XN,
                Some("XP") => Axis::XP,
                Some("YN") => Axis::YN,
                Some("YP") => Axis::YP,
                Some("ZN") => Axis::ZN,
                Some("ZP") => Axis::ZP,
                _ => panic!("expected an axis, got {value}"),
            }
        }
    }

    impl Encode for Axis {
        fn encode(&self) -> Value {
            Value::from(format!("{self:?}"))
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.Axis");
        golden.check_method(
            "rotation(F)Lorg/joml/Quaternionf;",
            |this: &mut Axis, angle| this.rotation(angle),
        );
        golden.check_method(
            "rotationDegrees(F)Lorg/joml/Quaternionf;",
            |this: &mut Axis, angle| this.rotation_degrees(angle),
        );
        golden.check_method(
            "rotate(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;",
            |this: &mut Axis, mut matrix: Matrix3f, angle| {
                *this.rotate_matrix3f(&mut matrix, angle)
            },
        );
        golden.check_method(
            "rotateDegrees(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;",
            |this: &mut Axis, mut matrix: Matrix3f, angle| {
                *this.rotate_degrees_matrix3f(&mut matrix, angle)
            },
        );
        golden.check_method(
            "rotate(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;",
            |this: &mut Axis, mut matrix: Matrix4f, angle| {
                *this.rotate_matrix4f(&mut matrix, angle)
            },
        );
        golden.check_method(
            "rotateDegrees(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;",
            |this: &mut Axis, mut matrix: Matrix4f, angle| {
                *this.rotate_degrees_matrix4f(&mut matrix, angle)
            },
        );
        golden.finish();
    }
}
