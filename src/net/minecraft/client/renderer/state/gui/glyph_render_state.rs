//! net/minecraft/client/renderer/state/gui/GlyphRenderState.java
//!
//! A text's glyph or effect as a GUI element: drawn flat with full light under the text's pose,
//! from its font page with the UI lightmap (`TextureSetup.singleTextureWithLightmap` with
//! `getClampToEdge(NEAREST)`), which the GUI's text pipelines don't sample. It has no bounds: it
//! goes in the layer its text went in.

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::net::minecraft::client::gui::font::text_renderable::TextRenderable;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::org::joml::matrix3x2f::Matrix3x2f;
use crate::org::joml::matrix4f::Matrix4f;

pub struct GlyphRenderState {
    pub pose: Matrix3x2f,
    pub renderable: TextRenderable,
    pub scissor_area: Option<ScreenRectangle>,
    texture_setup: TextureSetup,
}

impl GlyphRenderState {
    /// The texture setup is `textureSetup()`'s, which reads the font page's view, the sampler
    /// cache and the game renderer's lightmap: the caller's.
    pub fn new(
        pose: Matrix3x2f,
        renderable: TextRenderable,
        scissor_area: Option<ScreenRectangle>,
        texture_setup: TextureSetup,
    ) -> GlyphRenderState {
        GlyphRenderState {
            pose,
            renderable,
            scissor_area,
            texture_setup,
        }
    }
}

impl ScreenArea for GlyphRenderState {
    fn bounds(&self) -> Option<ScreenRectangle> {
        None
    }
}

impl GuiElementRenderState for GlyphRenderState {
    fn build_vertices(&self, vertex_consumer: &mut dyn VertexConsumer) {
        let mut pose = Matrix4f::new();
        pose.mul_matrix3x2f(&self.pose);
        self.renderable
            .render(&pose, vertex_consumer, 15728880, true);
    }

    fn pipeline(&self) -> &Arc<RenderPipeline> {
        self.renderable.gui_pipeline()
    }

    fn texture_setup(&self) -> &TextureSetup {
        &self.texture_setup
    }

    fn scissor_area(&self) -> Option<ScreenRectangle> {
        self.scissor_area
    }
}
