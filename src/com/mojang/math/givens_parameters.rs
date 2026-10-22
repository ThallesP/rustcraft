//! com/mojang/math/GivensParameters.java
//!
//! A record. Its `equals`, `hashCode` and `toString` aren't ported: nothing calls them.

use crate::org::joml::math::Math;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::quaternionf::Quaternionf;

/// A rotation by an angle around one axis, as the sine and cosine of half the angle, which is
/// how a quaternion holds it. `MatrixUtil`'s eigen and singular value decompositions are built
/// from these.
#[derive(Clone, Copy, Debug)]
pub struct GivensParameters {
    sin_half: f32,
    cos_half: f32,
}

impl GivensParameters {
    pub const fn new(sin_half: f32, cos_half: f32) -> GivensParameters {
        GivensParameters { sin_half, cos_half }
    }

    pub fn from_unnormalized(sin_half: f32, cos_half: f32) -> GivensParameters {
        let w = Math::invsqrt_f32(sin_half * sin_half + cos_half * cos_half);
        GivensParameters::new(w * sin_half, w * cos_half)
    }

    pub fn from_positive_angle(angle: f32) -> GivensParameters {
        let sin = Math::sin_f32(angle / 2.0);
        let cos = Math::cos_from_sin_f32(sin, angle / 2.0);
        GivensParameters::new(sin, cos)
    }

    pub const fn sin_half(&self) -> f32 {
        self.sin_half
    }

    pub const fn cos_half(&self) -> f32 {
        self.cos_half
    }

    pub fn inverse(&self) -> GivensParameters {
        GivensParameters::new(-self.sin_half, self.cos_half)
    }

    pub fn around_x_quaternionf<'a>(&self, input: &'a mut Quaternionf) -> &'a mut Quaternionf {
        input.set_xyzw(self.sin_half, 0.0, 0.0, self.cos_half)
    }

    pub fn around_y_quaternionf<'a>(&self, input: &'a mut Quaternionf) -> &'a mut Quaternionf {
        input.set_xyzw(0.0, self.sin_half, 0.0, self.cos_half)
    }

    pub fn around_z_quaternionf<'a>(&self, input: &'a mut Quaternionf) -> &'a mut Quaternionf {
        input.set_xyzw(0.0, 0.0, self.sin_half, self.cos_half)
    }

    /// The cosine of the whole angle.
    pub fn cos(&self) -> f32 {
        self.cos_half * self.cos_half - self.sin_half * self.sin_half
    }

    /// The sine of the whole angle.
    pub fn sin(&self) -> f32 {
        2.0 * self.sin_half * self.cos_half
    }

    pub fn around_x_matrix3f<'a>(&self, input: &'a mut Matrix3f) -> &'a mut Matrix3f {
        input.m01 = 0.0;
        input.m02 = 0.0;
        input.m10 = 0.0;
        input.m20 = 0.0;
        let c = self.cos();
        let s = self.sin();
        input.m11 = c;
        input.m22 = c;
        input.m12 = s;
        input.m21 = -s;
        input.m00 = 1.0;
        input
    }

    pub fn around_y_matrix3f<'a>(&self, input: &'a mut Matrix3f) -> &'a mut Matrix3f {
        input.m01 = 0.0;
        input.m10 = 0.0;
        input.m12 = 0.0;
        input.m21 = 0.0;
        let c = self.cos();
        let s = self.sin();
        input.m00 = c;
        input.m22 = c;
        input.m02 = -s;
        input.m20 = s;
        input.m11 = 1.0;
        input
    }

    pub fn around_z_matrix3f<'a>(&self, input: &'a mut Matrix3f) -> &'a mut Matrix3f {
        input.m02 = 0.0;
        input.m12 = 0.0;
        input.m20 = 0.0;
        input.m21 = 0.0;
        let c = self.cos();
        let s = self.sin();
        input.m00 = c;
        input.m11 = c;
        input.m01 = s;
        input.m10 = -s;
        input.m22 = 1.0;
        input
    }
}

#[cfg(test)]
mod tests {
    use super::GivensParameters;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix3f::Matrix3f;
    use crate::org::joml::quaternionf::Quaternionf;

    encode_fields!(GivensParameters {
        sin_half: f32,
        cos_half: f32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.GivensParameters");
        golden.check("<init>(FF)V", GivensParameters::new);
        golden.check(
            "fromUnnormalized(FF)Lcom/mojang/math/GivensParameters;",
            GivensParameters::from_unnormalized,
        );
        golden.check(
            "fromPositiveAngle(F)Lcom/mojang/math/GivensParameters;",
            GivensParameters::from_positive_angle,
        );
        golden.check_method("sinHalf()F", |this: &mut GivensParameters| this.sin_half());
        golden.check_method("cosHalf()F", |this: &mut GivensParameters| this.cos_half());
        golden.check_method(
            "inverse()Lcom/mojang/math/GivensParameters;",
            |this: &mut GivensParameters| this.inverse(),
        );
        golden.check_method("cos()F", |this: &mut GivensParameters| this.cos());
        golden.check_method("sin()F", |this: &mut GivensParameters| this.sin());
        golden.check_method(
            "aroundX(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut GivensParameters, mut input: Quaternionf| {
                *this.around_x_quaternionf(&mut input)
            },
        );
        golden.check_method(
            "aroundY(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut GivensParameters, mut input: Quaternionf| {
                *this.around_y_quaternionf(&mut input)
            },
        );
        golden.check_method(
            "aroundZ(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut GivensParameters, mut input: Quaternionf| {
                *this.around_z_quaternionf(&mut input)
            },
        );
        golden.check_method(
            "aroundX(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut GivensParameters, mut input: Matrix3f| *this.around_x_matrix3f(&mut input),
        );
        golden.check_method(
            "aroundY(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut GivensParameters, mut input: Matrix3f| *this.around_y_matrix3f(&mut input),
        );
        golden.check_method(
            "aroundZ(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut GivensParameters, mut input: Matrix3f| *this.around_z_matrix3f(&mut input),
        );
        golden.finish();
    }
}
