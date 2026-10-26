//! net/minecraft/client/gui/components/Button.java, with AbstractButton.java's state and code
//!
//! A button that calls its `OnPress` when pressed. Java's subclasses (`Button.Plain`, the sprite
//! icon buttons, the friends button, `PlainTextButton`) differ in what they draw over their sprite and in a few
//! overrides, so they're `ButtonKind`'s variants, each with its class's own fields. The press
//! hands the callback the widget, Java's `Button`, which is all a callback reads of it.
//!
//! Not ported yet: narration (`createNarration`), and the click's sound.

use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_button;
use crate::net::minecraft::client::gui::components::abstract_widget::{
    AbstractWidget, AbstractWidgetClass,
};
use crate::net::minecraft::client::gui::components::friends_button::FriendsButton;
use crate::net::minecraft::client::gui::components::sprite_icon_button::SpriteIconButton;
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::{
    GuiGraphicsExtractor, HoveredTextEffects,
};
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_utils::ComponentUtils;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::mth::Mth;

/// `Button.OnPress`.
pub type OnPress = Box<dyn FnMut(&mut AbstractWidget)>;

pub struct Button {
    widget: AbstractWidget,
    /// `AbstractButton.overrideRenderHighlightedSprite`.
    override_render_highlighted_sprite: Option<Box<dyn Fn() -> bool>>,
    on_press: OnPress,
    pub kind: ButtonKind,
}

/// `Button`'s subclasses.
pub enum ButtonKind {
    /// `Button.Plain`: the sprite and the message.
    Plain,
    /// `SpriteIconButton.CenteredIcon`: the sprite and an icon in its middle.
    CenteredIcon(SpriteIconButton),
    /// `SpriteIconButton.TextAndIcon`: the message and an icon on the right.
    TextAndIcon(SpriteIconButton),
    /// `FriendsButton`, a centered icon with the count of friend requests.
    Friends(SpriteIconButton, FriendsButton),
    /// `PlainTextButton`: its text alone, underlined while hovered or focused.
    PlainText(PlainTextButton),
}

impl Button {
    pub const SMALL_WIDTH: i32 = 120;
    pub const DEFAULT_WIDTH: i32 = 150;
    pub const BIG_WIDTH: i32 = 200;
    pub const DEFAULT_HEIGHT: i32 = 20;
    pub const DEFAULT_SPACING: i32 = 8;

    pub fn builder(message: Component, on_press: OnPress) -> Builder {
        Builder::new(message, on_press)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: Component,
        on_press: OnPress,
        kind: ButtonKind,
    ) -> Button {
        Button {
            widget: AbstractWidget::with_inactive_message(x, y, width, height, message),
            override_render_highlighted_sprite: None,
            on_press,
            kind,
        }
    }

    /// `onPress(InputWithModifiers)`: a sprite icon button may show it's loading first.
    pub fn on_press(&mut self) {
        if let ButtonKind::CenteredIcon(icon)
        | ButtonKind::TextAndIcon(icon)
        | ButtonKind::Friends(icon, _) = &mut self.kind
            && icon.switch_to_loading_after_press
        {
            let tooltip = icon.set_loading(true);
            self.widget.set_tooltip(tooltip);
        }
        (self.on_press)(&mut self.widget);
    }

    pub fn set_override_render_highlighted_sprite(&mut self, sprite: Box<dyn Fn() -> bool>) {
        self.override_render_highlighted_sprite = Some(sprite);
    }

    pub fn set_tooltip(&mut self, tooltip: Option<Tooltip>) {
        self.widget.set_tooltip(tooltip);
    }

    /// Whether the sprite is the highlighted one.
    fn highlighted(&self) -> bool {
        match &self.override_render_highlighted_sprite {
            Some(sprite) => sprite(),
            None => self.is_hovered_or_focused(),
        }
    }

    pub fn extract_default_sprite(&self, graphics: &mut GuiGraphicsExtractor) {
        abstract_button::extract_default_sprite(self, graphics, self.highlighted());
    }

