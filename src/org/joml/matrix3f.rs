//! org/joml/Matrix3f.java (JOML 1.10.9)
//!
//! Column-major like the shaders: `m20` is column 2, row 0. Only the methods the client calls so
//! far: the golden suite in oracle/golden/ lists them.

use crate::org::joml::math::Math;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

#[derive(Clone, Copy, Debug)]
pub struct Matrix3f {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
}

impl Default for Matrix3f {
    fn default() -> Matrix3f {
        Matrix3f::new()
    }
}

impl Matrix3f {
    /// The identity.
    pub const fn new() -> Matrix3f {
        Matrix3f::from_values(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    }

    pub fn from_matrix3f(mat: &Matrix3f) -> Matrix3f {
        let mut matrix = Matrix3f::from_values(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        matrix.set(mat);
        matrix
    }

    pub fn from_matrix4f(mat: &Matrix4f) -> Matrix3f {
        let mut matrix = Matrix3f::from_values(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        matrix.set_matrix4f(mat);
        matrix
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn from_values(
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> Matrix3f {
        Matrix3f {
            m00,
            m01,
            m02,
            m10,
            m11,
            m12,
            m20,
            m21,
            m22,
        }
    }

    pub fn set(&mut self, m: &Matrix3f) -> &mut Self {
        *self = *m;
        self
    }

    pub fn set_matrix4f(&mut self, mat: &Matrix4f) -> &mut Self {
        self.m00 = mat.m00();
        self.m01 = mat.m01();
        self.m02 = mat.m02();
        self.m10 = mat.m10();
        self.m11 = mat.m11();
        self.m12 = mat.m12();
        self.m20 = mat.m20();
        self.m21 = mat.m21();
        self.m22 = mat.m22();
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_values(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    ) -> &mut Self {
        *self = Matrix3f::from_values(m00, m01, m02, m10, m11, m12, m20, m21, m22);
        self
    }

    pub fn mul(&mut self, right: &Matrix3f) -> &mut Self {
        let this = *self;
        this.mul_into(right, self)
    }

    pub fn mul_into<'a>(&self, right: &Matrix3f, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let nm00 = Math::fma_f32(
            self.m00,
            right.m00,
            Math::fma_f32(self.m10, right.m01, self.m20 * right.m02),
        );
        let nm01 = Math::fma_f32(
            self.m01,
            right.m00,
            Math::fma_f32(self.m11, right.m01, self.m21 * right.m02),
        );
        let nm02 = Math::fma_f32(
            self.m02,
            right.m00,
            Math::fma_f32(self.m12, right.m01, self.m22 * right.m02),
        );
        let nm10 = Math::fma_f32(
            self.m00,
            right.m10,
            Math::fma_f32(self.m10, right.m11, self.m20 * right.m12),
        );
        let nm11 = Math::fma_f32(
            self.m01,
            right.m10,
            Math::fma_f32(self.m11, right.m11, self.m21 * right.m12),
        );
        let nm12 = Math::fma_f32(
            self.m02,
            right.m10,
            Math::fma_f32(self.m12, right.m11, self.m22 * right.m12),
        );
        let nm20 = Math::fma_f32(
            self.m00,
            right.m20,
            Math::fma_f32(self.m10, right.m21, self.m20 * right.m22),
        );
        let nm21 = Math::fma_f32(
            self.m01,
            right.m20,
            Math::fma_f32(self.m11, right.m21, self.m21 * right.m22),
        );
        let nm22 = Math::fma_f32(
            self.m02,
            right.m20,
            Math::fma_f32(self.m12, right.m21, self.m22 * right.m22),
        );
        dest.set_values(nm00, nm01, nm02, nm10, nm11, nm12, nm20, nm21, nm22)
    }

    pub fn invert(&mut self) -> &mut Self {
        let this = *self;
        this.invert_into(self)
    }

    pub fn invert_into<'a>(&self, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let a = Math::fma_f32(self.m00, self.m11, -self.m01 * self.m10);
        let b = Math::fma_f32(self.m02, self.m10, -self.m00 * self.m12);
        let c = Math::fma_f32(self.m01, self.m12, -self.m02 * self.m11);
        let d = Math::fma_f32(a, self.m22, Math::fma_f32(b, self.m21, c * self.m20));
        let s = 1.0 / d;
        let nm00 = Math::fma_f32(self.m11, self.m22, -self.m21 * self.m12) * s;
        let nm01 = Math::fma_f32(self.m21, self.m02, -self.m01 * self.m22) * s;
        let nm02 = c * s;
        let nm10 = Math::fma_f32(self.m20, self.m12, -self.m10 * self.m22) * s;
        let nm11 = Math::fma_f32(self.m00, self.m22, -self.m20 * self.m02) * s;
        let nm12 = b * s;
        let nm20 = Math::fma_f32(self.m10, self.m21, -self.m20 * self.m11) * s;
        let nm21 = Math::fma_f32(self.m20, self.m01, -self.m00 * self.m21) * s;
        let nm22 = a * s;
        dest.set_values(nm00, nm01, nm02, nm10, nm11, nm12, nm20, nm21, nm22)
    }

    pub fn transpose(&mut self) -> &mut Self {
        let this = *self;
        this.transpose_into(self)
    }

    pub fn transpose_into<'a>(&self, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        dest.set_values(
            self.m00, self.m10, self.m20, self.m01, self.m11, self.m21, self.m02, self.m12,
            self.m22,
        )
    }

    /// MemUtil.INSTANCE.identity(this).
    pub fn identity(&mut self) -> &mut Self {
        self.set_values(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let this = *self;
        this.scale_into(x, y, z, self)
    }

    pub fn scale_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix3f,
    ) -> &'a mut Matrix3f {
        dest.m00 = self.m00 * x;
        dest.m01 = self.m01 * x;
        dest.m02 = self.m02 * x;
        dest.m10 = self.m10 * y;
        dest.m11 = self.m11 * y;
        dest.m12 = self.m12 * y;
        dest.m20 = self.m20 * z;
        dest.m21 = self.m21 * z;
        dest.m22 = self.m22 * z;
        dest
    }

