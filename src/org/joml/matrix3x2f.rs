//! org/joml/Matrix3x2f.java (JOML 1.10.9)
//!
//! A 2D affine transform: a 3x3 matrix whose last row is `(0, 0, 1)`, column-major like the
//! others, so `m20` and `m21` are the translation. It's what the GUI draws with
//! (`GuiGraphicsExtractor.pose()` is a `Matrix3x2fStack`). Only the methods the client calls so
//! far: the golden suite in oracle/golden/ lists them.
//!
//! A fluent overload passes `this` as JOML's `dest`, so it's ported as writing into `self` from a
//! copy of it: none of these reads a field of `this` after writing it to `dest`.

use crate::org::joml::math::Math;
use crate::org::joml::vector2f::Vector2f;

#[derive(Clone, Copy, Debug)]
pub struct Matrix3x2f {
    pub m00: f32,
    pub m01: f32,
    pub m10: f32,
    pub m11: f32,
    pub m20: f32,
    pub m21: f32,
}

impl Default for Matrix3x2f {
    fn default() -> Matrix3x2f {
        Matrix3x2f::new()
    }
}

impl Matrix3x2f {
    /// The identity.
    pub const fn new() -> Matrix3x2f {
        Matrix3x2f {
            m00: 1.0,
            m01: 0.0,
            m10: 0.0,
            m11: 1.0,
            m20: 0.0,
            m21: 0.0,
        }
    }

    pub fn from_matrix3x2f(mat: &Matrix3x2f) -> Matrix3x2f {
        *mat
    }

    pub fn set(&mut self, m: &Matrix3x2f) -> &mut Self {
        *self = *m;
        self
    }

    /// Doesn't check that the matrix can be inverted: a zero determinant gives infinities and
    /// NaNs.
    pub fn invert_into<'a>(&self, dest: &'a mut Matrix3x2f) -> &'a mut Matrix3x2f {
        let s = 1.0 / (self.m00 * self.m11 - self.m01 * self.m10);
        let nm00 = self.m11 * s;
        let nm01 = -self.m01 * s;
        let nm10 = -self.m10 * s;
        let nm11 = self.m00 * s;
        let nm20 = (self.m10 * self.m21 - self.m20 * self.m11) * s;
        let nm21 = (self.m20 * self.m01 - self.m00 * self.m21) * s;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest
    }

    pub fn translate(&mut self, x: f32, y: f32) -> &mut Self {
        let this = *self;
        this.translate_into(x, y, self)
    }

    pub fn translate_into<'a>(
        &self,
        x: f32,
        y: f32,
        dest: &'a mut Matrix3x2f,
    ) -> &'a mut Matrix3x2f {
        let rm20 = x;
        let rm21 = y;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21;
        dest.m00 = self.m00;
        dest.m01 = self.m01;
        dest.m10 = self.m10;
        dest.m11 = self.m11;
        dest
    }

    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        let this = *self;
        this.scale_into(x, y, self)
    }

    pub fn scale_into<'a>(&self, x: f32, y: f32, dest: &'a mut Matrix3x2f) -> &'a mut Matrix3x2f {
        dest.m00 = self.m00 * x;
        dest.m01 = self.m01 * x;
        dest.m10 = self.m10 * y;
        dest.m11 = self.m11 * y;
        dest.m20 = self.m20;
        dest.m21 = self.m21;
        dest
    }

    pub fn scale_scalar(&mut self, xy: f32) -> &mut Self {
        self.scale(xy, xy)
    }

    pub fn scale_scalar_into<'a>(&self, xy: f32, dest: &'a mut Matrix3x2f) -> &'a mut Matrix3x2f {
        self.scale_into(xy, xy, dest)
    }

    pub fn rotate(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_into(ang, self)
    }

    pub fn rotate_into<'a>(&self, ang: f32, dest: &'a mut Matrix3x2f) -> &'a mut Matrix3x2f {
        let cos = Math::cos_f32(ang);
        let sin = Math::sin_f32(ang);
        let rm00 = cos;
        let rm01 = sin;
        let rm10 = -sin;
        let rm11 = cos;
        let nm00 = self.m00 * rm00 + self.m10 * rm01;
        let nm01 = self.m01 * rm00 + self.m11 * rm01;
        dest.m10 = self.m00 * rm10 + self.m10 * rm11;
        dest.m11 = self.m01 * rm10 + self.m11 * rm11;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m20 = self.m20;
        dest.m21 = self.m21;
        dest
    }

    pub fn transform_position<'a>(&self, v: &'a mut Vector2f) -> &'a mut Vector2f {
        v.set_xy(
            self.m00 * v.x + self.m10 * v.y + self.m20,
            self.m01 * v.x + self.m11 * v.y + self.m21,
        )
    }

    pub fn transform_position_xy_into<'a>(
        &self,
        x: f32,
        y: f32,
        dest: &'a mut Vector2f,
    ) -> &'a mut Vector2f {
        dest.set_xy(
            self.m00 * x + self.m10 * y + self.m20,
            self.m01 * x + self.m11 * y + self.m21,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix3x2f;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::vector2f::Vector2f;

    encode_fields!(Matrix3x2f {
        m00: f32,
        m01: f32,
        m10: f32,
        m11: f32,
        m20: f32,
        m21: f32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix3x2f");
        golden.check("<init>()V", Matrix3x2f::new);
        golden.check("<init>(Lorg/joml/Matrix3x2fc;)V", |mat: Matrix3x2f| {
            Matrix3x2f::from_matrix3x2f(&mat)
        });
        golden.check_method(
            "set(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, m: Matrix3x2f| *this.set(&m),
        );
        golden.check_method(
            "translate(FF)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, x, y| *this.translate(x, y),
        );
        golden.check_method(
            "translate(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, x, y, mut dest: Matrix3x2f| {
                *this.translate_into(x, y, &mut dest)
            },
        );
        golden.check_method(
            "scale(FF)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, x, y| *this.scale(x, y),
        );
        golden.check_method(
            "scale(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, x, y, mut dest: Matrix3x2f| *this.scale_into(x, y, &mut dest),
        );
        golden.check_method(
            "scale(F)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, xy| *this.scale_scalar(xy),
        );
        golden.check_method(
            "scale(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, xy, mut dest: Matrix3x2f| {
                *this.scale_scalar_into(xy, &mut dest)
            },
        );
        golden.check_method(
            "rotate(F)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, ang| *this.rotate(ang),
        );
        golden.check_method(
            "rotate(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, ang, mut dest: Matrix3x2f| *this.rotate_into(ang, &mut dest),
        );
        golden.check_method(
            "invert(Lorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            |this: &mut Matrix3x2f, mut dest: Matrix3x2f| *this.invert_into(&mut dest),
        );
        golden.check_method(
            "transformPosition(Lorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            |this: &mut Matrix3x2f, mut v: Vector2f| *this.transform_position(&mut v),
        );
        golden.check_method(
            "transformPosition(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            |this: &mut Matrix3x2f, x, y, mut dest: Vector2f| {
                *this.transform_position_xy_into(x, y, &mut dest)
            },
        );
        golden.finish();
    }
}
