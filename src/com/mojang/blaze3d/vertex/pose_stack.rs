//! com/mojang/blaze3d/vertex/PoseStack.java
//!
//! Not ported yet: `translate(Vec3)`, which needs `Vec3`.

use crate::com::mojang::math::axis::Axis;
use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::com::mojang::math::transformation::Transformation;
use crate::java::lang::math::Math;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4fc::PROPERTY_ORTHONORMAL;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// Popped poses stay in `poses` and are overwritten by the next push, like Java's list.
#[derive(Clone, Debug)]
pub struct PoseStack {
    poses: Vec<Pose>,
    last_index: usize,
}

impl Default for PoseStack {
    fn default() -> PoseStack {
        PoseStack::new()
    }
}

impl PoseStack {
    pub fn new() -> PoseStack {
        let mut poses = Vec::with_capacity(16);
        poses.push(Pose::new());
        PoseStack {
            poses,
            last_index: 0,
        }
    }

    pub fn translate_f64(&mut self, xo: f64, yo: f64, zo: f64) {
        self.translate_f32(xo as f32, yo as f32, zo as f32);
    }

    pub fn translate_f32(&mut self, xo: f32, yo: f32, zo: f32) {
        self.last_mut().translate(xo, yo, zo);
    }

    pub fn scale(&mut self, x_scale: f32, y_scale: f32, z_scale: f32) {
        self.last_mut().scale(x_scale, y_scale, z_scale);
    }

    pub fn rotate(&mut self, by: &Quaternionf) {
        self.last_mut().rotate(by);
    }

    pub fn rotate_axis(&mut self, axis: Axis, angle: f32) {
        self.last_mut().rotate_axis(axis, angle);
    }

    pub fn rotate_degrees(&mut self, axis: Axis, angle: f32) {
        self.last_mut().rotate_degrees(axis, angle);
    }

    pub fn rotate_around(
        &mut self,
        rotation: &Quaternionf,
        pivot_x: f32,
        pivot_y: f32,
        pivot_z: f32,
    ) {
        self.last_mut()
            .rotate_around(rotation, pivot_x, pivot_y, pivot_z);
    }

    pub fn push_pose(&mut self) {
        let last_pose = self.last().clone();
        self.last_index += 1;
        if self.last_index >= self.poses.len() {
            self.poses.push(last_pose);
        } else {
            self.poses[self.last_index].set(&last_pose);
        }
    }

    pub fn pop_pose(&mut self) {
        if self.last_index == 0 {
            panic!("NoSuchElementException: popPose with one pose left");
        }
        self.last_index -= 1;
    }

    pub fn last(&self) -> &Pose {
        &self.poses[self.last_index]
    }

    /// `last()` for callers that change the pose: Java hands out the same mutable `Pose`.
    pub fn last_mut(&mut self) -> &mut Pose {
        &mut self.poses[self.last_index]
    }

    pub fn is_empty(&self) -> bool {
        self.last_index == 0
    }

    pub fn set_identity(&mut self) {
        self.last_mut().set_identity();
    }

    /// Like Java, adds to `matrix`'s properties when it doesn't say it's a pure translation.
    pub fn mul_pose(&mut self, matrix: &mut Matrix4f) {
        self.last_mut().mul_pose(matrix);
    }

    /// Adds to the properties of the transformation's own matrix, like `mul_pose`.
    pub fn mul_pose_transformation(&mut self, matrix: &mut Transformation) {
        self.last_mut().mul_pose_transformation(matrix);
    }
}

/// PoseStack.Pose: a model-view matrix and the matrix its normals go through.
#[derive(Clone, Debug)]
pub struct Pose {
    pose: Matrix4f,
    normal: Matrix3f,
    trusted_normals: bool,
}

impl Default for Pose {
    fn default() -> Pose {
        Pose::new()
    }
}

impl Pose {
    pub fn new() -> Pose {
        Pose {
            pose: Matrix4f::new(),
            normal: Matrix3f::new(),
            trusted_normals: true,
        }
    }

    fn compute_normal_matrix(&mut self) {
        self.normal.set_matrix4f(&self.pose).invert().transpose();
        self.trusted_normals = false;
    }

    pub fn set(&mut self, pose: &Pose) {
        self.pose.set(&pose.pose);
        self.normal.set(&pose.normal);
        self.trusted_normals = pose.trusted_normals;
    }

    pub fn pose(&self) -> &Matrix4f {
        &self.pose
    }

    pub fn normal(&self) -> &Matrix3f {
        &self.normal
    }

