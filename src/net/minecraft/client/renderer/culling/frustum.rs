//! net/minecraft/client/renderer/culling/Frustum.java
//!
//! Not ported yet: `getFrustumBoundingBox` (`AABB.Builder`), for the debug renderer.

use crate::net::minecraft::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::org::joml::frustum_intersection::FrustumIntersection;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector4f::Vector4f;

/// The camera's view volume, for culling boxes in the level: JOML's intersection over the
/// projection and view matrices, with boxes taken relative to the camera.
#[derive(Clone)]
pub struct Frustum {
    intersection: FrustumIntersection,
    matrix: Matrix4f,
    view_vector: Vector4f,
    cam_x: f64,
    cam_y: f64,
    cam_z: f64,
}

impl Frustum {
    pub const OFFSET_STEP: i32 = 4;

    pub fn new(model_view: &Matrix4f, projection: &Matrix4f) -> Frustum {
        let mut frustum = Frustum {
            intersection: FrustumIntersection::new(),
            matrix: Matrix4f::new(),
            view_vector: Vector4f::new(),
            cam_x: 0.0,
            cam_y: 0.0,
            cam_z: 0.0,
        };
        frustum.calculate_frustum(model_view, projection);
        frustum
    }

    /// Steps the camera back along the view until the cube of `cube_size` around it is inside.
    pub fn offset_to_fully_include_camera_cube(&mut self, cube_size: i32) -> &mut Self {
        let cube_size = f64::from(cube_size);
        let cam_x1 = (self.cam_x / cube_size).floor() * cube_size;
        let cam_y1 = (self.cam_y / cube_size).floor() * cube_size;
        let cam_z1 = (self.cam_z / cube_size).floor() * cube_size;
        let cam_x2 = (self.cam_x / cube_size).ceil() * cube_size;
        let cam_y2 = (self.cam_y / cube_size).ceil() * cube_size;
        let cam_z2 = (self.cam_z / cube_size).ceil() * cube_size;
        while self.intersection.intersect_aab(
            (cam_x1 - self.cam_x) as f32,
            (cam_y1 - self.cam_y) as f32,
            (cam_z1 - self.cam_z) as f32,
            (cam_x2 - self.cam_x) as f32,
            (cam_y2 - self.cam_y) as f32,
            (cam_z2 - self.cam_z) as f32,
        ) != FrustumIntersection::INSIDE
        {
            self.cam_x -= f64::from(self.view_vector.x * 4.0);
            self.cam_y -= f64::from(self.view_vector.y * 4.0);
            self.cam_z -= f64::from(self.view_vector.z * 4.0);
        }

        self
    }

    pub fn prepare(&mut self, cam_x: f64, cam_y: f64, cam_z: f64) {
        self.cam_x = cam_x;
        self.cam_y = cam_y;
        self.cam_z = cam_z;
    }

    fn calculate_frustum(&mut self, model_view: &Matrix4f, projection: &Matrix4f) {
        projection.mul_into(model_view, &mut self.matrix);
        self.intersection.set(&self.matrix);
        let mut view_vector = Vector4f::from_xyzw(0.0, 0.0, 1.0, 0.0);
        self.matrix.transform_transpose(&mut view_vector);
        self.view_vector = view_vector;
    }

    pub fn is_visible(&self, bb: &AABB) -> bool {
        let intersection_result =
            self.cube_in_frustum_xyz(bb.min_x, bb.min_y, bb.min_z, bb.max_x, bb.max_y, bb.max_z);
        intersection_result == FrustumIntersection::INSIDE
            || intersection_result == FrustumIntersection::INTERSECT
    }

    pub fn cube_in_frustum(&self, bb: &BoundingBox) -> i32 {
        self.cube_in_frustum_xyz(
            f64::from(bb.min_x()),
            f64::from(bb.min_y()),
            f64::from(bb.min_z()),
            f64::from(bb.max_x() + 1),
            f64::from(bb.max_y() + 1),
            f64::from(bb.max_z() + 1),
        )
    }

    fn cube_in_frustum_xyz(
        &self,
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> i32 {
        let x1 = (min_x - self.cam_x) as f32;
        let y1 = (min_y - self.cam_y) as f32;
        let z1 = (min_z - self.cam_z) as f32;
        let x2 = (max_x - self.cam_x) as f32;
        let y2 = (max_y - self.cam_y) as f32;
        let z2 = (max_z - self.cam_z) as f32;
        self.intersection.intersect_aab(x1, y1, z1, x2, y2, z2)
    }

    pub fn get_cam_x(&self) -> f64 {
        self.cam_x
    }

    pub fn get_cam_y(&self) -> f64 {
        self.cam_y
    }

    pub fn get_cam_z(&self) -> f64 {
        self.cam_z
    }
}
