//! net/minecraft/client/gui/GuiGraphicsExtractor.java
//!
//! What the GUI's code draws with: fills, blits and sprites from the GUI atlas (stretched, tiled or
//! nine-sliced as their `gui` section says) under a pose and a scissor stack, each made into an
//! element state in the frame's `GuiRenderState`. Java reads the window's GUI size, the atlases and
//! the texture manager from its `Minecraft`, a back-reference; here they're its fields, borrowed
//! from what owns them for the extract.
//!
//! Not ported yet: text (with the font), items and their decorations, maps, the picture in picture
//! states (entities, skins, books, banners, the profiler chart), tooltips, the hovered text's
//! effects, the cursor and the preedit overlay.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::state::gui::blit_render_state::BlitRenderState;
use crate::net::minecraft::client::renderer::state::gui::colored_rectangle_render_state::ColoredRectangleRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_render_state::GuiRenderState;
use crate::net::minecraft::client::renderer::state::gui::tiled_blit_render_state::TiledBlitRenderState;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;
use crate::net::minecraft::client::resources::metadata::gui::gui_metadata_section::GuiMetadataSection;
use crate::net::minecraft::client::resources::metadata::gui::gui_sprite_scaling::{
    GuiSpriteScaling, NineSlice,
};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::org::joml::matrix3x2f_stack::Matrix3x2fStack;

pub struct GuiGraphicsExtractor<'a> {
    gui_width: i32,
    gui_height: i32,
    pose: Matrix3x2fStack,
    scissor_stack: ScissorStack,
    gui_sprites: &'a TextureAtlas,
    texture_manager: &'a TextureManager,
    gui_render_state: &'a mut GuiRenderState,
    mouse_x: i32,
    mouse_y: i32,
}

