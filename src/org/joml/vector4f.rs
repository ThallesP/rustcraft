//! org/joml/Vector4f.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.

use crate::org::joml::math::Math;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4fc::{PROPERTY_AFFINE, PROPERTY_IDENTITY, PROPERTY_TRANSLATION};
use crate::org::joml::vector3f::Vector3f;

#[derive(Clone, Copy, Debug)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Vector4f {
    fn default() -> Vector4f {
        Vector4f::new()
    }
}

impl Vector4f {
    /// `(0, 0, 0, 1)`.
    pub const fn new() -> Vector4f {
        Vector4f::from_xyzw(0.0, 0.0, 0.0, 1.0)
    }

    pub const fn from_scalar(d: f32) -> Vector4f {
        Vector4f::from_xyzw(d, d, d, d)
    }

    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Vector4f {
        Vector4f { x, y, z, w }
    }

    pub fn from_vector3f(v: &Vector3f, w: f32) -> Vector4f {
        Vector4f::from_xyzw(v.x, v.y, v.z, w)
    }

    pub fn from_vector4f(v: &Vector4f) -> Vector4f {
        Vector4f::from_xyzw(v.x, v.y, v.z, v.w)
    }

    pub fn set(&mut self, v: &Vector4f) -> &mut Self {
        self.set_xyzw(v.x, v.y, v.z, v.w)
    }

    pub fn set_vector3f(&mut self, v: &Vector3f, w: f32) -> &mut Self {
        self.set_xyzw(v.x, v.y, v.z, w)
    }

    pub fn set_xyzw(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }

    pub fn mul(&mut self, v: &Vector4f) -> &mut Self {
        let this = *self;
        this.mul_into(v, self)
    }

    pub fn mul_into<'a>(&self, v: &Vector4f, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        dest.x = self.x * v.x;
        dest.y = self.y * v.y;
        dest.z = self.z * v.z;
        dest.w = self.w * v.w;
        dest
    }

    pub fn mul_matrix4f(&mut self, mat: &Matrix4f) -> &mut Self {
        let this = *self;
        this.mul_matrix4f_into(mat, self)
    }

    pub fn mul_matrix4f_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return dest.set(self);
        }
        if (prop & PROPERTY_TRANSLATION) != 0 {
            return self.mul_translation_into(mat, dest);
        }
        if (prop & PROPERTY_AFFINE) != 0 {
            return self.mul_affine_into(mat, dest);
        }
        self.mul_generic_into(mat, dest)
    }

    fn mul_translation_into<'a>(&self, mat: &Matrix4f, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.x = Math::fma_f32(mat.m30(), w, x);
        dest.y = Math::fma_f32(mat.m31(), w, y);
        dest.z = Math::fma_f32(mat.m32(), w, z);
        dest.w = w;
        dest
    }

    fn mul_affine_into<'a>(&self, mat: &Matrix4f, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.x = Math::fma_f32(
            mat.m00(),
            x,
            Math::fma_f32(mat.m10(), y, Math::fma_f32(mat.m20(), z, mat.m30() * w)),
        );
        dest.y = Math::fma_f32(
            mat.m01(),
            x,
            Math::fma_f32(mat.m11(), y, Math::fma_f32(mat.m21(), z, mat.m31() * w)),
        );
        dest.z = Math::fma_f32(
            mat.m02(),
            x,
            Math::fma_f32(mat.m12(), y, Math::fma_f32(mat.m22(), z, mat.m32() * w)),
        );
        dest.w = w;
        dest
    }

    fn mul_generic_into<'a>(&self, mat: &Matrix4f, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.x = Math::fma_f32(
            mat.m00(),
            x,
            Math::fma_f32(mat.m10(), y, Math::fma_f32(mat.m20(), z, mat.m30() * w)),
        );
        dest.y = Math::fma_f32(
            mat.m01(),
            x,
            Math::fma_f32(mat.m11(), y, Math::fma_f32(mat.m21(), z, mat.m31() * w)),
        );
        dest.z = Math::fma_f32(
            mat.m02(),
            x,
            Math::fma_f32(mat.m12(), y, Math::fma_f32(mat.m22(), z, mat.m32() * w)),
        );
        dest.w = Math::fma_f32(
            mat.m03(),
            x,
            Math::fma_f32(mat.m13(), y, Math::fma_f32(mat.m23(), z, mat.m33() * w)),
        );
        dest
    }

    pub fn mul_transpose(&mut self, mat: &Matrix4f) -> &mut Self {
        let this = *self;
        this.mul_transpose_into(mat, self)
    }

    pub fn mul_transpose_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return dest.set(self);
        }
        if (prop & PROPERTY_AFFINE) != 0 {
            return self.mul_affine_transpose_into(mat, dest);
        }
        self.mul_generic_transpose_into(mat, dest)
    }

    fn mul_affine_transpose_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.x = Math::fma_f32(mat.m00(), x, Math::fma_f32(mat.m01(), y, mat.m02() * z));
        dest.y = Math::fma_f32(mat.m10(), x, Math::fma_f32(mat.m11(), y, mat.m12() * z));
        dest.z = Math::fma_f32(mat.m20(), x, Math::fma_f32(mat.m21(), y, mat.m22() * z));
        dest.w = Math::fma_f32(mat.m30(), x, Math::fma_f32(mat.m31(), y, mat.m32() * z + w));
        dest
    }

    fn mul_generic_transpose_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.x = Math::fma_f32(
            mat.m00(),
            x,
            Math::fma_f32(mat.m01(), y, Math::fma_f32(mat.m02(), z, mat.m03() * w)),
        );
        dest.y = Math::fma_f32(
            mat.m10(),
            x,
            Math::fma_f32(mat.m11(), y, Math::fma_f32(mat.m12(), z, mat.m13() * w)),
        );
        dest.z = Math::fma_f32(
            mat.m20(),
            x,
            Math::fma_f32(mat.m21(), y, Math::fma_f32(mat.m22(), z, mat.m23() * w)),
        );
        dest.w = Math::fma_f32(
            mat.m30(),
            x,
            Math::fma_f32(mat.m31(), y, Math::fma_f32(mat.m32(), z, mat.m33() * w)),
        );
        dest
    }

    pub fn mul_scalar(&mut self, scalar: f32) -> &mut Self {
        let this = *self;
        this.mul_scalar_into(scalar, self)
    }

    pub fn mul_scalar_into<'a>(&self, scalar: f32, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        dest.x = self.x * scalar;
        dest.y = self.y * scalar;
        dest.z = self.z * scalar;
        dest.w = self.w * scalar;
        dest
    }

    pub fn div_scalar(&mut self, scalar: f32) -> &mut Self {
        let this = *self;
        this.div_scalar_into(scalar, self)
    }

    /// Multiplies by the reciprocal, which can round differently from dividing.
    pub fn div_scalar_into<'a>(&self, scalar: f32, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        let inv = 1.0 / scalar;
        dest.x = self.x * inv;
        dest.y = self.y * inv;
        dest.z = self.z * inv;
        dest.w = self.w * inv;
        dest
    }

    pub fn lerp(&mut self, other: &Vector4f, t: f32) -> &mut Self {
        let this = *self;
        this.lerp_into(other, t, self)
    }

    pub fn lerp_into<'a>(
        &self,
        other: &Vector4f,
        t: f32,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        dest.x = Math::fma_f32(other.x - self.x, t, self.x);
        dest.y = Math::fma_f32(other.y - self.y, t, self.y);
        dest.z = Math::fma_f32(other.z - self.z, t, self.z);
        dest.w = Math::fma_f32(other.w - self.w, t, self.w);
        dest
    }
}

