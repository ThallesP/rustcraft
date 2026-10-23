//! org/joml/GeometryUtils.java (JOML 1.10.9)
//!
//! Only `normal`, which `FaceBakery` finds a baked quad's facing with.

use crate::org::joml::vector3f::Vector3f;

pub struct GeometryUtils;

impl GeometryUtils {
    /// The unit normal of the triangle `v0`, `v1`, `v2`, wound counter-clockwise.
    pub fn normal(v0: &Vector3f, v1: &Vector3f, v2: &Vector3f, dest: &mut Vector3f) {
        GeometryUtils::normal_xyz(v0.x, v0.y, v0.z, v1.x, v1.y, v1.z, v2.x, v2.y, v2.z, dest);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn normal_xyz(
        v0x: f32,
        v0y: f32,
        v0z: f32,
        v1x: f32,
        v1y: f32,
        v1z: f32,
        v2x: f32,
        v2y: f32,
        v2z: f32,
        dest: &mut Vector3f,
    ) {
        dest.x = ((v1y - v0y) * (v2z - v0z)) - ((v1z - v0z) * (v2y - v0y));
        dest.y = ((v1z - v0z) * (v2x - v0x)) - ((v1x - v0x) * (v2z - v0z));
        dest.z = ((v1x - v0x) * (v2y - v0y)) - ((v1y - v0y) * (v2x - v0x));
        dest.normalize();
    }
}
