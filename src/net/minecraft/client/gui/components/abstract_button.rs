//! net/minecraft/client/gui/components/AbstractButton.java
//!
//! A button's sprite (nine-sliced `widget/button`, highlighted when hovered or focused, greyed
//! when inactive), its label centered over it, and a press by a click or by Enter, Space or the
//! keypad's Enter. The abstract class's state and code are `Button`'s, its only subclass ported
//! (`button.rs`); its static parts are here.

use crate::net::minecraft::client::gui::active_text_collector::ActiveTextCollector;
use crate::net::minecraft::client::gui::components::abstract_widget::{self, AbstractWidgetClass};
use crate::net::minecraft::client::gui::components::widget_sprites::WidgetSprites;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;

pub const TEXT_MARGIN: i32 = 2;

/// `SPRITES`.
pub fn sprites() -> WidgetSprites {
    WidgetSprites::three(
        Identifier::with_default_namespace("widget/button"),
        Identifier::with_default_namespace("widget/button_disabled"),
        Identifier::with_default_namespace("widget/button_highlighted"),
    )
}

/// `extractDefaultLabel`: the message, two pixels in from the sides.
pub fn extract_default_label<W: AbstractWidgetClass + ?Sized>(
    this: &W,
    output: &mut dyn ActiveTextCollector,
) {
    let message = this.get_message();
    abstract_widget::extract_scrolling_string_over_contents(this, output, &message, TEXT_MARGIN);
}

/// `extractDefaultSprite`: the button's sprite for its state, highlighted as `highlighted` says
/// (`overrideRenderHighlightedSprite`'s answer, or hovered or focused).
pub fn extract_default_sprite<W: AbstractWidgetClass + ?Sized>(
    this: &W,
    graphics: &mut GuiGraphicsExtractor,
    highlighted: bool,
) {
    let widget = this.widget();
    graphics.blit_sprite_color(
        &RenderPipelines::get().GUI_TEXTURED,
        sprites().get(widget.active, highlighted),
        widget.get_x(),
        widget.get_y(),
        this.get_width(),
        this.get_height(),
        ARGB::white_f32(widget.alpha),
    );
}
