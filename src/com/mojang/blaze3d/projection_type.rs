//! com/mojang/blaze3d/ProjectionType.java

use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::org::joml::matrix4f::Matrix4f;

/// A projection's kind: how its translucent quads sort, and how a render type's layering moves
/// its draws towards the camera.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionType {
    PERSPECTIVE,
    ORTHOGRAPHIC,
}

impl ProjectionType {
    pub fn vertex_sorting(&self) -> VertexSorting {
        match self {
            ProjectionType::PERSPECTIVE => VertexSorting::distance_to_origin(),
            ProjectionType::ORTHOGRAPHIC => VertexSorting::orthographic_z(),
        }
    }

    /// The model view `matrix` shrunk towards the camera by `bias` (a perspective), or moved
    /// along Z (orthographic).
    pub fn apply_layering_transform(&self, matrix: &mut Matrix4f, bias: f32) {
        match self {
            ProjectionType::PERSPECTIVE => {
                let factor = 1.0 - bias / 4096.0;
                matrix.scale(factor, factor, factor);
            }
            ProjectionType::ORTHOGRAPHIC => {
                matrix.translate(0.0, 0.0, bias / 512.0);
            }
        }
    }
}
