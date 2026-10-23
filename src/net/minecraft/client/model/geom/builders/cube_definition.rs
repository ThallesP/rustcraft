//! net/minecraft/client/model/geom/builders/CubeDefinition.java
//!
//! The comment (a cube's id, which only the debug tools read) isn't kept. Java's `UVPair`s are
//! pairs of floats here.

use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
use crate::net::minecraft::client::model::geom::model_part::Cube;
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::vector3f::Vector3f;

/// A cube of a part before it's baked: its box, growth, mirroring, texture offset and scale, and
/// which of its faces show (an `EnumSet`, in `Direction`'s order).
#[derive(Clone, Debug)]
pub struct CubeDefinition {
    origin: Vector3f,
    dimensions: Vector3f,
    grow: CubeDeformation,
    mirror: bool,
    tex_coord: (f32, f32),
    tex_scale: (f32, f32),
    visible_faces: Vec<Direction>,
}

impl CubeDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x_tex_offs: f32,
        y_tex_offs: f32,
        min_x: f32,
        min_y: f32,
        min_z: f32,
        width: f32,
        height: f32,
        depth: f32,
        grow: CubeDeformation,
        mirror: bool,
        x_tex_scale: f32,
        y_tex_scale: f32,
        visible_faces: &[Direction],
    ) -> CubeDefinition {
        CubeDefinition {
            tex_coord: (x_tex_offs, y_tex_offs),
            origin: Vector3f::from_xyz(min_x, min_y, min_z),
            dimensions: Vector3f::from_xyz(width, height, depth),
            grow,
            mirror,
            tex_scale: (x_tex_scale, y_tex_scale),
            visible_faces: visible_faces.to_vec(),
        }
    }

    pub fn bake(&self, tex_scale_x: i32, tex_scale_y: i32) -> Cube {
        Cube::new(
            self.tex_coord.0 as i32,
            self.tex_coord.1 as i32,
            self.origin.x,
            self.origin.y,
            self.origin.z,
            self.dimensions.x,
            self.dimensions.y,
            self.dimensions.z,
            self.grow.grow_x,
            self.grow.grow_y,
            self.grow.grow_z,
            self.mirror,
            tex_scale_x as f32 * self.tex_scale.0,
            tex_scale_y as f32 * self.tex_scale.1,
            &self.visible_faces,
        )
    }
}
