//! org/joml/Vector3d.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the occlusion graph's walk toward the camera.

use crate::org::joml::math::Math;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3d {
    pub const fn from_xyz(x: f64, y: f64, z: f64) -> Vector3d {
        Vector3d { x, y, z }
    }

    pub fn sub(&mut self, v: &Vector3d) -> &mut Self {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
        self
    }

    pub fn add(&mut self, v: &Vector3d) -> &mut Self {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
        self
    }

    pub fn mul_scalar(&mut self, scalar: f64) -> &mut Self {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
        self
    }

    pub fn normalize(&mut self) -> &mut Self {
        let inv_length = Math::invsqrt_f64(Math::fma_f64(
            self.x,
            self.x,
            Math::fma_f64(self.y, self.y, self.z * self.z),
        ));
        self.x *= inv_length;
        self.y *= inv_length;
        self.z *= inv_length;
        self
    }

    pub fn distance_squared_xyz(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        let dz = self.z - z;
        Math::fma_f64(dx, dx, Math::fma_f64(dy, dy, dz * dz))
    }
}
