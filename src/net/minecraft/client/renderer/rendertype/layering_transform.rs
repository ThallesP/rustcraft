//! net/minecraft/client/renderer/rendertype/LayeringTransform.java
//!
//! Only `NO_LAYERING` so far: the view offsets come with the render types that use them, with
//! `ProjectionType.applyLayeringTransform`.

use crate::org::joml::matrix4f::Matrix4f;

/// How a render type moves its draws towards or away from the camera.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayeringTransform {
    NO_LAYERING,
}

impl LayeringTransform {
    /// `getModifier()`, applied to the model view matrix: none for `NO_LAYERING`.
    pub fn apply(&self, _model_view_matrix: &mut Matrix4f) {
        match self {
            LayeringTransform::NO_LAYERING => {}
        }
    }
}
