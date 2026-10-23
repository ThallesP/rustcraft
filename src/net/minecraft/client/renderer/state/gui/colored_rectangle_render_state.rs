//! net/minecraft/client/renderer/state/gui/ColoredRectangleRenderState.java

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::org::joml::matrix3x2f::Matrix3x2f;

#[derive(Clone)]
pub struct ColoredRectangleRenderState {
    pub pipeline: Arc<RenderPipeline>,
    pub texture_setup: TextureSetup,
    pub pose: Matrix3x2f,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub col1: i32,
    pub col2: i32,
    pub scissor_area: Option<ScreenRectangle>,
    pub bounds: Option<ScreenRectangle>,
}

impl ColoredRectangleRenderState {
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
        col1: i32,
        col2: i32,
        scissor_area: Option<ScreenRectangle>,
    ) -> ColoredRectangleRenderState {
        let bounds = get_bounds(x0, y0, x1, y1, &pose, scissor_area);
        ColoredRectangleRenderState {
            pipeline,
            texture_setup,
            pose,
            x0,
            y0,
            x1,
            y1,
            col1,
            col2,
            scissor_area,
            bounds,
        }
    }
}

/// `getBounds`, shared by the element states: the rectangle's bounds under the pose, inside the
/// scissor area.
pub(crate) fn get_bounds(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    pose: &Matrix3x2f,
    scissor_area: Option<ScreenRectangle>,
) -> Option<ScreenRectangle> {
    let bounds = ScreenRectangle::new(x0, y0, x1.wrapping_sub(x0), y1.wrapping_sub(y0))
        .transform_max_bounds(pose);
    match scissor_area {
        Some(scissor_area) => scissor_area.intersection(&bounds),
        None => Some(bounds),
    }
}

impl ScreenArea for ColoredRectangleRenderState {
    fn bounds(&self) -> Option<ScreenRectangle> {
        self.bounds
    }
}

impl GuiElementRenderState for ColoredRectangleRenderState {
    fn build_vertices(&self, vertex_consumer: &mut dyn VertexConsumer) {
        let (x0, y0, x1, y1) = (
            self.x0 as f32,
            self.y0 as f32,
            self.x1 as f32,
            self.y1 as f32,
        );
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x0, y0)
            .set_color(self.col1);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x0, y1)
            .set_color(self.col2);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x1, y1)
            .set_color(self.col2);
        vertex_consumer
            .add_vertex_with_2d_pose(&self.pose, x1, y0)
            .set_color(self.col1);
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
