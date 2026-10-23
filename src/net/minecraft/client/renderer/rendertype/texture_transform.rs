//! net/minecraft/client/renderer/rendertype/TextureTransform.java
//!
//! Only `DEFAULT_TEXTURING`, the identity, so far. The glint's and the offset texturings come
//! with the render types that use them.

use crate::org::joml::matrix4f::Matrix4f;

/// A render type's texture matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureTransform {
    DEFAULT_TEXTURING,
}

impl TextureTransform {
    pub fn create_matrix(&self) -> Matrix4f {
        match self {
            TextureTransform::DEFAULT_TEXTURING => Matrix4f::new(),
        }
    }
}
