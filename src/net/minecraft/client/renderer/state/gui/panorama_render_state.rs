//! net/minecraft/client/renderer/state/gui/PanoramaRenderState.java

/// The panorama's turn about y this frame, in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanoramaRenderState {
    pub spin: f32,
}