    pub fn transform_normal<'a>(
        &self,
        normal: &Vector3f,
        destination: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        self.transform_normal_xyz(normal.x, normal.y, normal.z, destination)
    }

    /// Normalizes the result unless the normal matrix only ever rotated, or scaled uniformly.
    pub fn transform_normal_xyz<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        destination: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        let result = self.normal.transform_xyz_into(x, y, z, destination);
        if self.trusted_normals {
            result
        } else {
            result.normalize()
        }
    }

    pub fn translate(&mut self, xo: f32, yo: f32, zo: f32) -> &mut Matrix4f {
        self.pose.translate(xo, yo, zo)
    }

    /// A uniform scale leaves normals alone, apart from flipping them when it's negative.
    pub fn scale(&mut self, x_scale: f32, y_scale: f32, z_scale: f32) {
        self.pose.scale(x_scale, y_scale, z_scale);
        if x_scale.abs() == y_scale.abs() && y_scale.abs() == z_scale.abs() {
            if x_scale < 0.0 || y_scale < 0.0 || z_scale < 0.0 {
                self.normal.scale(
                    Math::signum_f32(x_scale),
                    Math::signum_f32(y_scale),
                    Math::signum_f32(z_scale),
                );
            }
        } else {
            self.normal
                .scale(1.0 / x_scale, 1.0 / y_scale, 1.0 / z_scale);
            self.trusted_normals = false;
        }
    }

    pub fn rotate(&mut self, by: &Quaternionf) {
        self.pose.rotate(by);
        self.normal.rotate(by);
    }

    pub fn rotate_axis(&mut self, axis: Axis, angle: f32) {
        axis.rotate_matrix4f(&mut self.pose, angle);
        axis.rotate_matrix3f(&mut self.normal, angle);
    }

    pub fn rotate_degrees(&mut self, axis: Axis, angle: f32) {
        axis.rotate_degrees_matrix4f(&mut self.pose, angle);
        axis.rotate_degrees_matrix3f(&mut self.normal, angle);
    }

    pub fn rotate_around(
        &mut self,
        rotation: &Quaternionf,
        pivot_x: f32,
        pivot_y: f32,
        pivot_z: f32,
    ) {
        self.pose.rotate_around(rotation, pivot_x, pivot_y, pivot_z);
        self.normal.rotate(rotation);
    }

    pub fn set_identity(&mut self) {
        self.pose.identity();
        self.normal.identity();
        self.trusted_normals = true;
    }

    /// Multiplies first, then asks `matrix` about its properties, which can add to them.
    pub fn mul_pose(&mut self, matrix: &mut Matrix4f) {
        self.pose.mul(matrix);
        if !MatrixUtil::is_pure_translation(matrix) {
            if MatrixUtil::check_property_raw(matrix, PROPERTY_ORTHONORMAL) {
                self.normal.mul(&Matrix3f::from_matrix4f(matrix));
            } else {
                self.compute_normal_matrix();
            }
        }
    }

    /// Java hands `mulPose` the transformation's own matrix, so the properties it adds stay on the
    /// transformation, for everything else that holds it.
    pub fn mul_pose_transformation(&mut self, transformation: &mut Transformation) {
        self.mul_pose(transformation.get_matrix_mut());
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::PoseStack;
    use crate::com::mojang::math::axis::Axis;
    use crate::com::mojang::math::transformation::Transformation;
    use crate::golden::{Decode, Encode, Golden, Thrown, catch};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.blaze3d.vertex.PoseStack");
        golden.check("PoseStack", |program: Value| run(&program));
        golden.finish();
    }

    // Mirrors PoseStackSuite in oracle/golden/: each op's output, then the top pose's state. A
    // panicking op ends the program like a thrown exception.
    fn run(program: &Value) -> Vec<Box<dyn Encode>> {
        let mut stack = PoseStack::new();
        let mut steps: Vec<Box<dyn Encode>> = Vec::new();
        for op in program.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let Ok(output) = catch(|| apply(&mut stack, op)) else {
                steps.push(Box::new(Thrown));
                break;
            };
            let last = stack.last();
            steps.push(Box::new((
                output,
                *last.pose(),
                *last.normal(),
                last.trusted_normals,
            )));
        }
        steps
    }

    fn apply(stack: &mut PoseStack, op: &[Value]) -> Box<dyn Encode> {
        let float = |index: usize| f32::decode(&op[index]);
        let double = |index: usize| f64::decode(&op[index]);
        match op[0].as_str().expect("ops start with their name") {
            "pushPose" => stack.push_pose(),
            "popPose" => stack.pop_pose(),
            "translate" => stack.translate_f32(float(1), float(2), float(3)),
            "translateDouble" => stack.translate_f64(double(1), double(2), double(3)),
            "scale" => stack.scale(float(1), float(2), float(3)),
            "rotate" => stack.rotate(&Quaternionf::decode(&op[1])),
            "rotateAxis" => stack.rotate_axis(Axis::decode(&op[1]), float(2)),
            "rotateDegrees" => stack.rotate_degrees(Axis::decode(&op[1]), float(2)),
            "rotateAround" => {
                stack.rotate_around(&Quaternionf::decode(&op[1]), float(2), float(3), float(4))
            }
            "setIdentity" => stack.set_identity(),
            "mulPose" => {
                let mut matrix = Matrix4f::decode(&op[1]);
                stack.mul_pose(&mut matrix);
                return Box::new(matrix);
            }
            "mulPoseTransformation" => {
                let mut transformation = Transformation::decode(&op[1]);
                stack.mul_pose_transformation(&mut transformation);
                return Box::new(transformation);
            }
            "transformNormal" => {
                let mut destination = Vector3f::new();
                let normal = *stack.last().transform_normal_xyz(
                    float(1),
                    float(2),
                    float(3),
                    &mut destination,
                );
                return Box::new(normal);
            }
            "transformNormalVector" => {
                let mut destination = Vector3f::new();
                let normal = *stack
                    .last()
                    .transform_normal(&Vector3f::decode(&op[1]), &mut destination);
                return Box::new(normal);
            }
            "isEmpty" => return Box::new(stack.is_empty()),
            name => panic!("unknown op {name}"),
        }
        Box::new(())
    }
}
