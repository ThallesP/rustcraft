//! net/minecraft/client/renderer/Projection.java
//!
//! `getMatrix` asks `RenderSystem.getDevice().getDeviceInfo().isZZeroToOne()`, whether clip
//! space depth runs from 0 to 1 (OpenGL's is -1 to 1 unless `ARB_clip_control` changes it).
//! `RenderSystem` isn't ported, so the answer is a parameter.

use crate::com::mojang::blaze3d::projection_type::ProjectionType;
use crate::org::joml::matrix4f::Matrix4f;

// `(float) (Math.PI / 180.0)`, which turns the FOV into radians.
const DEGREES_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;

#[derive(Clone, Debug)]
pub struct Projection {
    projection_type: ProjectionType,
    z_near: f32,
    z_far: f32,
    perspective_fov: f32,
    width: f32,
    height: f32,
    ortho_invert_y: bool,
    is_matrix_dirty: bool,
    matrix: Matrix4f,
    matrix_version: i64,
}

impl Default for Projection {
    fn default() -> Projection {
        Projection::new()
    }
}

impl Projection {
    pub fn new() -> Projection {
        Projection {
            projection_type: ProjectionType::PERSPECTIVE,
            z_near: 0.0,
            z_far: 0.0,
            perspective_fov: 0.0,
            width: 0.0,
            height: 0.0,
            ortho_invert_y: false,
            is_matrix_dirty: false,
            matrix: Matrix4f::new(),
            matrix_version: -1,
        }
    }

    /// Only dirties the matrix when something changed. A NaN never equals itself, so it always does.
    pub fn setup_perspective(
        &mut self,
        z_near: f32,
        z_far: f32,
        fov: f32,
        width: f32,
        height: f32,
    ) {
        if self.projection_type != ProjectionType::PERSPECTIVE
            || self.z_near != z_near
            || self.z_far != z_far
            || self.perspective_fov != fov
            || self.width != width
            || self.height != height
        {
            self.is_matrix_dirty = true;
            self.projection_type = ProjectionType::PERSPECTIVE;
            self.z_near = z_near;
            self.z_far = z_far;
            self.perspective_fov = fov;
            self.width = width;
            self.height = height;
        }
    }

    pub fn setup_ortho(
        &mut self,
        z_near: f32,
        z_far: f32,
        width: f32,
        height: f32,
        invert_y: bool,
    ) {
        if self.projection_type != ProjectionType::ORTHOGRAPHIC
            || self.z_near != z_near
            || self.z_far != z_far
            || self.width != width
            || self.height != height
            || self.ortho_invert_y != invert_y
        {
            self.is_matrix_dirty = true;
            self.projection_type = ProjectionType::ORTHOGRAPHIC;
            self.z_near = z_near;
            self.z_far = z_far;
            self.perspective_fov = 0.0;
            self.width = width;
            self.height = height;
            self.ortho_invert_y = invert_y;
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.is_matrix_dirty = true;
        self.width = width;
        self.height = height;
    }

    /// Recomputes the matrix if it's dirty, with near and far swapped: the game renders with
    /// reversed depth, where the far plane gets the smallest depth and the near plane the largest.
    pub fn get_matrix<'a>(
        &mut self,
        dest: &'a mut Matrix4f,
        z_zero_to_one: bool,
    ) -> &'a mut Matrix4f {
        if !self.is_matrix_dirty {
            return dest.set(&self.matrix);
        }

        self.is_matrix_dirty = false;
        self.matrix_version += 1;
        let near = self.z_far;
        let far = self.z_near;
        if self.projection_type == ProjectionType::PERSPECTIVE {
            dest.set(self.matrix.set_perspective(
                self.perspective_fov * DEGREES_TO_RADIANS,
                self.width / self.height,
                near,
                far,
                z_zero_to_one,
            ))
        } else {
            dest.set(self.matrix.set_ortho(
                0.0,
                self.width,
                if self.ortho_invert_y {
                    self.height
                } else {
                    0.0
                },
                if self.ortho_invert_y {
                    0.0
                } else {
                    self.height
                },
                near,
                far,
                z_zero_to_one,
            ))
        }
    }

    pub fn get_matrix_version(&self) -> i64 {
        if self.is_matrix_dirty {
            self.matrix_version + 1
        } else {
            self.matrix_version
        }
    }

    pub fn z_near(&self) -> f32 {
        self.z_near
    }

    pub fn z_far(&self) -> f32 {
        self.z_far
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn fov(&self) -> f32 {
        self.perspective_fov
    }

    pub fn invert_y(&self) -> bool {
        self.ortho_invert_y
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Projection;
    use crate::golden::{Decode, Encode, Golden};
    use crate::org::joml::matrix4f::Matrix4f;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.Projection");
        golden.check("Projection", |z_zero_to_one: bool, program: Value| {
            run(z_zero_to_one, &program)
        });
        golden.finish();
    }

    // Mirrors ProjectionSuite in oracle/golden/: each op's output, then the getters.
    fn run(z_zero_to_one: bool, program: &Value) -> Vec<Box<dyn Encode>> {
        let mut projection = Projection::new();
        let mut steps: Vec<Box<dyn Encode>> = Vec::new();
        for op in program.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let output = apply(&mut projection, op, z_zero_to_one);
            steps.push(Box::new((
                output,
                projection.z_near(),
                projection.z_far(),
                projection.fov(),
                projection.width(),
                projection.height(),
                projection.invert_y(),
                projection.get_matrix_version(),
            )));
        }
        steps
    }

    fn apply(projection: &mut Projection, op: &[Value], z_zero_to_one: bool) -> Box<dyn Encode> {
        let float = |index: usize| f32::decode(&op[index]);
        match op[0].as_str().expect("ops start with their name") {
            "setupPerspective" => {
                projection.setup_perspective(float(1), float(2), float(3), float(4), float(5))
            }
            "setupOrtho" => {
                projection.setup_ortho(float(1), float(2), float(3), float(4), bool::decode(&op[5]))
            }
            "setSize" => projection.set_size(float(1), float(2)),
            "getMatrix" => {
                let mut dest = Matrix4f::decode(&op[1]);
                return Box::new(*projection.get_matrix(&mut dest, z_zero_to_one));
            }
            name => panic!("unknown op {name}"),
        }
        Box::new(())
    }
}
