//! net/minecraft/client/renderer/state/gui/BlitRenderState.java

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::state::gui::colored_rectangle_render_state::get_bounds;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::org::joml::matrix3x2f::Matrix3x2f;

#[derive(Clone)]
pub struct BlitRenderState {
    pub pipeline: Arc<RenderPipeline>,
    pub texture_setup: TextureSetup,
    pub pose: Matrix3x2f,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub u0: f32,
    pub u1: f32,
    pub v0: f32,
    pub v1: f32,
    pub color: i32,
    pub scissor_area: Option<ScreenRectangle>,
    pub bounds: Option<ScreenRectangle>,
}

impl BlitRenderState {
    /// The constructor working out the bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipeline: Arc<RenderPipeline>,
        texture_setup: TextureSetup,
        pose: Matrix3x2f,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
        color: i32,
        scissor_area: Option<ScreenRectangle>,
    ) -> BlitRenderState {
        let bounds = get_bounds(x0, y0, x1, y1, &pose, scissor_area);
        BlitRenderState {
            pipeline,
            texture_setup,
            pose,
            x0,
            y0,
            x1,
            y1,
            u0,
            u1,
            v0,
            v1,
            color,
            scissor_area,
            bounds,
        }
    }
}

impl ScreenArea for BlitRenderState {
    fn bounds(&self) -> Option<ScreenRectangle> {
        self.bounds
    }
}

impl GuiElementRenderState for BlitRenderState {
    fn build_vertices(&self, vertex_consumer: &mut dyn VertexConsumer) {
        let (x0, y0, x1, y1) = (
            self.x0 as f32,
            self.y0 as f32,
            self.x1 as f32,
            self.y1 as f32,
        );
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x0, y0)
            .set_uv(self.u0, self.v0)
            .set_color(self.color);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x0, y1)
            .set_uv(self.u0, self.v1)
            .set_color(self.color);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x1, y1)
            .set_uv(self.u1, self.v1)
            .set_color(self.color);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x1, y0)
            .set_uv(self.u1, self.v0)
            .set_color(self.color);
    }

    fn pipeline(&self) -> &Arc<RenderPipeline> {
        &self.pipeline
    }

    fn texture_setup(&self) -> &TextureSetup {
        &self.texture_setup
    }

    fn scissor_area(&self) -> Option<ScreenRectangle> {
        self.scissor_area
    }
}