#[cfg(test)]
mod tests {
    use super::Vector4f;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::vector3f::Vector3f;

    encode_fields!(Vector4f {
        x: f32,
        y: f32,
        z: f32,
        w: f32
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Vector4f");
        golden.check("<init>()V", Vector4f::new);
        golden.check("<init>(F)V", Vector4f::from_scalar);
        golden.check("<init>(FFFF)V", Vector4f::from_xyzw);
        golden.check("<init>(Lorg/joml/Vector3fc;F)V", |v: Vector3f, w| {
            Vector4f::from_vector3f(&v, w)
        });
        golden.check("<init>(Lorg/joml/Vector4fc;)V", |v: Vector4f| {
            Vector4f::from_vector4f(&v)
        });
        golden.check_method(
            "set(FFFF)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, x, y, z, w| *this.set_xyzw(x, y, z, w),
        );
        golden.check_method(
            "set(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, v: Vector4f| *this.set(&v),
        );
        golden.check_method(
            "set(Lorg/joml/Vector3fc;F)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, v: Vector3f, w| *this.set_vector3f(&v, w),
        );
        golden.check_method(
            "mul(F)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, scalar| *this.mul_scalar(scalar),
        );
        golden.check_method(
            "mul(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, scalar, mut dest: Vector4f| {
                *this.mul_scalar_into(scalar, &mut dest)
            },
        );
        golden.check_method(
            "mul(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, v: Vector4f| *this.mul(&v),
        );
        golden.check_method(
            "mul(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, v: Vector4f, mut dest: Vector4f| *this.mul_into(&v, &mut dest),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, mat: Matrix4f| *this.mul_matrix4f(&mat),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, mat: Matrix4f, mut dest: Vector4f| {
                *this.mul_matrix4f_into(&mat, &mut dest)
            },
        );
        golden.check_method(
            "mulTranspose(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, mat: Matrix4f| *this.mul_transpose(&mat),
        );
        golden.check_method(
            "mulTranspose(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, mat: Matrix4f, mut dest: Vector4f| {
                *this.mul_transpose_into(&mat, &mut dest)
            },
        );
        golden.check_method(
            "div(F)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, scalar| *this.div_scalar(scalar),
        );
        golden.check_method(
            "div(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, scalar, mut dest: Vector4f| {
                *this.div_scalar_into(scalar, &mut dest)
            },
        );
        golden.check_method(
            "lerp(Lorg/joml/Vector4fc;F)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, other: Vector4f, t| *this.lerp(&other, t),
        );
        golden.check_method(
            "lerp(Lorg/joml/Vector4fc;FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Vector4f, other: Vector4f, t, mut dest: Vector4f| {
                *this.lerp_into(&other, t, &mut dest)
            },
        );
        golden.finish();
    }
}