    pub fn scale_scalar(&mut self, xyz: f32) -> &mut Self {
        self.scale(xyz, xyz, xyz)
    }

    pub fn transform<'a>(&self, v: &'a mut Vector3f) -> &'a mut Vector3f {
        v.mul_matrix3f(self)
    }

    pub fn transform_into<'a>(&self, v: &Vector3f, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        v.mul_matrix3f_into(self, dest)
    }

    pub fn transform_xyz_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.set_xyz(
            Math::fma_f32(self.m00, x, Math::fma_f32(self.m10, y, self.m20 * z)),
            Math::fma_f32(self.m01, x, Math::fma_f32(self.m11, y, self.m21 * z)),
            Math::fma_f32(self.m02, x, Math::fma_f32(self.m12, y, self.m22 * z)),
        )
    }

    pub fn rotate_x(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_x_into(ang, self)
    }

    pub fn rotate_x_into<'a>(&self, ang: f32, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        let rm11 = cos;
        let rm21 = -sin;
        let rm12 = sin;
        let rm22 = cos;

        let nm10 = self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m12 * rm11 + self.m22 * rm12;
        dest.m20 = self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m12 * rm21 + self.m22 * rm22;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m00 = self.m00;
        dest.m01 = self.m01;
        dest.m02 = self.m02;
        dest
    }

    pub fn rotate_y(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_y_into(ang, self)
    }

    pub fn rotate_y_into<'a>(&self, ang: f32, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        let rm00 = cos;
        let rm20 = sin;
        let rm02 = -sin;
        let rm22 = cos;

        let nm00 = self.m00 * rm00 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m22 * rm02;
        dest.m20 = self.m00 * rm20 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m22 * rm22;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m10 = self.m10;
        dest.m11 = self.m11;
        dest.m12 = self.m12;
        dest
    }

    pub fn rotate_z(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_z_into(ang, self)
    }

    pub fn rotate_z_into<'a>(&self, ang: f32, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        let rm00 = cos;
        let rm10 = -sin;
        let rm01 = sin;
        let rm11 = cos;

        let nm00 = self.m00 * rm00 + self.m10 * rm01;
        let nm01 = self.m01 * rm00 + self.m11 * rm01;
        let nm02 = self.m02 * rm00 + self.m12 * rm01;
        dest.m10 = self.m00 * rm10 + self.m10 * rm11;
        dest.m11 = self.m01 * rm10 + self.m11 * rm11;
        dest.m12 = self.m02 * rm10 + self.m12 * rm11;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m20 = self.m20;
        dest.m21 = self.m21;
        dest.m22 = self.m22;
        dest
    }

    pub fn rotate_axis_xyz(&mut self, ang: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let this = *self;
        this.rotate_axis_xyz_into(ang, x, y, z, self)
    }

    pub fn rotate_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix3f,
    ) -> &'a mut Matrix3f {
        let s = Math::sin_f32(ang);
        let c = Math::cos_from_sin_f32(s, ang);
        let one_minus_c = 1.0 - c;

        let (xx, xy, xz) = (x * x, x * y, x * z);
        let (yy, yz) = (y * y, y * z);
        let zz = z * z;
        let rm00 = xx * one_minus_c + c;
        let rm01 = xy * one_minus_c + z * s;
        let rm02 = xz * one_minus_c - y * s;
        let rm10 = xy * one_minus_c - z * s;
        let rm11 = yy * one_minus_c + c;
        let rm12 = yz * one_minus_c + x * s;
        let rm20 = xz * one_minus_c + y * s;
        let rm21 = yz * one_minus_c - x * s;
        let rm22 = zz * one_minus_c + c;

        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest
    }

    pub fn rotate(&mut self, quat: &Quaternionf) -> &mut Self {
        let this = *self;
        this.rotate_into(quat, self)
    }

    pub fn rotate_into<'a>(&self, quat: &Quaternionf, dest: &'a mut Matrix3f) -> &'a mut Matrix3f {
        let (w2, x2) = (quat.w * quat.w, quat.x * quat.x);
        let (y2, z2) = (quat.y * quat.y, quat.z * quat.z);
        let zw = quat.z * quat.w;
        let dzw = zw + zw;
        let xy = quat.x * quat.y;
        let dxy = xy + xy;
        let xz = quat.x * quat.z;
        let dxz = xz + xz;
        let yw = quat.y * quat.w;
        let dyw = yw + yw;
        let yz = quat.y * quat.z;
        let dyz = yz + yz;
        let xw = quat.x * quat.w;
        let dxw = xw + xw;
        let rm00 = w2 + x2 - z2 - y2;
        let rm01 = dxy + dzw;
        let rm02 = dxz - dyw;
        let rm10 = dxy - dzw;
        let rm11 = y2 - z2 + w2 - x2;
        let rm12 = dyz + dxw;
        let rm20 = dyw + dxz;
        let rm21 = dyz - dxw;
        let rm22 = z2 - y2 - x2 + w2;
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest
    }

    pub fn rotate_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate_axis_xyz(angle, axis.x, axis.y, axis.z)
    }

    pub fn rotate_axis_into<'a>(
        &self,
        angle: f32,
        axis: &Vector3f,
        dest: &'a mut Matrix3f,
    ) -> &'a mut Matrix3f {
        self.rotate_axis_xyz_into(angle, axis.x, axis.y, axis.z, dest)
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix3f;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    encode_fields!(Matrix3f {
        m00: f32,
        m01: f32,
        m02: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m20: f32,
        m21: f32,
        m22: f32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix3f");
        golden.check("<init>()V", Matrix3f::new);
        golden.check("<init>(FFFFFFFFF)V", Matrix3f::from_values);
        golden.check("<init>(Lorg/joml/Matrix3fc;)V", |mat: Matrix3f| {
            Matrix3f::from_matrix3f(&mat)
        });
        golden.check("<init>(Lorg/joml/Matrix4fc;)V", |mat: Matrix4f| {
            Matrix3f::from_matrix4f(&mat)
        });
        golden.check_method(
            "set(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, m: Matrix3f| *this.set(&m),
        );
        golden.check_method(
            "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, mat: Matrix4f| *this.set_matrix4f(&mat),
        );
        golden.check_method(
            "set(FFFFFFFFF)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, m00, m01, m02, m10, m11, m12, m20, m21, m22| {
                *this.set_values(m00, m01, m02, m10, m11, m12, m20, m21, m22)
            },
        );
        golden.check_method("identity()Lorg/joml/Matrix3f;", |this: &mut Matrix3f| {
            *this.identity()
        });
        golden.check_method(
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, right: Matrix3f| *this.mul(&right),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, right: Matrix3f, mut dest: Matrix3f| {
                *this.mul_into(&right, &mut dest)
            },
        );
        golden.check_method("invert()Lorg/joml/Matrix3f;", |this: &mut Matrix3f| {
            *this.invert()
        });
        golden.check_method(
            "invert(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, mut dest: Matrix3f| *this.invert_into(&mut dest),
        );
        golden.check_method("transpose()Lorg/joml/Matrix3f;", |this: &mut Matrix3f| {
            *this.transpose()
        });
        golden.check_method(
            "transpose(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, mut dest: Matrix3f| *this.transpose_into(&mut dest),
        );
        golden.check_method(
            "scale(FFF)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, x, y, z| *this.scale(x, y, z),
        );
        golden.check_method(
            "scale(FFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, x, y, z, mut dest: Matrix3f| *this.scale_into(x, y, z, &mut dest),
        );
        golden.check_method("scale(F)Lorg/joml/Matrix3f;", |this: &mut Matrix3f, xyz| {
            *this.scale_scalar(xyz)
        });
        golden.check_method(
            "rotateX(F)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang| *this.rotate_x(ang),
        );
        golden.check_method(
            "rotateX(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang, mut dest: Matrix3f| *this.rotate_x_into(ang, &mut dest),
        );
        golden.check_method(
            "rotateY(F)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang| *this.rotate_y(ang),
        );
        golden.check_method(
            "rotateY(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang, mut dest: Matrix3f| *this.rotate_y_into(ang, &mut dest),
        );
        golden.check_method(
            "rotateZ(F)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang| *this.rotate_z(ang),
        );
        golden.check_method(
            "rotateZ(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang, mut dest: Matrix3f| *this.rotate_z_into(ang, &mut dest),
        );
        golden.check_method(
            "rotate(FFFF)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang, x, y, z| *this.rotate_axis_xyz(ang, x, y, z),
        );
        golden.check_method(
            "rotate(FFFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, ang, x, y, z, mut dest: Matrix3f| {
                *this.rotate_axis_xyz_into(ang, x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, angle, axis: Vector3f| *this.rotate_axis(angle, &axis),
        );
        golden.check_method(
            "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, angle, axis: Vector3f, mut dest: Matrix3f| {
                *this.rotate_axis_into(angle, &axis, &mut dest)
            },
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, quat: Quaternionf| *this.rotate(&quat),
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            |this: &mut Matrix3f, quat: Quaternionf, mut dest: Matrix3f| {
                *this.rotate_into(&quat, &mut dest)
            },
        );
        golden.check_method(
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix3f, mut v: Vector3f| *this.transform(&mut v),
        );
        golden.check_method(
            "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix3f, v: Vector3f, mut dest: Vector3f| {
                *this.transform_into(&v, &mut dest)
            },
        );
        golden.check_method(
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix3f, x, y, z, mut dest: Vector3f| {
                *this.transform_xyz_into(x, y, z, &mut dest)
            },
        );
        golden.finish();
    }
}
