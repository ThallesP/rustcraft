//! net/minecraft/client/renderer/rendertype/LayeringTransform.java

use crate::com::mojang::blaze3d::projection_type::ProjectionType;
use crate::org::joml::matrix4f::Matrix4f;

/// How a render type moves its draws towards or away from the camera, through the projection's
/// layering (`ProjectionType.applyLayeringTransform`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayeringTransform {
    NO_LAYERING,
    VIEW_OFFSET_Z_LAYERING,
    VIEW_OFFSET_Z_LAYERING_FORWARD,
}

impl LayeringTransform {
    /// `getModifier()`, applied to the model view matrix, for the projection the frame draws
    /// with (`RenderSystem.getProjectionType()`).
    pub fn apply(&self, model_view_matrix: &mut Matrix4f, projection_type: ProjectionType) {
        match self {
            LayeringTransform::NO_LAYERING => {}
            LayeringTransform::VIEW_OFFSET_Z_LAYERING => {
                projection_type.apply_layering_transform(model_view_matrix, 1.0)
            }
            LayeringTransform::VIEW_OFFSET_Z_LAYERING_FORWARD => {
                projection_type.apply_layering_transform(model_view_matrix, -1.0)
            }
        }
    }
}
