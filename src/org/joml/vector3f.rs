//! org/joml/Vector3f.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.

use crate::org::joml::math::Math;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4fc::{PROPERTY_AFFINE, PROPERTY_IDENTITY, PROPERTY_TRANSLATION};
use crate::org::joml::quaternionf::Quaternionf;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub const fn new() -> Vector3f {
        Vector3f::from_xyz(0.0, 0.0, 0.0)
    }

    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Vector3f {
        Vector3f { x, y, z }
    }

    pub fn from_vector3f(v: &Vector3f) -> Vector3f {
        Vector3f::from_xyz(v.x, v.y, v.z)
    }

    pub fn set(&mut self, v: &Vector3f) -> &mut Self {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
        self
    }

    pub fn set_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn zero(&mut self) -> &mut Self {
        self.set_xyz(0.0, 0.0, 0.0)
    }

    pub fn sub(&mut self, v: &Vector3f) -> &mut Self {
        self.sub_xyz(v.x, v.y, v.z)
    }

    pub fn sub_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self
    }

    pub fn add(&mut self, v: &Vector3f) -> &mut Self {
        self.add_xyz(v.x, v.y, v.z)
    }

    pub fn add_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    pub fn mul(&mut self, v: &Vector3f) -> &mut Self {
        self.mul_xyz(v.x, v.y, v.z)
    }

    pub fn mul_matrix3f(&mut self, mat: &Matrix3f) -> &mut Self {
        let this = *self;
        this.mul_matrix3f_into(mat, self)
    }

    pub fn mul_matrix3f_into<'a>(
        &self,
        mat: &Matrix3f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (lx, ly, lz) = (self.x, self.y, self.z);
        dest.x = Math::fma_f32(mat.m00, lx, Math::fma_f32(mat.m10, ly, mat.m20 * lz));
        dest.y = Math::fma_f32(mat.m01, lx, Math::fma_f32(mat.m11, ly, mat.m21 * lz));
        dest.z = Math::fma_f32(mat.m02, lx, Math::fma_f32(mat.m12, ly, mat.m22 * lz));
        dest
    }

    pub fn mul_scalar(&mut self, scalar: f32) -> &mut Self {
        self.mul_xyz(scalar, scalar, scalar)
    }

    /// Multiplies by the scalar's reciprocal, like JOML.
    pub fn div_scalar(&mut self, scalar: f32) -> &mut Self {
        let inv = 1.0 / scalar;
        self.x *= inv;
        self.y *= inv;
        self.z *= inv;
        self
    }

    /// Java throws `IllegalArgumentException` past 2.
    pub fn get(&self, component: i32) -> f32 {
        match component {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("IllegalArgumentException"),
        }
    }

    pub fn is_finite(&self) -> bool {
        Math::is_finite_f32(self.x) && Math::is_finite_f32(self.y) && Math::is_finite_f32(self.z)
    }

    pub fn mul_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x *= x;
        self.y *= y;
        self.z *= z;
        self
    }

    pub fn mul_project(&mut self, mat: &Matrix4f) -> &mut Self {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return self;
        }
        let this = *self;
        if (prop & PROPERTY_TRANSLATION) != 0 {
            return this.mul_project_translation_into(mat, self);
        }
        if (prop & PROPERTY_AFFINE) != 0 {
            return this.mul_project_affine_into(mat, self);
        }
        this.mul_project_generic_into(mat, self)
    }

    pub fn mul_project_into<'a>(&self, mat: &Matrix4f, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return dest.set(self);
        }
        if (prop & PROPERTY_TRANSLATION) != 0 {
            return self.mul_project_translation_into(mat, dest);
        }
        if (prop & PROPERTY_AFFINE) != 0 {
            return self.mul_project_affine_into(mat, dest);
        }
        self.mul_project_generic_into(mat, dest)
    }

    fn mul_project_translation_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        self.mul_position_translation_into(mat, dest)
    }

    fn mul_project_affine_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (x, y, z) = (self.x, self.y, self.z);
        dest.x =
            Math::fma_f32(mat.m00(), x, Math::fma_f32(mat.m10(), y, mat.m20() * z)) + mat.m30();
        dest.y =
            Math::fma_f32(mat.m01(), x, Math::fma_f32(mat.m11(), y, mat.m21() * z)) + mat.m31();
        dest.z =
            Math::fma_f32(mat.m02(), x, Math::fma_f32(mat.m12(), y, mat.m22() * z)) + mat.m32();
        dest
    }

    fn mul_project_generic_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (x, y, z) = (self.x, self.y, self.z);
        let inv_w = 1.0
            / Math::fma_f32(
                mat.m03(),
                x,
                Math::fma_f32(mat.m13(), y, Math::fma_f32(mat.m23(), z, mat.m33())),
            );
        dest.x = Math::fma_f32(
            mat.m00(),
            x,
            Math::fma_f32(mat.m10(), y, Math::fma_f32(mat.m20(), z, mat.m30())),
        ) * inv_w;
        dest.y = Math::fma_f32(
            mat.m01(),
            x,
            Math::fma_f32(mat.m11(), y, Math::fma_f32(mat.m21(), z, mat.m31())),
        ) * inv_w;
        dest.z = Math::fma_f32(
            mat.m02(),
            x,
            Math::fma_f32(mat.m12(), y, Math::fma_f32(mat.m22(), z, mat.m32())),
        ) * inv_w;
        dest
    }

    pub fn mul_position(&mut self, mat: &Matrix4f) -> &mut Self {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return self;
        }
        let this = *self;
        if (prop & PROPERTY_TRANSLATION) != 0 {
            return this.mul_position_translation_into(mat, self);
        }
        this.mul_position_generic_into(mat, self)
    }

    pub fn mul_position_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let prop = mat.properties();
        if (prop & PROPERTY_IDENTITY) != 0 {
            return dest.set(self);
        }
        if (prop & PROPERTY_TRANSLATION) != 0 {
            return self.mul_position_translation_into(mat, dest);
        }
        self.mul_position_generic_into(mat, dest)
    }

    fn mul_position_translation_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.x = self.x + mat.m30();
        dest.y = self.y + mat.m31();
        dest.z = self.z + mat.m32();
        dest
    }

    fn mul_position_generic_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (x, y, z) = (self.x, self.y, self.z);
        dest.x = Math::fma_f32(
            mat.m00(),
            x,
            Math::fma_f32(mat.m10(), y, Math::fma_f32(mat.m20(), z, mat.m30())),
        );
        dest.y = Math::fma_f32(
            mat.m01(),
            x,
            Math::fma_f32(mat.m11(), y, Math::fma_f32(mat.m21(), z, mat.m31())),
        );
        dest.z = Math::fma_f32(
            mat.m02(),
            x,
            Math::fma_f32(mat.m12(), y, Math::fma_f32(mat.m22(), z, mat.m32())),
        );
        dest
    }

    pub fn mul_direction(&mut self, mat: &Matrix4f) -> &mut Self {
        let this = *self;
        this.mul_direction_into(mat, self)
    }

    pub fn mul_direction_into<'a>(
        &self,
        mat: &Matrix4f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (x, y, z) = (self.x, self.y, self.z);
        dest.x = Math::fma_f32(mat.m00(), x, Math::fma_f32(mat.m10(), y, mat.m20() * z));
        dest.y = Math::fma_f32(mat.m01(), x, Math::fma_f32(mat.m11(), y, mat.m21() * z));
        dest.z = Math::fma_f32(mat.m02(), x, Math::fma_f32(mat.m12(), y, mat.m22() * z));
        dest
    }

    pub fn rotate(&mut self, quat: &Quaternionf) -> &mut Self {
        let this = *self;
        quat.transform_into(&this, self)
    }

    pub fn rotate_into<'a>(&self, quat: &Quaternionf, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        quat.transform_into(self, dest)
    }

    pub fn length_squared(&self) -> f32 {
        Math::fma_f32(
            self.x,
            self.x,
            Math::fma_f32(self.y, self.y, self.z * self.z),
        )
    }

    pub fn length(&self) -> f32 {
        Math::sqrt_f32(Math::fma_f32(
            self.x,
            self.x,
            Math::fma_f32(self.y, self.y, self.z * self.z),
        ))
    }

    pub fn normalize(&mut self) -> &mut Self {
        let this = *self;
        this.normalize_into(self)
    }

    pub fn normalize_into<'a>(&self, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        let scalar = Math::invsqrt_f32(Math::fma_f32(
            self.x,
            self.x,
            Math::fma_f32(self.y, self.y, self.z * self.z),
        ));
        dest.x = self.x * scalar;
        dest.y = self.y * scalar;
        dest.z = self.z * scalar;
        dest
    }

    pub fn normalize_length(&mut self, length: f32) -> &mut Self {
        let this = *self;
        this.normalize_length_into(length, self)
    }

    pub fn normalize_length_into<'a>(
        &self,
        length: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let scalar = Math::invsqrt_f32(Math::fma_f32(
            self.x,
            self.x,
            Math::fma_f32(self.y, self.y, self.z * self.z),
        )) * length;
        dest.x = self.x * scalar;
        dest.y = self.y * scalar;
        dest.z = self.z * scalar;
        dest
    }

    pub fn cross(&mut self, v: &Vector3f) -> &mut Self {
        self.cross_xyz(v.x, v.y, v.z)
    }

    pub fn cross_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let rx = Math::fma_f32(self.y, z, -self.z * y);
        let ry = Math::fma_f32(self.z, x, -self.x * z);
        let rz = Math::fma_f32(self.x, y, -self.y * x);
        self.set_xyz(rx, ry, rz)
    }

    pub fn distance(&self, v: &Vector3f) -> f32 {
        self.distance_xyz(v.x, v.y, v.z)
    }

    pub fn distance_xyz(&self, x: f32, y: f32, z: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        let dz = self.z - z;
        Math::sqrt_f32(Math::fma_f32(dx, dx, Math::fma_f32(dy, dy, dz * dz)))
    }

    pub fn distance_squared(&self, v: &Vector3f) -> f32 {
        self.distance_squared_xyz(v.x, v.y, v.z)
    }

    pub fn distance_squared_xyz(&self, x: f32, y: f32, z: f32) -> f32 {
        let dx = self.x - x;
        let dy = self.y - y;
        let dz = self.z - z;
        Math::fma_f32(dx, dx, Math::fma_f32(dy, dy, dz * dz))
    }

    pub fn dot(&self, v: &Vector3f) -> f32 {
        self.dot_xyz(v.x, v.y, v.z)
    }

    pub fn dot_xyz(&self, x: f32, y: f32, z: f32) -> f32 {
        Math::fma_f32(self.x, x, Math::fma_f32(self.y, y, self.z * z))
    }

    pub fn negate(&mut self) -> &mut Self {
        self.set_xyz(-self.x, -self.y, -self.z)
    }

    pub fn lerp(&mut self, other: &Vector3f, t: f32) -> &mut Self {
        let this = *self;
        this.lerp_into(other, t, self)
    }

    pub fn lerp_into<'a>(
        &self,
        other: &Vector3f,
        t: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.x = Math::fma_f32(other.x - self.x, t, self.x);
        dest.y = Math::fma_f32(other.y - self.y, t, self.y);
        dest.z = Math::fma_f32(other.z - self.z, t, self.z);
        dest
    }
}

