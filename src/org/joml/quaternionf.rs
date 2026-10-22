//! org/joml/Quaternionf.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.

use crate::org::joml::math::Math;
use crate::org::joml::vector3f::Vector3f;

#[derive(Clone, Copy, Debug)]
pub struct Quaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternionf {
    fn default() -> Quaternionf {
        Quaternionf::new()
    }
}

impl Quaternionf {
    /// The identity rotation.
    pub const fn new() -> Quaternionf {
        Quaternionf::from_xyzw(0.0, 0.0, 0.0, 1.0)
    }

    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Quaternionf {
        Quaternionf { x, y, z, w }
    }

    pub fn from_quaternionf(source: &Quaternionf) -> Quaternionf {
        let mut quaternion = Quaternionf::from_xyzw(0.0, 0.0, 0.0, 0.0);
        quaternion.set(source);
        quaternion
    }

    pub fn normalize(&mut self) -> &mut Self {
        let this = *self;
        this.normalize_into(self)
    }

    pub fn normalize_into<'a>(&self, dest: &'a mut Quaternionf) -> &'a mut Quaternionf {
        let inv_norm = Math::invsqrt_f32(Math::fma_f32(
            self.x,
            self.x,
            Math::fma_f32(
                self.y,
                self.y,
                Math::fma_f32(self.z, self.z, self.w * self.w),
            ),
        ));
        dest.x = self.x * inv_norm;
        dest.y = self.y * inv_norm;
        dest.z = self.z * inv_norm;
        dest.w = self.w * inv_norm;
        dest
    }

    pub fn set(&mut self, q: &Quaternionf) -> &mut Self {
        self.x = q.x;
        self.y = q.y;
        self.z = q.z;
        self.w = q.w;
        self
    }

    pub fn set_xyzw(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
        self
    }

    pub fn rotation_axis_xyz(
        &mut self,
        angle: f32,
        axis_x: f32,
        axis_y: f32,
        axis_z: f32,
    ) -> &mut Self {
        let hangle = angle / 2.0;
        let sin_angle = Math::sin_f32(hangle);
        let inv_v_length = Math::invsqrt_f32(axis_x * axis_x + axis_y * axis_y + axis_z * axis_z);
        self.set_xyzw(
            axis_x * inv_v_length * sin_angle,
            axis_y * inv_v_length * sin_angle,
            axis_z * inv_v_length * sin_angle,
            Math::cos_from_sin_f32(sin_angle, hangle),
        )
    }

    pub fn rotation_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotation_axis_xyz(angle, axis.x, axis.y, axis.z)
    }

    pub fn rotation_x(&mut self, angle: f32) -> &mut Self {
        let sin = Math::sin_f32(angle * 0.5);
        let cos = Math::cos_from_sin_f32(sin, angle * 0.5);
        self.set_xyzw(sin, 0.0, 0.0, cos)
    }

    pub fn rotation_y(&mut self, angle: f32) -> &mut Self {
        let sin = Math::sin_f32(angle * 0.5);
        let cos = Math::cos_from_sin_f32(sin, angle * 0.5);
        self.set_xyzw(0.0, sin, 0.0, cos)
    }

    pub fn rotation_z(&mut self, angle: f32) -> &mut Self {
        let sin = Math::sin_f32(angle * 0.5);
        let cos = Math::cos_from_sin_f32(sin, angle * 0.5);
        self.set_xyzw(0.0, 0.0, sin, cos)
    }

    pub fn mul(&mut self, q: &Quaternionf) -> &mut Self {
        let this = *self;
        this.mul_into(q, self)
    }

    pub fn mul_into<'a>(&self, q: &Quaternionf, dest: &'a mut Quaternionf) -> &'a mut Quaternionf {
        self.mul_xyzw_into(q.x, q.y, q.z, q.w, dest)
    }

    pub fn mul_xyzw(&mut self, qx: f32, qy: f32, qz: f32, qw: f32) -> &mut Self {
        let this = *self;
        this.mul_xyzw_into(qx, qy, qz, qw, self)
    }

    pub fn mul_xyzw_into<'a>(
        &self,
        qx: f32,
        qy: f32,
        qz: f32,
        qw: f32,
        dest: &'a mut Quaternionf,
    ) -> &'a mut Quaternionf {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        dest.set_xyzw(
            Math::fma_f32(w, qx, Math::fma_f32(x, qw, Math::fma_f32(y, qz, -z * qy))),
            Math::fma_f32(w, qy, Math::fma_f32(-x, qz, Math::fma_f32(y, qw, z * qx))),
            Math::fma_f32(w, qz, Math::fma_f32(x, qy, Math::fma_f32(-y, qx, z * qw))),
            Math::fma_f32(w, qw, Math::fma_f32(-x, qx, Math::fma_f32(-y, qy, -z * qz))),
        )
    }

    pub fn transform<'a>(&self, vec: &'a mut Vector3f) -> &'a mut Vector3f {
        let (x, y, z) = (vec.x, vec.y, vec.z);
        self.transform_xyz_into(x, y, z, vec)
    }

    pub fn transform_into<'a>(&self, vec: &Vector3f, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        self.transform_xyz_into(vec.x, vec.y, vec.z, dest)
    }

    pub fn transform_xyz_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let (xx, yy, zz, ww) = (
            self.x * self.x,
            self.y * self.y,
            self.z * self.z,
            self.w * self.w,
        );
        let (xy, xz, yz, xw) = (
            self.x * self.y,
            self.x * self.z,
            self.y * self.z,
            self.x * self.w,
        );
        let (zw, yw, k) = (self.z * self.w, self.y * self.w, 1.0 / (xx + yy + zz + ww));
        dest.set_xyz(
            Math::fma_f32(
                (xx - yy - zz + ww) * k,
                x,
                Math::fma_f32(2.0 * (xy - zw) * k, y, (2.0 * (xz + yw) * k) * z),
            ),
            Math::fma_f32(
                2.0 * (xy + zw) * k,
                x,
                Math::fma_f32((yy - xx - zz + ww) * k, y, (2.0 * (yz - xw) * k) * z),
            ),
            Math::fma_f32(
                2.0 * (xz - yw) * k,
                x,
                Math::fma_f32(2.0 * (yz + xw) * k, y, ((zz - xx - yy + ww) * k) * z),
            ),
        )
    }

    pub fn conjugate(&mut self) -> &mut Self {
        let this = *self;
        this.conjugate_into(self)
    }

    pub fn conjugate_into<'a>(&self, dest: &'a mut Quaternionf) -> &'a mut Quaternionf {
        dest.x = -self.x;
        dest.y = -self.y;
        dest.z = -self.z;
        dest.w = self.w;
        dest
    }

    pub fn identity(&mut self) -> &mut Self {
        self.set_xyzw(0.0, 0.0, 0.0, 1.0)
    }

    pub fn rotation_xyz(&mut self, angle_x: f32, angle_y: f32, angle_z: f32) -> &mut Self {
        let sx = Math::sin_f32(angle_x * 0.5);
        let cx = Math::cos_from_sin_f32(sx, angle_x * 0.5);
        let sy = Math::sin_f32(angle_y * 0.5);
        let cy = Math::cos_from_sin_f32(sy, angle_y * 0.5);
        let sz = Math::sin_f32(angle_z * 0.5);
        let cz = Math::cos_from_sin_f32(sz, angle_z * 0.5);

        let cycz = cy * cz;
        let sysz = sy * sz;
        let sycz = sy * cz;
        let cysz = cy * sz;
        self.w = cx * cycz - sx * sysz;
        self.x = sx * cycz + cx * sysz;
        self.y = cx * sycz - sx * cysz;
        self.z = cx * cysz + sx * sycz;
        self
    }

    pub fn rotation_zyx(&mut self, angle_z: f32, angle_y: f32, angle_x: f32) -> &mut Self {
        let sx = Math::sin_f32(angle_x * 0.5);
        let cx = Math::cos_from_sin_f32(sx, angle_x * 0.5);
        let sy = Math::sin_f32(angle_y * 0.5);
        let cy = Math::cos_from_sin_f32(sy, angle_y * 0.5);
        let sz = Math::sin_f32(angle_z * 0.5);
        let cz = Math::cos_from_sin_f32(sz, angle_z * 0.5);

        let cycz = cy * cz;
        let sysz = sy * sz;
        let sycz = sy * cz;
        let cysz = cy * sz;
        self.w = cx * cycz + sx * sysz;
        self.x = sx * cycz - cx * sysz;
        self.y = cx * sycz + sx * cysz;
        self.z = cx * cysz - sx * sycz;
        self
    }

    pub fn rotation_yxz(&mut self, angle_y: f32, angle_x: f32, angle_z: f32) -> &mut Self {
        let sx = Math::sin_f32(angle_x * 0.5);
        let cx = Math::cos_from_sin_f32(sx, angle_x * 0.5);
        let sy = Math::sin_f32(angle_y * 0.5);
        let cy = Math::cos_from_sin_f32(sy, angle_y * 0.5);
        let sz = Math::sin_f32(angle_z * 0.5);
        let cz = Math::cos_from_sin_f32(sz, angle_z * 0.5);

        let x = cy * sx;
        let y = sy * cx;
        let z = sy * sx;
        let w = cy * cx;
        self.x = x * cz + y * sz;
        self.y = y * cz - x * sz;
        self.z = w * sz - z * cz;
        self.w = w * cz + z * sz;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Quaternionf;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::vector3f::Vector3f;

    encode_fields!(Quaternionf {
        x: f32,
        y: f32,
        z: f32,
        w: f32
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Quaternionf");
        golden.check("<init>()V", Quaternionf::new);
        golden.check("<init>(FFFF)V", Quaternionf::from_xyzw);
        golden.check("<init>(Lorg/joml/Quaternionfc;)V", |source: Quaternionf| {
            Quaternionf::from_quaternionf(&source)
        });
        golden.check_method(
            "set(FFFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, x, y, z, w| *this.set_xyzw(x, y, z, w),
        );
        golden.check_method(
            "set(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, q: Quaternionf| *this.set(&q),
        );
        golden.check_method(
            "identity()Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf| *this.identity(),
        );
        golden.check_method(
            "rotationX(F)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, angle| *this.rotation_x(angle),
        );
        golden.check_method(
            "rotationY(F)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, angle| *this.rotation_y(angle),
        );
        golden.check_method(
            "rotationZ(F)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, angle| *this.rotation_z(angle),
        );
        golden.check_method(
            "rotationXYZ(FFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, x, y, z| *this.rotation_xyz(x, y, z),
        );
        golden.check_method(
            "rotationYXZ(FFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, y, x, z| *this.rotation_yxz(y, x, z),
        );
        golden.check_method(
            "rotationZYX(FFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, z, y, x| *this.rotation_zyx(z, y, x),
        );
        golden.check_method(
            "rotationAxis(FFFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, angle, x, y, z| *this.rotation_axis_xyz(angle, x, y, z),
        );
        golden.check_method(
            "rotationAxis(FLorg/joml/Vector3fc;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, angle, axis: Vector3f| *this.rotation_axis(angle, &axis),
        );
        golden.check_method(
            "conjugate()Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf| *this.conjugate(),
        );
        golden.check_method(
            "conjugate(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, mut dest: Quaternionf| *this.conjugate_into(&mut dest),
        );
        golden.check_method(
            "normalize()Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf| *this.normalize(),
        );
        golden.check_method(
            "normalize(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, mut dest: Quaternionf| *this.normalize_into(&mut dest),
        );
        golden.check_method(
            "mul(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, q: Quaternionf| *this.mul(&q),
        );
        golden.check_method(
            "mul(Lorg/joml/Quaternionfc;Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, q: Quaternionf, mut dest: Quaternionf| {
                *this.mul_into(&q, &mut dest)
            },
        );
        golden.check_method(
            "mul(FFFF)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, x, y, z, w| *this.mul_xyzw(x, y, z, w),
        );
        golden.check_method(
            "mul(FFFFLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |this: &mut Quaternionf, x, y, z, w, mut dest: Quaternionf| {
                *this.mul_xyzw_into(x, y, z, w, &mut dest)
            },
        );
        golden.check_method(
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Quaternionf, mut vec: Vector3f| *this.transform(&mut vec),
        );
        golden.check_method(
            "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Quaternionf, vec: Vector3f, mut dest: Vector3f| {
                *this.transform_into(&vec, &mut dest)
            },
        );
        golden.check_method(
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Quaternionf, x, y, z, mut dest: Vector3f| {
                *this.transform_xyz_into(x, y, z, &mut dest)
            },
        );
        golden.finish();
    }
}
