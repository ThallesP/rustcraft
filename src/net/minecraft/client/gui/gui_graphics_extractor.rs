//! net/minecraft/client/gui/GuiGraphicsExtractor.java
//!
//! What the GUI's code draws with: fills, blits and sprites from the GUI atlas (stretched, tiled or
//! nine-sliced as their `gui` section says) under a pose and a scissor stack, each made into an
//! element state in the frame's `GuiRenderState`. Java reads the window's GUI size, the atlases and
//! the texture manager from its `Minecraft`, a back-reference; here they're its fields, borrowed
//! from what owns them for the extract.
//!
//! Tooltips are deferred to the end of the screen's extract (`extractDeferredElements`), a stratum
//! above it, and the text collectors (`textRenderer`) draw a widget's or the chat's text.
//!
//! Not ported yet: items and their decorations, maps, the picture in picture states (entities,
//! skins, books, banners, the profiler chart), tooltip images, the hovered text's effects (the
//! style under the mouse, its hover tooltip and the clickable text's cursor), the cursor and the
//! preedit overlay.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::gui::active_text_collector::{ActiveTextCollector, Parameters};
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_component::{
    self, ClientTooltipComponent,
};
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_positioner::ClientTooltipPositioner;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::default_tooltip_positioner::DefaultTooltipPositioner;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::tooltip_render_util::TooltipRenderUtil;
use crate::net::minecraft::client::gui::text_alignment::TextAlignment;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::state::gui::blit_render_state::BlitRenderState;
use crate::net::minecraft::client::renderer::state::gui::colored_rectangle_render_state::ColoredRectangleRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_render_state::GuiRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_text_render_state::GuiTextRenderState;
use crate::net::minecraft::client::renderer::state::gui::tiled_blit_render_state::TiledBlitRenderState;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;
use crate::net::minecraft::client::resources::metadata::gui::gui_metadata_section::GuiMetadataSection;
use crate::net::minecraft::client::resources::metadata::gui::gui_sprite_scaling::{
    GuiSpriteScaling, NineSlice,
};
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::{FormattedText, Of};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::org::joml::matrix3x2f_stack::Matrix3x2fStack;

pub struct GuiGraphicsExtractor<'a> {
    gui_width: i32,
    gui_height: i32,
    /// `minecraft.options.getBackgroundColor(0.0F)`, behind `textWithBackdrop`'s text.
    text_background_color: i32,
    pose: Matrix3x2fStack,
    scissor_stack: ScissorStack,
    gui_sprites: &'a TextureAtlas,
    /// `minecraft.getTextureManager()`, which loads a texture a blit names the first time, on the
    /// device with the sampler cache's samplers.
    texture_manager: &'a mut TextureManager,
    device: &'a dyn GpuDevice,
    sampler_cache: &'a SamplerCache,
    gui_render_state: &'a mut GuiRenderState,
    mouse_x: i32,
    mouse_y: i32,
    /// `minecraft.font`.
    font: Rc<Font>,
    deferred_tooltip: Option<DeferredTooltip>,
}

/// `deferredTooltip`: the tooltip `setTooltipForNextFrame` asked for, drawn by
/// `extractDeferredElements` (Java's lambda over `tooltip`'s arguments).
struct DeferredTooltip {
    font: Rc<Font>,
    lines: Vec<Box<dyn ClientTooltipComponent>>,
    xo: i32,
    yo: i32,
    positioner: Box<dyn ClientTooltipPositioner>,
    style: Option<Identifier>,
    extra_space_after_first_line: bool,
}

/// `GuiGraphicsExtractor.HoveredTextEffects`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoveredTextEffects {
    NONE,
    TOOLTIP_ONLY,
    TOOLTIP_AND_CURSOR,
}

impl HoveredTextEffects {
    pub fn allow_tooltip(self) -> bool {
        self != HoveredTextEffects::NONE
    }

    pub fn allow_cursor_changes(self) -> bool {
        self == HoveredTextEffects::TOOLTIP_AND_CURSOR
    }

    pub fn not_clickable(can_tooltip: bool) -> HoveredTextEffects {
        if can_tooltip {
            HoveredTextEffects::TOOLTIP_ONLY
        } else {
            HoveredTextEffects::NONE
        }
    }
}

