//! net/minecraft/client/gui/screens/inventory/tooltip/TooltipRenderUtil.java

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct TooltipRenderUtil;

impl TooltipRenderUtil {
    pub const MOUSE_OFFSET: i32 = 12;
    pub const PADDING_LEFT: i32 = 3;
    pub const PADDING_RIGHT: i32 = 3;
    pub const PADDING_TOP: i32 = 3;
    pub const PADDING_BOTTOM: i32 = 3;

    /// `extractTooltipBackground`: the background and the frame, nine-sliced around the text with
    /// the margin the sprites' shadows take.
    pub fn extract_tooltip_background(
        graphics: &mut GuiGraphicsExtractor,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        style: Option<&Identifier>,
    ) {
        let x0 = x - 3 - 9;
        let y0 = y - 3 - 9;
        let padded_width = w + 3 + 3 + 18;
        let padded_height = h + 3 + 3 + 18;
        let pipeline = &RenderPipelines::get().GUI_TEXTURED;
        graphics.blit_sprite(
            pipeline,
            &TooltipRenderUtil::get_background_sprite(style),
            x0,
            y0,
            padded_width,
            padded_height,
        );
        graphics.blit_sprite(
            pipeline,
            &TooltipRenderUtil::get_frame_sprite(style),
            x0,
            y0,
            padded_width,
            padded_height,
        );
    }

    fn get_background_sprite(style: Option<&Identifier>) -> Identifier {
        match style {
            None => Identifier::with_default_namespace("tooltip/background"),
            Some(style) => style.with_path(&format!("tooltip/{}_background", style.get_path())),
        }
    }

    fn get_frame_sprite(style: Option<&Identifier>) -> Identifier {
        match style {
            None => Identifier::with_default_namespace("tooltip/frame"),
            Some(style) => style.with_path(&format!("tooltip/{}_frame", style.get_path())),
        }
    }
}
