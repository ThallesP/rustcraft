//! org/joml/FrustumIntersection.java (JOML 1.10.9)
//!
//! The six planes of a projection-view matrix's frustum, and tests of points and axis-aligned
//! boxes against them. The game culls chunk sections and entities with it (`Frustum`, which
//! needs `AABB` and comes with milestone 2). Only the methods the client calls so far: the golden
//! suite in oracle/golden/ lists them.
//!
//! Each plane is `x * X + y * Y + z * Z + W >= 0` for the points on its inner side, normalized
//! when `set` is asked to allow sphere tests (`set(m)` always does). `planes` keeps a second copy
//! of them, which only JOML's `distanceToPlane` reads.

use crate::org::joml::math::Math;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector4f::Vector4f;

#[derive(Clone, Copy, Debug)]
pub struct FrustumIntersection {
    nx_x: f32,
    nx_y: f32,
    nx_z: f32,
    nx_w: f32,
    px_x: f32,
    px_y: f32,
    px_z: f32,
    px_w: f32,
    ny_x: f32,
    ny_y: f32,
    ny_z: f32,
    ny_w: f32,
    py_x: f32,
    py_y: f32,
    py_z: f32,
    py_w: f32,
    nz_x: f32,
    nz_y: f32,
    nz_z: f32,
    nz_w: f32,
    pz_x: f32,
    pz_y: f32,
    pz_z: f32,
    pz_w: f32,
    planes: [Vector4f; 6],
}

impl Default for FrustumIntersection {
    fn default() -> FrustumIntersection {
        FrustumIntersection::new()
    }
}

impl FrustumIntersection {
    pub const PLANE_NX: i32 = 0;
    pub const PLANE_PX: i32 = 1;
    pub const PLANE_NY: i32 = 2;
    pub const PLANE_PY: i32 = 3;
    pub const PLANE_NZ: i32 = 4;
    pub const PLANE_PZ: i32 = 5;

    /// The box crosses a plane.
    pub const INTERSECT: i32 = -1;
    /// The box is inside every plane.
    pub const INSIDE: i32 = -2;
    pub const OUTSIDE: i32 = -3;

    pub const PLANE_MASK_NX: i32 = 1 << FrustumIntersection::PLANE_NX;
    pub const PLANE_MASK_PX: i32 = 1 << FrustumIntersection::PLANE_PX;
    pub const PLANE_MASK_NY: i32 = 1 << FrustumIntersection::PLANE_NY;
    pub const PLANE_MASK_PY: i32 = 1 << FrustumIntersection::PLANE_PY;
    pub const PLANE_MASK_NZ: i32 = 1 << FrustumIntersection::PLANE_NZ;
    pub const PLANE_MASK_PZ: i32 = 1 << FrustumIntersection::PLANE_PZ;

    /// Every plane is zero, and `planes` holds `(0, 0, 0, 1)`s, until `set` is called.
    pub const fn new() -> FrustumIntersection {
        FrustumIntersection {
            nx_x: 0.0,
            nx_y: 0.0,
            nx_z: 0.0,
            nx_w: 0.0,
            px_x: 0.0,
            px_y: 0.0,
            px_z: 0.0,
            px_w: 0.0,
            ny_x: 0.0,
            ny_y: 0.0,
            ny_z: 0.0,
            ny_w: 0.0,
            py_x: 0.0,
            py_y: 0.0,
            py_z: 0.0,
            py_w: 0.0,
            nz_x: 0.0,
            nz_y: 0.0,
            nz_z: 0.0,
            nz_w: 0.0,
            pz_x: 0.0,
            pz_y: 0.0,
            pz_z: 0.0,
            pz_w: 0.0,
            planes: [Vector4f::new(); 6],
        }
    }

    pub fn set(&mut self, m: &Matrix4f) -> &mut Self {
        self.set_allow_test_spheres(m, true)
    }

