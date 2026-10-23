//! net/minecraft/client/resources/model/cuboid/CuboidRotation.java
//!
//! `RotationValue`, an interface its two records implement, is an enum of them.

use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::net::minecraft::core::direction::Axis;
use crate::org::joml::math::Math;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;

/// An element's rotation about `origin` (in blocks), and the matrix it makes.
#[derive(Clone, Debug)]
pub struct CuboidRotation {
    pub origin: Vector3f,
    pub value: RotationValue,
    pub rescale: bool,
    pub transform: Matrix4f,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RotationValue {
    /// `EulerXYZRotation`: degrees about each axis, applied x, then y, then z.
    EulerXYZ { x: f32, y: f32, z: f32 },
    /// `SingleAxisRotation`: degrees about one axis.
    SingleAxis { axis: Axis, angle: f32 },
}

// Java's `(float)(Math.PI / 180.0)`.
const DEGREES_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;

impl CuboidRotation {
    pub fn new(origin: Vector3f, value: RotationValue, rescale: bool) -> CuboidRotation {
        CuboidRotation {
            origin,
            value,
            rescale,
            transform: CuboidRotation::compute_transform(value, rescale),
        }
    }

    fn compute_transform(value: RotationValue, rescale: bool) -> Matrix4f {
        let mut result = value.transformation();
        if rescale && !MatrixUtil::is_identity(&mut result) {
            let scale = CuboidRotation::compute_rescale(&result);
            result.scale_vector3f(&scale);
        }
        result
    }

    fn compute_rescale(rotation: &Matrix4f) -> Vector3f {
        let scale_x = CuboidRotation::scale_factor_for_axis(rotation, Axis::X);
        let scale_y = CuboidRotation::scale_factor_for_axis(rotation, Axis::Y);
        let scale_z = CuboidRotation::scale_factor_for_axis(rotation, Axis::Z);
        Vector3f::from_xyz(scale_x, scale_y, scale_z)
    }

    fn scale_factor_for_axis(rotation: &Matrix4f, axis: Axis) -> f32 {
        let mut axis_unit = *axis.get_positive().get_unit_vec3f();
        let transformed_axis_unit = rotation.transform_direction(&mut axis_unit);
        let abs_x = Math::abs_f32(transformed_axis_unit.x);
        let abs_y = Math::abs_f32(transformed_axis_unit.y);
        let abs_z = Math::abs_f32(transformed_axis_unit.z);
        let max_component = Math::max_f32(Math::max_f32(abs_x, abs_y), abs_z);
        1.0 / max_component
    }
}

impl RotationValue {
    pub fn transformation(&self) -> Matrix4f {
        match *self {
            RotationValue::EulerXYZ { x, y, z } => *Matrix4f::new().rotation_zyx(
                z * DEGREES_TO_RADIANS,
                y * DEGREES_TO_RADIANS,
                x * DEGREES_TO_RADIANS,
            ),
            RotationValue::SingleAxis { axis, angle } => {
                let mut result = Matrix4f::new();
                if angle == 0.0 {
                    return result;
                }

                let rotate_around = axis.get_positive().get_unit_vec3f();
                result.rotation_axis(angle * DEGREES_TO_RADIANS, rotate_around);
                result
            }
        }
    }
}
