//! net/minecraft/client/gui/components/LogoRenderer.java
//!
//! The title's "Minecraft" logo and the "Java Edition" under it. Java shows the "Minceraft" logo
//! once in 10,000 renderers, from a thread-local random; the port does the same.

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::random_source;

pub struct LogoRenderer {
    show_easter_egg: bool,
    keep_logo_through_fade: bool,
}

impl LogoRenderer {
    pub const MINECRAFT_LOGO: &'static str = "textures/gui/title/minecraft.png";
    pub const EASTER_EGG_LOGO: &'static str = "textures/gui/title/minceraft.png";
    pub const MINECRAFT_EDITION: &'static str = "textures/gui/title/edition.png";
    pub const LOGO_WIDTH: i32 = 256;
    pub const LOGO_HEIGHT: i32 = 44;
    pub const DEFAULT_HEIGHT_OFFSET: i32 = 30;

    pub fn new(keep_logo_through_fade: bool) -> LogoRenderer {
        LogoRenderer {
            show_easter_egg: random_source::create_thread_local_instance().next_float() < 1.0E-4,
            keep_logo_through_fade,
        }
    }

    /// `extractRenderState(GuiGraphicsExtractor, int, float)`: 30 pixels down.
    pub fn extract_render_state(
        &self,
        graphics: &mut GuiGraphicsExtractor,
        width: i32,
        alpha: f32,
    ) {
        self.extract_render_state_at(graphics, width, alpha, LogoRenderer::DEFAULT_HEIGHT_OFFSET);
    }

    pub fn extract_render_state_at(
        &self,
        graphics: &mut GuiGraphicsExtractor,
        width: i32,
        alpha: f32,
        height_offset: i32,
    ) {
        let logo_x = width / 2 - 128;
        let effective_alpha = if self.keep_logo_through_fade {
            1.0
        } else {
            alpha
        };
        let color = ARGB::white_f32(effective_alpha);
        let logo = if self.show_easter_egg {
            LogoRenderer::EASTER_EGG_LOGO
        } else {
            LogoRenderer::MINECRAFT_LOGO
        };
        graphics.blit_color(
            &RenderPipelines::get().GUI_TEXTURED,
            &Identifier::with_default_namespace(logo),
            logo_x,
            height_offset,
            0.0,
            0.0,
            256,
            44,
            256,
            64,
            color,
        );
        let edition_x = width / 2 - 64;
        let y = height_offset + 44 - 7;
        graphics.blit_color(
            &RenderPipelines::get().GUI_TEXTURED,
            &Identifier::with_default_namespace(LogoRenderer::MINECRAFT_EDITION),
            edition_x,
            y,
            0.0,
            0.0,
            128,
            14,
            128,
            16,
            color,
        );
    }

    pub fn keep_logo_through_fade(&self) -> bool {
        self.keep_logo_through_fade
    }
}