    pub fn set_allow_test_spheres(&mut self, m: &Matrix4f, allow_test_spheres: bool) -> &mut Self {
        let mut invl;
        self.nx_x = m.m03() + m.m00();
        self.nx_y = m.m13() + m.m10();
        self.nx_z = m.m23() + m.m20();
        self.nx_w = m.m33() + m.m30();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.nx_x * self.nx_x + self.nx_y * self.nx_y + self.nx_z * self.nx_z,
            );
            self.nx_x *= invl;
            self.nx_y *= invl;
            self.nx_z *= invl;
            self.nx_w *= invl;
        }
        self.planes[0].set_xyzw(self.nx_x, self.nx_y, self.nx_z, self.nx_w);
        self.px_x = m.m03() - m.m00();
        self.px_y = m.m13() - m.m10();
        self.px_z = m.m23() - m.m20();
        self.px_w = m.m33() - m.m30();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.px_x * self.px_x + self.px_y * self.px_y + self.px_z * self.px_z,
            );
            self.px_x *= invl;
            self.px_y *= invl;
            self.px_z *= invl;
            self.px_w *= invl;
        }
        self.planes[1].set_xyzw(self.px_x, self.px_y, self.px_z, self.px_w);
        self.ny_x = m.m03() + m.m01();
        self.ny_y = m.m13() + m.m11();
        self.ny_z = m.m23() + m.m21();
        self.ny_w = m.m33() + m.m31();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.ny_x * self.ny_x + self.ny_y * self.ny_y + self.ny_z * self.ny_z,
            );
            self.ny_x *= invl;
            self.ny_y *= invl;
            self.ny_z *= invl;
            self.ny_w *= invl;
        }
        self.planes[2].set_xyzw(self.ny_x, self.ny_y, self.ny_z, self.ny_w);
        self.py_x = m.m03() - m.m01();
        self.py_y = m.m13() - m.m11();
        self.py_z = m.m23() - m.m21();
        self.py_w = m.m33() - m.m31();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.py_x * self.py_x + self.py_y * self.py_y + self.py_z * self.py_z,
            );
            self.py_x *= invl;
            self.py_y *= invl;
            self.py_z *= invl;
            self.py_w *= invl;
        }
        self.planes[3].set_xyzw(self.py_x, self.py_y, self.py_z, self.py_w);
        self.nz_x = m.m03() + m.m02();
        self.nz_y = m.m13() + m.m12();
        self.nz_z = m.m23() + m.m22();
        self.nz_w = m.m33() + m.m32();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.nz_x * self.nz_x + self.nz_y * self.nz_y + self.nz_z * self.nz_z,
            );
            self.nz_x *= invl;
            self.nz_y *= invl;
            self.nz_z *= invl;
            self.nz_w *= invl;
        }
        self.planes[4].set_xyzw(self.nz_x, self.nz_y, self.nz_z, self.nz_w);
        self.pz_x = m.m03() - m.m02();
        self.pz_y = m.m13() - m.m12();
        self.pz_z = m.m23() - m.m22();
        self.pz_w = m.m33() - m.m32();
        if allow_test_spheres {
            invl = Math::invsqrt_f32(
                self.pz_x * self.pz_x + self.pz_y * self.pz_y + self.pz_z * self.pz_z,
            );
            self.pz_x *= invl;
            self.pz_y *= invl;
            self.pz_z *= invl;
            self.pz_w *= invl;
        }
        self.planes[5].set_xyzw(self.pz_x, self.pz_y, self.pz_z, self.pz_w);
        self
    }

    pub fn test_point(&self, x: f32, y: f32, z: f32) -> bool {
        self.nx_x * x + self.nx_y * y + self.nx_z * z + self.nx_w >= 0.0
            && self.px_x * x + self.px_y * y + self.px_z * z + self.px_w >= 0.0
            && self.ny_x * x + self.ny_y * y + self.ny_z * z + self.ny_w >= 0.0
            && self.py_x * x + self.py_y * y + self.py_z * z + self.py_w >= 0.0
            && self.nz_x * x + self.nz_y * y + self.nz_z * z + self.nz_w >= 0.0
            && self.pz_x * x + self.pz_y * y + self.pz_z * z + self.pz_w >= 0.0
    }

    /// `INSIDE`, `INTERSECT`, or the first plane (`PLANE_NX`, …) the box is fully outside of.
    /// Each plane tests the box's corner furthest along its normal, then the nearest one.
    pub fn intersect_aab(
        &self,
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
    ) -> i32 {
        // Java's `nxX < 0 ? minX : maxX` and `nxX < 0 ? maxX : minX`.
        let furthest = |normal: f32, min: f32, max: f32| if normal < 0.0 { min } else { max };
        let nearest = |normal: f32, min: f32, max: f32| if normal < 0.0 { max } else { min };
        let mut plane = FrustumIntersection::PLANE_NX;
        let mut inside = true;
        if self.nx_x * furthest(self.nx_x, min_x, max_x)
            + self.nx_y * furthest(self.nx_y, min_y, max_y)
            + self.nx_z * furthest(self.nx_z, min_z, max_z)
            >= -self.nx_w
        {
            plane = FrustumIntersection::PLANE_PX;
            inside &= self.nx_x * nearest(self.nx_x, min_x, max_x)
                + self.nx_y * nearest(self.nx_y, min_y, max_y)
                + self.nx_z * nearest(self.nx_z, min_z, max_z)
                >= -self.nx_w;
            if self.px_x * furthest(self.px_x, min_x, max_x)
                + self.px_y * furthest(self.px_y, min_y, max_y)
                + self.px_z * furthest(self.px_z, min_z, max_z)
                >= -self.px_w
            {
                plane = FrustumIntersection::PLANE_NY;
                inside &= self.px_x * nearest(self.px_x, min_x, max_x)
                    + self.px_y * nearest(self.px_y, min_y, max_y)
                    + self.px_z * nearest(self.px_z, min_z, max_z)
                    >= -self.px_w;
                if self.ny_x * furthest(self.ny_x, min_x, max_x)
                    + self.ny_y * furthest(self.ny_y, min_y, max_y)
                    + self.ny_z * furthest(self.ny_z, min_z, max_z)
                    >= -self.ny_w
                {
                    plane = FrustumIntersection::PLANE_PY;
                    inside &= self.ny_x * nearest(self.ny_x, min_x, max_x)
                        + self.ny_y * nearest(self.ny_y, min_y, max_y)
                        + self.ny_z * nearest(self.ny_z, min_z, max_z)
                        >= -self.ny_w;
                    if self.py_x * furthest(self.py_x, min_x, max_x)
                        + self.py_y * furthest(self.py_y, min_y, max_y)
                        + self.py_z * furthest(self.py_z, min_z, max_z)
                        >= -self.py_w
                    {
                        plane = FrustumIntersection::PLANE_NZ;
                        inside &= self.py_x * nearest(self.py_x, min_x, max_x)
                            + self.py_y * nearest(self.py_y, min_y, max_y)
                            + self.py_z * nearest(self.py_z, min_z, max_z)
                            >= -self.py_w;
                        if self.nz_x * furthest(self.nz_x, min_x, max_x)
                            + self.nz_y * furthest(self.nz_y, min_y, max_y)
                            + self.nz_z * furthest(self.nz_z, min_z, max_z)
                            >= -self.nz_w
                        {
                            plane = FrustumIntersection::PLANE_PZ;
                            inside &= self.nz_x * nearest(self.nz_x, min_x, max_x)
                                + self.nz_y * nearest(self.nz_y, min_y, max_y)
                                + self.nz_z * nearest(self.nz_z, min_z, max_z)
                                >= -self.nz_w;
                            if self.pz_x * furthest(self.pz_x, min_x, max_x)
                                + self.pz_y * furthest(self.pz_y, min_y, max_y)
                                + self.pz_z * furthest(self.pz_z, min_z, max_z)
                                >= -self.pz_w
                            {
                                inside &= self.pz_x * nearest(self.pz_x, min_x, max_x)
                                    + self.pz_y * nearest(self.pz_y, min_y, max_y)
                                    + self.pz_z * nearest(self.pz_z, min_z, max_z)
                                    >= -self.pz_w;
                                return if inside {
                                    FrustumIntersection::INSIDE
                                } else {
                                    FrustumIntersection::INTERSECT
                                };
                            }
                        }
                    }
                }
            }
        }
        plane
    }
}

