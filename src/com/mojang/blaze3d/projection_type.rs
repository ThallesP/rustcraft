//! com/mojang/blaze3d/ProjectionType.java
//!
//! Only the constants so far. `vertexSorting` and `applyLayeringTransform` come with
//! `VertexSorting` and the renderers that layer their draws.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionType {
    PERSPECTIVE,
    ORTHOGRAPHIC,
}