#[cfg(test)]
mod tests {
    use super::Vector3f;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix3f::Matrix3f;
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;

    encode_fields!(Vector3f {
        x: f32,
        y: f32,
        z: f32
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Vector3f");
        golden.check("<init>()V", Vector3f::new);
        golden.check("<init>(FFF)V", Vector3f::from_xyz);
        golden.check("<init>(Lorg/joml/Vector3fc;)V", |v: Vector3f| {
            Vector3f::from_vector3f(&v)
        });
        golden.check_method(
            "set(FFF)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, x, y, z| *this.set_xyz(x, y, z),
        );
        golden.check_method(
            "set(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, v: Vector3f| *this.set(&v),
        );
        golden.check_method("zero()Lorg/joml/Vector3f;", |this: &mut Vector3f| {
            *this.zero()
        });
        golden.check_method(
            "add(FFF)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, x, y, z| *this.add_xyz(x, y, z),
        );
        golden.check_method(
            "add(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, v: Vector3f| *this.add(&v),
        );
        golden.check_method(
            "sub(FFF)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, x, y, z| *this.sub_xyz(x, y, z),
        );
        golden.check_method(
            "sub(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, v: Vector3f| *this.sub(&v),
        );
        golden.check_method(
            "mul(F)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, scalar| *this.mul_scalar(scalar),
        );
        golden.check_method(
            "div(F)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, scalar| *this.div_scalar(scalar),
        );
        golden.check_method("get(I)F", |this: &mut Vector3f, component| {
            this.get(component)
        });
        golden.check_method("isFinite()Z", |this: &mut Vector3f| this.is_finite());
        golden.check_method(
            "mul(FFF)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, x, y, z| *this.mul_xyz(x, y, z),
        );
        golden.check_method(
            "mul(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, v: Vector3f| *this.mul(&v),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix3f| *this.mul_matrix3f(&mat),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix3f, mut dest: Vector3f| {
                *this.mul_matrix3f_into(&mat, &mut dest)
            },
        );
        golden.check_method("negate()Lorg/joml/Vector3f;", |this: &mut Vector3f| {
            *this.negate()
        });
        golden.check_method(
            "cross(FFF)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, x, y, z| *this.cross_xyz(x, y, z),
        );
        golden.check_method(
            "cross(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, v: Vector3f| *this.cross(&v),
        );
        golden.check_method(
            "lerp(Lorg/joml/Vector3fc;F)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, other: Vector3f, t| *this.lerp(&other, t),
        );
        golden.check_method(
            "lerp(Lorg/joml/Vector3fc;FLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, other: Vector3f, t, mut dest: Vector3f| {
                *this.lerp_into(&other, t, &mut dest)
            },
        );
        golden.check_method("dot(FFF)F", |this: &mut Vector3f, x, y, z| {
            this.dot_xyz(x, y, z)
        });
        golden.check_method("dot(Lorg/joml/Vector3fc;)F", |this: &mut Vector3f, v| {
            this.dot(&v)
        });
        golden.check_method("length()F", |this: &mut Vector3f| this.length());
        golden.check_method("lengthSquared()F", |this: &mut Vector3f| {
            this.length_squared()
        });
        golden.check_method("distance(FFF)F", |this: &mut Vector3f, x, y, z| {
            this.distance_xyz(x, y, z)
        });
        golden.check_method(
            "distance(Lorg/joml/Vector3fc;)F",
            |this: &mut Vector3f, v| this.distance(&v),
        );
        golden.check_method("distanceSquared(FFF)F", |this: &mut Vector3f, x, y, z| {
            this.distance_squared_xyz(x, y, z)
        });
        golden.check_method(
            "distanceSquared(Lorg/joml/Vector3fc;)F",
            |this: &mut Vector3f, v| this.distance_squared(&v),
        );
        golden.check_method("normalize()Lorg/joml/Vector3f;", |this: &mut Vector3f| {
            *this.normalize()
        });
        golden.check_method(
            "normalize(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mut dest: Vector3f| *this.normalize_into(&mut dest),
        );
        golden.check_method(
            "normalize(F)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, length| *this.normalize_length(length),
        );
        golden.check_method(
            "normalize(FLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, length, mut dest: Vector3f| {
                *this.normalize_length_into(length, &mut dest)
            },
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, quat: Quaternionf| *this.rotate(&quat),
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, quat: Quaternionf, mut dest: Vector3f| {
                *this.rotate_into(&quat, &mut dest)
            },
        );
        golden.check_method(
            "mulProject(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f| *this.mul_project(&mat),
        );
        golden.check_method(
            "mulProject(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f, mut dest: Vector3f| {
                *this.mul_project_into(&mat, &mut dest)
            },
        );
        golden.check_method(
            "mulPosition(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f| *this.mul_position(&mat),
        );
        golden.check_method(
            "mulPosition(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f, mut dest: Vector3f| {
                *this.mul_position_into(&mat, &mut dest)
            },
        );
        golden.check_method(
            "mulDirection(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f| *this.mul_direction(&mat),
        );
        golden.check_method(
            "mulDirection(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Vector3f, mat: Matrix4f, mut dest: Vector3f| {
                *this.mul_direction_into(&mat, &mut dest)
            },
        );
        golden.finish();
    }
}
