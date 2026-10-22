//! net/minecraft/client/renderer/LevelRenderer.java
//!
//! Only the order-independent transparency constants the pipelines are built from so far.

pub struct LevelRenderer;

impl LevelRenderer {
    /// `Math.powExact(2, 3)`.
    pub const OIT_COEFFICIENT_COUNT: i32 = 8;
    pub const OIT_TRANSMITTANCE_TARGET_COUNT: i32 = LevelRenderer::OIT_COEFFICIENT_COUNT / 4;
}