impl<'a> GuiGraphicsExtractor<'a> {
    /// The window's GUI size, the GUI atlas (`AtlasManager.getAtlasOrThrow(AtlasIds.GUI)`) and the
    /// texture manager are what Java reads from its `Minecraft`.
    pub fn new(
        gui_width: i32,
        gui_height: i32,
        gui_sprites: &'a TextureAtlas,
        texture_manager: &'a TextureManager,
        gui_render_state: &'a mut GuiRenderState,
        mouse_x: i32,
        mouse_y: i32,
    ) -> GuiGraphicsExtractor<'a> {
        GuiGraphicsExtractor {
            gui_width,
            gui_height,
            pose: Matrix3x2fStack::new(16),
            scissor_stack: ScissorStack::new(ScreenRectangle::new(0, 0, gui_width, gui_height)),
            gui_sprites,
            texture_manager,
            gui_render_state,
            mouse_x,
            mouse_y,
        }
    }

    pub fn gui_width(&self) -> i32 {
        self.gui_width
    }

    pub fn gui_height(&self) -> i32 {
        self.gui_height
    }

    pub fn mouse_x(&self) -> i32 {
        self.mouse_x
    }

    pub fn mouse_y(&self) -> i32 {
        self.mouse_y
    }

    pub fn pose(&mut self) -> &mut Matrix3x2fStack {
        &mut self.pose
    }

    pub fn next_stratum(&mut self) {
        self.gui_render_state.next_stratum();
    }

    pub fn blur_before_this_stratum(&mut self) {
        self.gui_render_state.blur_before_this_stratum();
    }

    pub fn enable_scissor(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let rectangle =
            ScreenRectangle::new(x0, y0, x1 - x0, y1 - y0).transform_axis_aligned(&self.pose);
        self.scissor_stack.push(rectangle);
    }

    pub fn disable_scissor(&mut self) {
        self.scissor_stack.pop();
    }

    pub fn contains_point_in_scissor(&self, x: i32, y: i32) -> bool {
        self.scissor_stack.contains_point(x, y)
    }

    pub fn horizontal_line(&mut self, mut x0: i32, mut x1: i32, y: i32, col: i32) {
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        self.fill(x0, y, x1 + 1, y + 1, col);
    }

    pub fn vertical_line(&mut self, x: i32, mut y0: i32, mut y1: i32, col: i32) {
        if y1 < y0 {
            std::mem::swap(&mut y0, &mut y1);
        }
        self.fill(x, y0 + 1, x + 1, y1, col);
    }

    pub fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, col: i32) {
        self.fill_with(&RenderPipelines::get().GUI, x0, y0, x1, y1, col);
    }

    /// `fill(RenderPipeline, int, int, int, int, int)`: the larger coordinates first.
    pub fn fill_with(
        &mut self,
        pipeline: &Arc<RenderPipeline>,
        mut x0: i32,
        mut y0: i32,
        mut x1: i32,
        mut y1: i32,
        col: i32,
    ) {
        if x0 < x1 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if y0 < y1 {
            std::mem::swap(&mut y0, &mut y1);
        }
        self.inner_fill(
            pipeline,
            TextureSetup::no_texture(),
            x0,
            y0,
            x1,
            y1,
            col,
            None,
        );
    }

    pub fn fill_gradient(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, col1: i32, col2: i32) {
        self.inner_fill(
            &RenderPipelines::get().GUI,
            TextureSetup::no_texture(),
            x0,
            y0,
            x1,
            y1,
            col1,
            Some(col2),
        );
    }

    /// `fill(RenderPipeline, TextureSetup, int, int, int, int)`.
    pub fn fill_textured(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture_setup: TextureSetup,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) {
        self.inner_fill(render_pipeline, texture_setup, x0, y0, x1, y1, -1, None);
    }

    pub fn outline(&mut self, x: i32, y: i32, width: i32, height: i32, color: i32) {
        self.fill(x, y, x + width, y + 1, color);
        self.fill(x, y + height - 1, x + width, y + height, color);
        self.fill(x, y + 1, x + 1, y + height - 1, color);
        self.fill(x + width - 1, y + 1, x + width, y + height - 1, color);
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_fill(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture_setup: TextureSetup,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color1: i32,
        color2: Option<i32>,
    ) {
        let state = ColoredRectangleRenderState::new(
            Arc::clone(render_pipeline),
            texture_setup,
            *self.pose,
            x0,
            y0,
            x1,
            y1,
            color1,
            color2.unwrap_or(color1),
            self.scissor_stack.peek(),
        );
        self.gui_render_state.add_gui_element(Box::new(state));
    }

    pub fn text_highlight(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, invert_text: bool) {
        if invert_text {
            self.fill_with(&RenderPipelines::get().GUI_INVERT, x0, y0, x1, y1, -1);
        }
        self.fill_with(
            &RenderPipelines::get().GUI_TEXT_HIGHLIGHT,
            x0,
            y0,
            x1,
            y1,
            -16776961,
        );
    }

    /// `blit(RenderPipeline, Identifier, int, int, float, float, int, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_color(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture: &Identifier,
        x: i32,
        y: i32,
        u: f32,
        v: f32,
        width: i32,
        height: i32,
        texture_width: i32,
        texture_height: i32,
        color: i32,
    ) {
        self.blit_region_color(
            render_pipeline,
            texture,
            x,
            y,
            u,
            v,
            width,
            height,
            width,
            height,
            texture_width,
            texture_height,
            color,
        );
    }

    /// `blit(RenderPipeline, Identifier, int, int, float, float, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture: &Identifier,
        x: i32,
        y: i32,
        u: f32,
        v: f32,
        width: i32,
        height: i32,
        texture_width: i32,
        texture_height: i32,
    ) {
        self.blit_region_color(
            render_pipeline,
            texture,
            x,
            y,
            u,
            v,
            width,
            height,
            width,
            height,
            texture_width,
            texture_height,
            -1,
        );
    }

    /// `blit(RenderPipeline, Identifier, int, int, float, float, int, int, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_region(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture: &Identifier,
        x: i32,
        y: i32,
        u: f32,
        v: f32,
        width: i32,
        height: i32,
        src_width: i32,
        src_height: i32,
        texture_width: i32,
        texture_height: i32,
    ) {
        self.blit_region_color(
            render_pipeline,
            texture,
            x,
            y,
            u,
            v,
            width,
            height,
            src_width,
            src_height,
            texture_width,
            texture_height,
            -1,
        );
    }

    /// `blit(RenderPipeline, Identifier, int, int, float, float, int, int, int, int, int, int,
    /// int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_region_color(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        texture: &Identifier,
        x: i32,
        y: i32,
        u: f32,
        v: f32,
        width: i32,
        height: i32,
        src_width: i32,
        src_height: i32,
        texture_width: i32,
        texture_height: i32,
        color: i32,
    ) {
        self.inner_blit_location(
            render_pipeline,
            texture,
            x,
            x + width,
            y,
            y + height,
            (u + 0.0) / texture_width as f32,
            (u + src_width as f32) / texture_width as f32,
            (v + 0.0) / texture_height as f32,
            (v + src_height as f32) / texture_height as f32,
            color,
        );
    }

    /// `blit(GpuTextureView, GpuSampler, int, int, int, int, float, float, float, float)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_texture(
        &mut self,
        texture_view: &GpuTextureView,
        sampler: &Rc<dyn GpuSampler>,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
    ) {
        self.inner_blit(
            &RenderPipelines::get().GUI_TEXTURED,
            texture_view.clone(),
            Rc::clone(sampler),
            x0,
            y0,
            x1,
            y1,
            u0,
            u1,
            v0,
            v1,
            -1,
        );
    }

    /// `blitSprite(RenderPipeline, Identifier, int, int, int, int)`.
    pub fn blit_sprite(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.blit_sprite_color(render_pipeline, location, x, y, width, height, -1);
    }

    /// `blitSprite(RenderPipeline, Identifier, int, int, int, int, float)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_sprite_alpha(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        alpha: f32,
    ) {
        self.blit_sprite_color(
            render_pipeline,
            location,
            x,
            y,
            width,
            height,
            ARGB::white_f32(alpha),
        );
    }

    /// `blitSprite(RenderPipeline, Identifier, int, int, int, int, int)`: the sprite scaled as its
    /// `gui` section says.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_sprite_color(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: i32,
    ) {
        let sprite = self.gui_sprites.get_sprite(location);
        match get_sprite_scaling(sprite) {
            GuiSpriteScaling::Stretch => {
                self.blit_atlas_sprite_color(render_pipeline, sprite, x, y, width, height, color)
            }
            GuiSpriteScaling::Tile(tile) => self.blit_tiled_sprite(
                render_pipeline,
                sprite,
                x,
                y,
                width,
                height,
                0,
                0,
                tile.width,
                tile.height,
                tile.width,
                tile.height,
                color,
            ),
            GuiSpriteScaling::NineSlice(nine_slice) => self.blit_nine_sliced_sprite(
                render_pipeline,
                sprite,
                &nine_slice,
                x,
                y,
                width,
                height,
                color,
            ),
        }
    }

    /// `blitSprite(RenderPipeline, Identifier, int, int, int, int, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_sprite_region(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        sprite_width: i32,
        sprite_height: i32,
        texture_x: i32,
        texture_y: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.blit_sprite_region_color(
            render_pipeline,
            location,
            sprite_width,
            sprite_height,
            texture_x,
            texture_y,
            x,
            y,
            width,
            height,
            -1,
        );
    }

    /// `blitSprite(RenderPipeline, Identifier, int, int, int, int, int, int, int, int, int)`: a
    /// region of a stretched sprite, or a scaled sprite cut to the region by a scissor.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_sprite_region_color(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        sprite_width: i32,
        sprite_height: i32,
        texture_x: i32,
        texture_y: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: i32,
    ) {
        let sprite = self.gui_sprites.get_sprite(location);
        if get_sprite_scaling(sprite) == GuiSpriteScaling::Stretch {
            self.blit_atlas_sprite_region(
                render_pipeline,
                sprite,
                sprite_width,
                sprite_height,
                texture_x,
                texture_y,
                x,
                y,
                width,
                height,
                color,
            );
        } else {
            self.enable_scissor(x, y, x + width, y + height);
            self.blit_sprite_color(
                render_pipeline,
                location,
                x - texture_x,
                y - texture_y,
                sprite_width,
                sprite_height,
                color,
            );
            self.disable_scissor();
        }
    }

    /// `blitSprite(RenderPipeline, TextureAtlasSprite, int, int, int, int)`.
    pub fn blit_atlas_sprite(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        sprite: &TextureAtlasSprite,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        self.blit_atlas_sprite_color(render_pipeline, sprite, x, y, width, height, -1);
    }

    /// `blitSprite(RenderPipeline, TextureAtlasSprite, int, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_atlas_sprite_color(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        sprite: &TextureAtlasSprite,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: i32,
    ) {
        if width != 0 && height != 0 {
            self.inner_blit_location(
                render_pipeline,
                sprite.atlas_location(),
                x,
                x + width,
                y,
                y + height,
                sprite.get_u0(),
                sprite.get_u1(),
                sprite.get_v0(),
                sprite.get_v1(),
                color,
            );
        }
    }

    /// The private `blitSprite(RenderPipeline, TextureAtlasSprite, int, int, int, int, int, int,
    /// int, int, int)`: a region of the sprite.
    #[allow(clippy::too_many_arguments)]
    fn blit_atlas_sprite_region(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        sprite: &TextureAtlasSprite,
        sprite_width: i32,
        sprite_height: i32,
        texture_x: i32,
        texture_y: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: i32,
    ) {
        if width != 0 && height != 0 {
            self.inner_blit_location(
                render_pipeline,
                sprite.atlas_location(),
                x,
                x + width,
                y,
                y + height,
                sprite.get_u(texture_x as f32 / sprite_width as f32),
                sprite.get_u((texture_x + width) as f32 / sprite_width as f32),
                sprite.get_v(texture_y as f32 / sprite_height as f32),
                sprite.get_v((texture_y + height) as f32 / sprite_height as f32),
                color,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn blit_nine_sliced_sprite(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        sprite: &TextureAtlasSprite,
        nine_slice: &NineSlice,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: i32,
    ) {
        let border = nine_slice.border;
        let border_left = border.left.min(width / 2);
        let border_right = border.right.min(width / 2);
        let border_top = border.top.min(height / 2);
        let border_bottom = border.bottom.min(height / 2);
        let (nw, nh) = (nine_slice.width, nine_slice.height);
        let p = render_pipeline;
        if width == nw && height == nh {
            self.blit_atlas_sprite_region(p, sprite, nw, nh, 0, 0, x, y, width, height, color);
        } else if height == nh {
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                0,
                0,
                x,
                y,
                border_left,
                height,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x + border_left,
                y,
                width - border_right - border_left,
                height,
                border_left,
                0,
                nw - border_right - border_left,
                nh,
                nw,
                nh,
                color,
            );
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                nw - border_right,
                0,
                x + width - border_right,
                y,
                border_right,
                height,
                color,
            );
        } else if width == nw {
            self.blit_atlas_sprite_region(p, sprite, nw, nh, 0, 0, x, y, width, border_top, color);
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x,
                y + border_top,
                width,
                height - border_bottom - border_top,
                0,
                border_top,
                nw,
                nh - border_bottom - border_top,
                nw,
                nh,
                color,
            );
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                0,
                nh - border_bottom,
                x,
                y + height - border_bottom,
                width,
                border_bottom,
                color,
            );
        } else {
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                0,
                0,
                x,
                y,
                border_left,
                border_top,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x + border_left,
                y,
                width - border_right - border_left,
                border_top,
                border_left,
                0,
                nw - border_right - border_left,
                border_top,
                nw,
                nh,
                color,
            );
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                nw - border_right,
                0,
                x + width - border_right,
                y,
                border_right,
                border_top,
                color,
            );
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                0,
                nh - border_bottom,
                x,
                y + height - border_bottom,
                border_left,
                border_bottom,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x + border_left,
                y + height - border_bottom,
                width - border_right - border_left,
                border_bottom,
                border_left,
                nh - border_bottom,
                nw - border_right - border_left,
                border_bottom,
                nw,
                nh,
                color,
            );
            self.blit_atlas_sprite_region(
                p,
                sprite,
                nw,
                nh,
                nw - border_right,
                nh - border_bottom,
                x + width - border_right,
                y + height - border_bottom,
                border_right,
                border_bottom,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x,
                y + border_top,
                border_left,
                height - border_bottom - border_top,
                0,
                border_top,
                border_left,
                nh - border_bottom - border_top,
                nw,
                nh,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x + border_left,
                y + border_top,
                width - border_right - border_left,
                height - border_bottom - border_top,
                border_left,
                border_top,
                nw - border_right - border_left,
                nh - border_bottom - border_top,
                nw,
                nh,
                color,
            );
            self.blit_nine_slice_inner_segment(
                p,
                nine_slice,
                sprite,
                x + width - border_right,
                y + border_top,
                border_right,
                height - border_bottom - border_top,
                nw - border_right,
                border_top,
                border_right,
                nh - border_bottom - border_top,
                nw,
                nh,
                color,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn blit_nine_slice_inner_segment(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        nine_slice: &NineSlice,
        sprite: &TextureAtlasSprite,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        texture_x: i32,
        texture_y: i32,
        texture_width: i32,
        texture_height: i32,
        sprite_width: i32,
        sprite_height: i32,
        color: i32,
    ) {
        if width > 0 && height > 0 {
            if nine_slice.stretch_inner {
                self.inner_blit_location(
                    render_pipeline,
                    sprite.atlas_location(),
                    x,
                    x + width,
                    y,
                    y + height,
                    sprite.get_u(texture_x as f32 / sprite_width as f32),
                    sprite.get_u((texture_x + texture_width) as f32 / sprite_width as f32),
                    sprite.get_v(texture_y as f32 / sprite_height as f32),
                    sprite.get_v((texture_y + texture_height) as f32 / sprite_height as f32),
                    color,
                );
            } else {
                self.blit_tiled_sprite(
                    render_pipeline,
                    sprite,
                    x,
                    y,
                    width,
                    height,
                    texture_x,
                    texture_y,
                    texture_width,
                    texture_height,
                    sprite_width,
                    sprite_height,
                    color,
                );
            }
        }
    }

    /// Java throws `IllegalArgumentException` for a tile size that isn't positive, the panic.
    #[allow(clippy::too_many_arguments)]
    fn blit_tiled_sprite(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        sprite: &TextureAtlasSprite,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        texture_x: i32,
        texture_y: i32,
        tile_width: i32,
        tile_height: i32,
        sprite_width: i32,
        sprite_height: i32,
        color: i32,
    ) {
        if width > 0 && height > 0 {
            if tile_width > 0 && tile_height > 0 {
                let (texture, sampler) = self.texture_manager.get_texture(sprite.atlas_location());
                let state = TiledBlitRenderState::new(
                    Arc::clone(render_pipeline),
                    TextureSetup::single_texture(texture.clone(), Rc::clone(sampler)),
                    *self.pose,
                    tile_width,
                    tile_height,
                    x,
                    y,
                    x + width,
                    y + height,
                    sprite.get_u(texture_x as f32 / sprite_width as f32),
                    sprite.get_u((texture_x + tile_width) as f32 / sprite_width as f32),
                    sprite.get_v(texture_y as f32 / sprite_height as f32),
                    sprite.get_v((texture_y + tile_height) as f32 / sprite_height as f32),
                    color,
                    self.scissor_stack.peek(),
                );
                self.gui_render_state.add_gui_element(Box::new(state));
            } else {
                panic!("Tile size must be positive, got {tile_width}x{tile_height}");
            }
        }
    }

    /// `innerBlit(RenderPipeline, Identifier, ...)`: the texture the texture manager has at the
    /// location.
    #[allow(clippy::too_many_arguments)]
    fn inner_blit_location(
        &mut self,
        render_pipeline: &Arc<RenderPipeline>,
        location: &Identifier,
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
        color: i32,
    ) {
        let (texture, sampler) = self.texture_manager.get_texture(location);
        let (texture, sampler) = (texture.clone(), Rc::clone(sampler));
        self.inner_blit(
            render_pipeline,
            texture,
            sampler,
            x0,
            y0,
            x1,
            y1,
            u0,
            u1,
            v0,
            v1,
            color,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn inner_blit(
        &mut self,
        pipeline: &Arc<RenderPipeline>,
        texture_view: GpuTextureView,
        sampler: Rc<dyn GpuSampler>,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
        color: i32,
    ) {
        let state = BlitRenderState::new(
            Arc::clone(pipeline),
            TextureSetup::single_texture(texture_view, sampler),
            *self.pose,
            x0,
            y0,
            x1,
            y1,
            u0,
            u1,
            v0,
            v1,
            color,
            self.scissor_stack.peek(),
        );
        self.gui_render_state.add_gui_element(Box::new(state));
    }
}

fn get_sprite_scaling(sprite: &TextureAtlasSprite) -> GuiSpriteScaling {
    sprite
        .contents()
        .get_gui_metadata()
        .unwrap_or(GuiMetadataSection::DEFAULT)
        .scaling
}

/// `GuiGraphicsExtractor.ScissorStack`.
struct ScissorStack {
    screen_size: ScreenRectangle,
    stack: Vec<ScreenRectangle>,
}

impl ScissorStack {
    fn new(screen_size: ScreenRectangle) -> ScissorStack {
        ScissorStack {
            screen_size,
            stack: Vec::new(),
        }
    }

    fn push(&mut self, rectangle: ScreenRectangle) {
        let last_rectangle = self.stack.last().copied().unwrap_or(self.screen_size);
        let intersection = rectangle
            .intersection(&last_rectangle)
            .unwrap_or(ScreenRectangle::empty());
        self.stack.push(intersection);
    }

    /// Java throws `IllegalStateException` on an underflow, the panic.
    fn pop(&mut self) {
        if self.stack.pop().is_none() {
            panic!("Scissor stack underflow");
        }
    }

    fn peek(&self) -> Option<ScreenRectangle> {
        self.stack.last().copied()
    }

    /// Java's `stack.peek()` is the deque's first rectangle, the one pushed first.
    fn contains_point(&self, x: i32, y: i32) -> bool {
        self.stack
            .first()
            .is_none_or(|rectangle| rectangle.contains_point(x, y))
    }
}
