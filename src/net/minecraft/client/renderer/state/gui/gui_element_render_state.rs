//! net/minecraft/client/renderer/state/gui/GuiElementRenderState.java

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;

pub trait GuiElementRenderState: ScreenArea {
    fn build_vertices(&self, vertex_consumer: &mut dyn VertexConsumer);

    fn pipeline(&self) -> &Arc<RenderPipeline>;

    fn texture_setup(&self) -> &TextureSetup;

    fn scissor_area(&self) -> Option<ScreenRectangle>;
}
