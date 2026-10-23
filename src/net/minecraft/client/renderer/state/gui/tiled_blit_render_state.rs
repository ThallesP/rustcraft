//! net/minecraft/client/renderer/state/gui/TiledBlitRenderState.java

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::state::gui::colored_rectangle_render_state::get_bounds;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix3x2f::Matrix3x2f;

#[derive(Clone)]
pub struct TiledBlitRenderState {
    pub pipeline: Arc<RenderPipeline>,
    pub texture_setup: TextureSetup,
    pub pose: Matrix3x2f,
    pub tile_width: i32,
    pub tile_height: i32,
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

impl TiledBlitRenderState {
    /// The constructor working out the bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipeline: Arc<RenderPipeline>,
        texture_setup: TextureSetup,
        pose: Matrix3x2f,
        tile_width: i32,
        tile_height: i32,
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
    ) -> TiledBlitRenderState {
        let bounds = get_bounds(x0, y0, x1, y1, &pose, scissor_area);
        TiledBlitRenderState {
            pipeline,
            texture_setup,
            pose,
            tile_width,
            tile_height,
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

impl ScreenArea for TiledBlitRenderState {
    fn bounds(&self) -> Option<ScreenRectangle> {
        self.bounds
    }
}

impl GuiElementRenderState for TiledBlitRenderState {
    fn build_vertices(&self, vertex_consumer: &mut dyn VertexConsumer) {
        let width = self.x1 - self.x0;
        let height = self.y1 - self.y0;
        let mut tile_x = 0;
        while tile_x < width {
            let remaining_width = width - tile_x;
            let (tile_width, u1) = if self.tile_width <= remaining_width {
                (self.tile_width, self.u1)
            } else {
                (
                    remaining_width,
                    Mth::lerp_f32(
                        remaining_width as f32 / self.tile_width as f32,
                        self.u0,
                        self.u1,
                    ),
                )
            };
            let mut tile_y = 0;
            while tile_y < height {
                let remaining_height = height - tile_y;
                let (tile_height, v1) = if self.tile_height <= remaining_height {
                    (self.tile_height, self.v1)
                } else {
                    (
                        remaining_height,
                        Mth::lerp_f32(
                            remaining_height as f32 / self.tile_height as f32,
                            self.v0,
                            self.v1,
                        ),
                    )
                };
                let x0 = (self.x0 + tile_x) as f32;
                let x1 = (self.x0 + tile_x + tile_width) as f32;
                let y0 = (self.y0 + tile_y) as f32;
                let y1 = (self.y0 + tile_y + tile_height) as f32;
                vertex_consumer
                    .add_vertex_with_2d_pose(&self.pose, x0, y0)
                    .set_uv(self.u0, self.v0)
                    .set_color(self.color);
                vertex_consumer
                    .add_vertex_with_2d_pose(&self.pose, x0, y1)
                    .set_uv(self.u0, v1)
                    .set_color(self.color);
                vertex_consumer
                    .add_vertex_with_2d_pose(&self.pose, x1, y1)
                    .set_uv(u1, v1)
                    .set_color(self.color);
                vertex_consumer
                    .add_vertex_with_2d_pose(&self.pose, x1, y0)
                    .set_uv(u1, self.v0)
                    .set_color(self.color);
                tile_y += self.tile_height;
            }
            tile_x += self.tile_width;
        }
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