impl<'a> GuiGraphicsExtractor<'a> {
    /// The window's GUI size, the GUI atlas (`AtlasManager.getAtlasOrThrow(AtlasIds.GUI)`), the
    /// texture manager and the options' text background are what Java reads from its `Minecraft`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gui_width: i32,
        gui_height: i32,
        text_background_color: i32,
        gui_sprites: &'a TextureAtlas,
        texture_manager: &'a mut TextureManager,
        device: &'a dyn GpuDevice,
        sampler_cache: &'a SamplerCache,
        gui_render_state: &'a mut GuiRenderState,
        mouse_x: i32,
        mouse_y: i32,
        font: Rc<Font>,
    ) -> GuiGraphicsExtractor<'a> {
        GuiGraphicsExtractor {
            gui_width,
            gui_height,
            text_background_color,
            pose: Matrix3x2fStack::new(16),
            scissor_stack: ScissorStack::new(ScreenRectangle::new(0, 0, gui_width, gui_height)),
            gui_sprites,
            texture_manager,
            device,
            sampler_cache,
            gui_render_state,
            mouse_x,
            mouse_y,
            font,
            deferred_tooltip: None,
        }
    }

    /// `minecraft.font`.
    pub fn font(&self) -> &Rc<Font> {
        &self.font
    }

    /// `getTexture`: the texture at `location`, loaded the first time.
    fn get_texture(&mut self, location: &Identifier) -> (GpuTextureView, Rc<dyn GpuSampler>) {
        let (view, sampler) =
            self.texture_manager
                .get_or_load_texture(self.device, self.sampler_cache, location);
        (view.clone(), Rc::clone(sampler))
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

    /// `requestCursor`: the window's cursor shapes aren't ported, so it asks nothing.
    pub fn request_cursor(&mut self) {}

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

    /// `text(Font, String, int, int, int, boolean)`: nothing for none (Java's null).
    pub fn text_str(
        &mut self,
        font: &Rc<Font>,
        str: Option<&str>,
        x: i32,
        y: i32,
        color: i32,
        drop_shadow: bool,
    ) {
        if let Some(str) = str {
            let text = Language::get_instance().get_visual_order(&Of(str.to_owned()));
            self.text_sequence(font, &text, x, y, color, drop_shadow);
        }
    }

    /// `text(Font, FormattedCharSequence, int, int, int, boolean)`: nothing when transparent.
    pub fn text_sequence(
        &mut self,
        font: &Rc<Font>,
        str: &FormattedCharSequence,
        x: i32,
        y: i32,
        color: i32,
        drop_shadow: bool,
    ) {
        if ARGB::alpha(color) != 0 {
            self.gui_render_state.add_text(GuiTextRenderState::new(
                Rc::clone(font),
                str.clone(),
                *self.pose,
                x,
                y,
                color,
                0,
                drop_shadow,
                false,
                self.scissor_stack.peek(),
            ));
        }
    }

    /// `text(Font, Component, int, int, int, boolean)`: its visual order's
    /// (`getVisualOrderText`).
    pub fn text_component(
        &mut self,
        font: &Rc<Font>,
        str: &impl FormattedText,
        x: i32,
        y: i32,
        color: i32,
        drop_shadow: bool,
    ) {
        let text = Language::get_instance().get_visual_order(str);
        self.text_sequence(font, &text, x, y, color, drop_shadow);
    }

    /// `centeredText(Font, String, int, int, int)`.
    pub fn centered_text_str(&mut self, font: &Rc<Font>, str: &str, x: i32, y: i32, color: i32) {
        let width = font.width(str);
        self.text_str(font, Some(str), x - width / 2, y, color, true);
    }

    /// `centeredText(Font, Component, int, int, int)`.
    pub fn centered_text_component(
        &mut self,
        font: &Rc<Font>,
        text: &impl FormattedText,
        x: i32,
        y: i32,
        color: i32,
    ) {
        let to_render = Language::get_instance().get_visual_order(text);
        let width = font.width_sequence(&to_render);
        self.text_sequence(font, &to_render, x - width / 2, y, color, true);
    }

    /// `textWithWordWrap`: the text's lines at the width, one under another; where the next line
    /// would go.
    #[allow(clippy::too_many_arguments)]
    pub fn text_with_word_wrap(
        &mut self,
        font: &Rc<Font>,
        string: &impl FormattedText,
        x: i32,
        mut y: i32,
        width: i32,
        col: i32,
        drop_shadow: bool,
    ) -> i32 {
        for line in font.split(string, width) {
            self.text_sequence(font, &line, x, y, col, drop_shadow);
            y += 9;
        }
        y
    }

    /// `textWithBackdrop`: the text over the options' background, the text's alpha applied.
    pub fn text_with_backdrop(
        &mut self,
        font: &Rc<Font>,
        str: &impl FormattedText,
        text_x: i32,
        text_y: i32,
        text_width: i32,
        text_color: i32,
    ) {
        let background_color = self.text_background_color;
        if background_color != 0 {
            self.fill(
                text_x - 2,
                text_y - 2,
                text_x + text_width + 2,
                text_y + 9 + 2,
                ARGB::multiply(background_color, text_color),
            );
        }
        self.text_component(font, str, text_x, text_y, text_color, true);
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
                let (texture, sampler) = self.get_texture(sprite.atlas_location());
                let state = TiledBlitRenderState::new(
                    Arc::clone(render_pipeline),
                    TextureSetup::single_texture(texture, sampler),
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
        let (texture, sampler) = self.get_texture(location);
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
impl<'a> GuiGraphicsExtractor<'a> {
    /// `setTooltipForNextFrame(Font, List<FormattedCharSequence>, Optional, ClientTooltipPositioner,
    /// int, int, boolean, Identifier)`: a widget's tooltip, kept unless one is there already or
    /// `replace_existing`. Tooltip images aren't ported.
    #[allow(clippy::too_many_arguments)]
    pub fn set_tooltip_for_next_frame_positioned(
        &mut self,
        font: &Rc<Font>,
        tooltip: Vec<FormattedCharSequence>,
        positioner: Box<dyn ClientTooltipPositioner>,
        xo: i32,
        yo: i32,
        replace_existing: bool,
        style: Option<Identifier>,
    ) {
        let components = tooltip
            .into_iter()
            .map(client_tooltip_component::create)
            .collect();
        self.set_tooltip_for_next_frame_internal(
            font,
            components,
            xo,
            yo,
            positioner,
            style,
            replace_existing,
            false,
        );
    }

    /// `setTooltipForNextFrame(Font, List<? extends FormattedCharSequence>, int, int)`: by the
    /// default positioner.
    pub fn set_tooltip_for_next_frame_lines(
        &mut self,
        font: &Rc<Font>,
        lines: Vec<FormattedCharSequence>,
        xo: i32,
        yo: i32,
    ) {
        self.set_tooltip_for_next_frame_positioned(
            font,
            lines,
            Box::new(DefaultTooltipPositioner),
            xo,
            yo,
            false,
            None,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn set_tooltip_for_next_frame_internal(
        &mut self,
        font: &Rc<Font>,
        lines: Vec<Box<dyn ClientTooltipComponent>>,
        xo: i32,
        yo: i32,
        positioner: Box<dyn ClientTooltipPositioner>,
        style: Option<Identifier>,
        replace_existing: bool,
        extra_space_after_first_line: bool,
    ) {
        if !lines.is_empty() && (self.deferred_tooltip.is_none() || replace_existing) {
            self.deferred_tooltip = Some(DeferredTooltip {
                font: Rc::clone(font),
                lines,
                xo,
                yo,
                positioner,
                style,
                extra_space_after_first_line,
            });
        }
    }

    /// `tooltip`: the lines over the tooltip's background where the positioner puts them.
    #[allow(clippy::too_many_arguments)]
    pub fn tooltip(
        &mut self,
        font: &Rc<Font>,
        lines: &[Box<dyn ClientTooltipComponent>],
        xo: i32,
        yo: i32,
        positioner: &dyn ClientTooltipPositioner,
        style: Option<&Identifier>,
        extra_space_after_first_line: bool,
    ) {
        let mut text_width = 0;
        let mut temp_height = if extra_space_after_first_line && lines.len() != 1 {
            0
        } else {
            -2
        };
        for line in lines {
            text_width = text_width.max(line.get_width(font));
            temp_height += line.get_height(font);
        }
        let (w, h) = (text_width, temp_height);
        let (x, y) = positioner.position_tooltip(self.gui_width(), self.gui_height(), xo, yo, w, h);
        self.pose.push_matrix();
        TooltipRenderUtil::extract_tooltip_background(self, x, y, w, h, style);
        let mut local_y = y;
        for (i, line) in lines.iter().enumerate() {
            line.extract_text(self, font, x, local_y);
            local_y += line.get_height(font)
                + if extra_space_after_first_line && i == 0 {
                    2
                } else {
                    0
                };
        }
        local_y = y;
        for (i, line) in lines.iter().enumerate() {
            line.extract_image(font, x, local_y, w, h, self);
            local_y += line.get_height(font)
                + if extra_space_after_first_line && i == 0 {
                    2
                } else {
                    0
                };
        }
        self.pose.pop_matrix();
    }

    /// `extractDeferredElements`: the tooltip asked for, a stratum above the rest. The hovered
    /// text's effects and the preedit overlay aren't ported.
    pub fn extract_deferred_elements(&mut self, _mouse_x: i32, _mouse_y: i32, _a: f32) {
        if let Some(tooltip) = self.deferred_tooltip.take() {
            self.next_stratum();
            self.tooltip(
                &tooltip.font,
                &tooltip.lines,
                tooltip.xo,
                tooltip.yo,
                &*tooltip.positioner,
                tooltip.style.as_ref(),
                tooltip.extra_space_after_first_line,
            );
        }
    }

    fn create_default_text_parameters(&self, opacity: f32) -> Parameters {
        Parameters {
            pose: *self.pose,
            opacity,
            scissor: self.scissor_stack.peek(),
        }
    }

    /// `textRendererForWidget`: text at the widget's alpha.
    pub fn text_renderer_for_widget(
        &mut self,
        alpha: f32,
        hovered_text_effects: HoveredTextEffects,
    ) -> RenderingTextCollector<'_, 'a> {
        let parameters = self.create_default_text_parameters(alpha);
        RenderingTextCollector {
            graphics: self,
            default_parameters: parameters,
            hovered_text_effects,
        }
    }

    /// `textRenderer(HoveredTextEffects)`.
    pub fn text_renderer(
        &mut self,
        hovered_text_effects: HoveredTextEffects,
    ) -> RenderingTextCollector<'_, 'a> {
        self.text_renderer_for_widget(1.0, hovered_text_effects)
    }
}

/// `GuiGraphicsExtractor.RenderingTextCollector`: draws the text it's given. The style under the
/// mouse isn't looked for yet (the hovered text's effects), so the effects only decide whether the
/// text keeps its empty areas (`includeEmpty`).
pub struct RenderingTextCollector<'g, 'a> {
    graphics: &'g mut GuiGraphicsExtractor<'a>,
    default_parameters: Parameters,
    hovered_text_effects: HoveredTextEffects,
}

impl ActiveTextCollector for RenderingTextCollector<'_, '_> {
    fn font_width(&self, message: &Component) -> i32 {
        self.graphics.font.width_text(message)
    }

    fn default_parameters(&self) -> Parameters {
        self.default_parameters
    }

    fn set_default_parameters(&mut self, new_parameters: Parameters) {
        self.default_parameters = new_parameters;
    }

    fn accept(
        &mut self,
        alignment: TextAlignment,
        anchor_x: i32,
        y: i32,
        parameters: &Parameters,
        text: &FormattedCharSequence,
    ) {
        let needs_full_style_scan = self.hovered_text_effects.allow_cursor_changes()
            || self.hovered_text_effects.allow_tooltip();
        let font = Rc::clone(&self.graphics.font);
        let left_x = alignment.calculate_left_text(anchor_x, &font, text);
        let render_state = GuiTextRenderState::new(
            font,
            text.clone(),
            parameters.pose,
            left_x,
            y,
            ARGB::white_f32(parameters.opacity),
            0,
            true,
            needs_full_style_scan,
            parameters.scissor,
        );
        if ARGB::as8_bit_channel(parameters.opacity) != 0 {
            self.graphics.gui_render_state.add_text(render_state);
        }
    }
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