#[cfg(test)]
mod tests {
    use super::FrustumIntersection;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::vector4f::Vector4f;

    encode_fields!(FrustumIntersection {
        nx_x: f32,
        nx_y: f32,
        nx_z: f32,
        nx_w: f32,
        px_x: f32,
        px_y: f32,
        px_z: f32,
        px_w: f32,
        ny_x: f32,
        ny_y: f32,
        ny_z: f32,
        ny_w: f32,
        py_x: f32,
        py_y: f32,
        py_z: f32,
        py_w: f32,
        nz_x: f32,
        nz_y: f32,
        nz_z: f32,
        nz_w: f32,
        pz_x: f32,
        pz_y: f32,
        pz_z: f32,
        pz_w: f32,
        planes: [Vector4f; 6],
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.FrustumIntersection");
        golden.field("PLANE_NX", FrustumIntersection::PLANE_NX);
        golden.field("PLANE_PX", FrustumIntersection::PLANE_PX);
        golden.field("PLANE_NY", FrustumIntersection::PLANE_NY);
        golden.field("PLANE_PY", FrustumIntersection::PLANE_PY);
        golden.field("PLANE_NZ", FrustumIntersection::PLANE_NZ);
        golden.field("PLANE_PZ", FrustumIntersection::PLANE_PZ);
        golden.field("INTERSECT", FrustumIntersection::INTERSECT);
        golden.field("INSIDE", FrustumIntersection::INSIDE);
        golden.field("OUTSIDE", FrustumIntersection::OUTSIDE);
        golden.field("PLANE_MASK_NX", FrustumIntersection::PLANE_MASK_NX);
        golden.field("PLANE_MASK_PX", FrustumIntersection::PLANE_MASK_PX);
        golden.field("PLANE_MASK_NY", FrustumIntersection::PLANE_MASK_NY);
        golden.field("PLANE_MASK_PY", FrustumIntersection::PLANE_MASK_PY);
        golden.field("PLANE_MASK_NZ", FrustumIntersection::PLANE_MASK_NZ);
        golden.field("PLANE_MASK_PZ", FrustumIntersection::PLANE_MASK_PZ);
        golden.check("<init>()V", FrustumIntersection::new);
        golden.check_method(
            "set(Lorg/joml/Matrix4fc;)Lorg/joml/FrustumIntersection;",
            |this: &mut FrustumIntersection, m: Matrix4f| *this.set(&m),
        );
        golden.check_method(
            "set(Lorg/joml/Matrix4fc;Z)Lorg/joml/FrustumIntersection;",
            |this: &mut FrustumIntersection, m: Matrix4f, allow_test_spheres| {
                *this.set_allow_test_spheres(&m, allow_test_spheres)
            },
        );
        golden.check_method(
            "intersectAab(FFFFFF)I",
            |this: &mut FrustumIntersection, min_x, min_y, min_z, max_x, max_y, max_z| {
                this.intersect_aab(min_x, min_y, min_z, max_x, max_y, max_z)
            },
        );
        golden.check_method(
            "testPoint(FFF)Z",
            |this: &mut FrustumIntersection, x, y, z| this.test_point(x, y, z),
        );
        golden.finish();
    }
}
