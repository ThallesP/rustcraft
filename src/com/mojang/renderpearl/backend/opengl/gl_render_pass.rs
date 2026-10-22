//! com/mojang/renderpearl/backend/opengl/GlRenderPass.java
//!
//! Only opening and closing so far: the pass's state is the encoder's.

use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;

pub struct GlRenderPass {
    pub(super) color_attachment_count: usize,
}

impl GlRenderPass {
    pub fn color_attachment_count(&self) -> usize {
        self.color_attachment_count
    }
}

impl RenderPass for GlRenderPass {}