    /// `extractContents`.
    fn extract_contents(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
        match &self.kind {
            ButtonKind::Plain => {
                self.extract_default_sprite(graphics);
                let alpha = self.widget.alpha;
                let mut output = graphics.text_renderer_for_widget(alpha, HoveredTextEffects::NONE);
                abstract_button::extract_default_label(self, &mut output);
            }
            ButtonKind::CenteredIcon(_) => SpriteIconButton::extract_centered_icon(self, graphics),
            ButtonKind::TextAndIcon(_) => SpriteIconButton::extract_text_and_icon(self, graphics),
            ButtonKind::Friends(_, _) => FriendsButton::extract_contents(self, graphics),
            ButtonKind::PlainText(plain_text) => {
                let message = if self.is_hovered_or_focused() {
                    &plain_text.underlined_message
                } else {
                    &plain_text.message
                };
                let color = 16777215 | Mth::ceil_f32(self.widget.alpha * 255.0) << 24;
                graphics.text_component(
                    &plain_text.font,
                    message,
                    self.widget.get_x(),
                    self.widget.get_y(),
                    color,
                    true,
                );
            }
        }
    }

    /// The sprite icon button's fields, for one.
    pub fn sprite_icon(&self) -> Option<&SpriteIconButton> {
        match &self.kind {
            ButtonKind::CenteredIcon(icon)
            | ButtonKind::TextAndIcon(icon)
            | ButtonKind::Friends(icon, _) => Some(icon),
            ButtonKind::Plain | ButtonKind::PlainText(_) => None,
        }
    }
}

impl AbstractWidgetClass for Button {
    fn widget(&self) -> &AbstractWidget {
        &self.widget
    }

    fn widget_mut(&mut self) -> &mut AbstractWidget {
        &mut self.widget
    }

    /// `AbstractButton.extractWidgetRenderState`: the contents (the cursor isn't ported).
    fn extract_widget_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        self.extract_contents(graphics, mouse_x, mouse_y, a);
        graphics.request_cursor();
    }

    fn on_click(&mut self, _event: &MouseButtonEvent, _double_click: bool) {
        self.on_press();
    }

    /// `AbstractButton.keyPressed`: Enter, Space or the keypad's Enter press it.
    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        if !self.is_active() {
            false
        } else if event.is_selection() {
            self.on_press();
            true
        } else {
            false
        }
    }

    /// `SpriteIconButton.isActive`: not while it's loading.
    fn is_active(&self) -> bool {
        self.widget.visible
            && self.widget.active
            && !self.sprite_icon().is_some_and(|icon| icon.loading)
    }
}

/// `PlainTextButton`'s fields.
pub struct PlainTextButton {
    font: Rc<Font>,
    message: Component,
    underlined_message: Component,
}

impl PlainTextButton {
    /// `PlainTextButton(int, int, int, int, Component, OnPress, Font)`.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: Component,
        on_press: OnPress,
        font: Rc<Font>,
    ) -> Button {
        let underlined_message =
            ComponentUtils::merge_styles(&message, &Style::EMPTY.with_underlined(Some(true)));
        Button::new(
            x,
            y,
            width,
            height,
            message.clone(),
            on_press,
            ButtonKind::PlainText(PlainTextButton {
                font,
                message,
                underlined_message,
            }),
        )
    }
}

/// `Button.Builder`.
pub struct Builder {
    message: Component,
    on_press: OnPress,
    tooltip: Option<Tooltip>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Builder {
    pub fn new(message: Component, on_press: OnPress) -> Builder {
        Builder {
            message,
            on_press,
            tooltip: None,
            x: 0,
            y: 0,
            width: 150,
            height: 20,
        }
    }

    pub fn pos(mut self, x: i32, y: i32) -> Builder {
        self.x = x;
        self.y = y;
        self
    }

    pub fn width(mut self, width: i32) -> Builder {
        self.width = width;
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Builder {
        self.width = width;
        self.height = height;
        self
    }

    pub fn bounds(self, x: i32, y: i32, width: i32, height: i32) -> Builder {
        self.pos(x, y).size(width, height)
    }

    pub fn tooltip(mut self, tooltip: Option<Tooltip>) -> Builder {
        self.tooltip = tooltip;
        self
    }

    /// `build()`: a `Button.Plain`.
    pub fn build(self) -> Button {
        let mut button = Button::new(
            self.x,
            self.y,
            self.width,
            self.height,
            self.message,
            self.on_press,
            ButtonKind::Plain,
        );
        button.set_tooltip(self.tooltip);
        button
    }
}
