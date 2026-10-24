//! net/minecraft/client/gui/components/SpriteIconButton.java
//!
//! A button with an icon from the GUI atlas, centered (`CenteredIcon`) or right of its message
//! (`TextAndIcon`), and a loading state that greys it out with a loading sprite. The class's fields
//! are `SpriteIconButton`, which a `Button` of either kind holds (`ButtonKind`), and its builder
//! makes the button.

use crate::net::minecraft::client::gui::active_text_collector::ActiveTextCollector;
use crate::net::minecraft::client::gui::components::abstract_widget::AbstractWidgetClass;
use crate::net::minecraft::client::gui::components::button::{Button, ButtonKind, OnPress};
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::components::widget_sprites::WidgetSprites;
use crate::net::minecraft::client::gui::gui_graphics_extractor::{
    GuiGraphicsExtractor, HoveredTextEffects,
};
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;

pub struct SpriteIconButton {
    pub sprite: WidgetSprites,
    pub sprite_width: i32,
    pub sprite_height: i32,
    pub sprite_offset_x: i32,
    pub sprite_offset_y: i32,
    pub switch_to_loading_after_press: bool,
    default_tooltip: Option<Tooltip>,
    pub loading: bool,
}

impl SpriteIconButton {
    const LOADING_SPRITE_W: i32 = 5;
    const LOADING_SPRITE_H: i32 = 2;

    /// The constructor's own fields; the button's tooltip is the default one, which the caller sets.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sprite_width: i32,
        sprite_height: i32,
        sprite_offset_x: i32,
        sprite_offset_y: i32,
        sprite: WidgetSprites,
        tooltip: Option<Component>,
        switch_to_loading_after_press: bool,
    ) -> SpriteIconButton {
        SpriteIconButton {
            sprite,
            sprite_width,
            sprite_height,
            sprite_offset_x,
            sprite_offset_y,
            switch_to_loading_after_press,
            default_tooltip: tooltip.map(Tooltip::create),
            loading: false,
        }
    }

    pub fn default_tooltip(&self) -> Option<Tooltip> {
        self.default_tooltip.clone()
    }

    /// `setLoading(boolean)`: the tooltip the button shows now, the loading one or its default.
    pub fn set_loading(&mut self, loading: bool) -> Option<Tooltip> {
        self.loading = loading;
        if loading {
            Some(Tooltip::create(
                Component::translatable("gui.friends.button.loading").into(),
            ))
        } else {
            self.default_tooltip.clone()
        }
    }

    pub fn builder(message: Component, on_press: OnPress, icon_only: bool) -> Builder {
        Builder {
            message,
            on_press,
            icon_only,
            width: 150,
            height: 20,
            sprite: None,
            sprite_width: 0,
            sprite_height: 0,
            sprite_offset_x: 0,
            sprite_offset_y: 0,
            tooltip: None,
            switch_to_loading_after_press: false,
        }
    }

    /// `extractSprite`.
    pub fn extract_sprite(button: &Button, graphics: &mut GuiGraphicsExtractor, x: i32, y: i32) {
        let icon = button.sprite_icon().expect("a sprite icon button");
        graphics.blit_sprite_alpha(
            &RenderPipelines::get().GUI_TEXTURED,
            icon.sprite
                .get(button.is_active(), button.is_hovered_or_focused()),
            x,
            y,
            icon.sprite_width,
            icon.sprite_height,
            button.widget().alpha,
        );
    }

    /// `extractLoadingStateIfLoading`: the greyed button and the loading sprite, while loading.
    pub fn extract_loading_state_if_loading(
        button: &Button,
        graphics: &mut GuiGraphicsExtractor,
    ) -> bool {
        if !button.sprite_icon().is_some_and(|icon| icon.loading) {
            return false;
        }
        let widget = button.widget();
        let pipeline = &RenderPipelines::get().GUI_TEXTURED;
        graphics.blit_sprite_color(
            pipeline,
            &Identifier::with_default_namespace("widget/button_disabled"),
            widget.get_x(),
            widget.get_y(),
            button.get_width(),
            button.get_height(),
            ARGB::white_f32(widget.alpha),
        );
        graphics.blit_sprite_color(
            pipeline,
            &Identifier::with_default_namespace("friends/loading"),
            widget.get_x() + (button.get_width() - SpriteIconButton::LOADING_SPRITE_W) / 2,
            widget.get_y() + (button.get_height() - SpriteIconButton::LOADING_SPRITE_H) / 2,
            SpriteIconButton::LOADING_SPRITE_W,
            SpriteIconButton::LOADING_SPRITE_H,
            ARGB::white_f32(widget.alpha),
        );
        true
    }

    /// `CenteredIcon.extractContents`.
    pub fn extract_centered_icon(button: &Button, graphics: &mut GuiGraphicsExtractor) {
        if !SpriteIconButton::extract_loading_state_if_loading(button, graphics) {
            button.extract_default_sprite(graphics);
            let icon = button.sprite_icon().expect("a sprite icon button");
            let widget = button.widget();
            let x = icon.sprite_offset_x + widget.get_x() + button.get_width() / 2
                - icon.sprite_width / 2;
            let y = icon.sprite_offset_y + widget.get_y() + button.get_height() / 2
                - icon.sprite_height / 2;
            SpriteIconButton::extract_sprite(button, graphics, x, y);
        }
    }

    /// `TextAndIcon.extractContents`.
    pub fn extract_text_and_icon(button: &Button, graphics: &mut GuiGraphicsExtractor) {
        if !SpriteIconButton::extract_loading_state_if_loading(button, graphics) {
            button.extract_default_sprite(graphics);
            let icon = button.sprite_icon().expect("a sprite icon button");
            let widget = button.widget();
            let left = widget.get_x() + 2;
            let right = widget.get_x() + button.get_width() - icon.sprite_width - 4;
            let center_x = widget.get_x() + button.get_width() / 2;
            let message = button.get_message();
            {
                let mut output =
                    graphics.text_renderer_for_widget(widget.alpha, HoveredTextEffects::NONE);
                output.accept_scrolling(
                    &message,
                    center_x,
                    left,
                    right,
                    widget.get_y(),
                    widget.get_y() + button.get_height(),
                );
            }
            let x =
                icon.sprite_offset_x + widget.get_x() + button.get_width() - icon.sprite_width - 2;
            let y = icon.sprite_offset_y + widget.get_y() + button.get_height() / 2
                - icon.sprite_height / 2;
            SpriteIconButton::extract_sprite(button, graphics, x, y);
        }
    }
}

/// `SpriteIconButton.Builder`.
pub struct Builder {
    message: Component,
    on_press: OnPress,
    icon_only: bool,
    width: i32,
    height: i32,
    sprite: Option<WidgetSprites>,
    sprite_width: i32,
    sprite_height: i32,
    sprite_offset_x: i32,
    sprite_offset_y: i32,
    tooltip: Option<Component>,
    switch_to_loading_after_press: bool,
}

impl Builder {
    pub fn width(mut self, width: i32) -> Builder {
        self.width = width;
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> Builder {
        self.width = width;
        self.height = height;
        self
    }

    /// `sprite(Identifier, int, int)`.
    pub fn sprite(self, sprite: Identifier, sprite_width: i32, sprite_height: i32) -> Builder {
        self.sprites(WidgetSprites::of(sprite), sprite_width, sprite_height)
    }

    /// `sprite(WidgetSprites, int, int)`.
    pub fn sprites(
        mut self,
        sprite: WidgetSprites,
        sprite_width: i32,
        sprite_height: i32,
    ) -> Builder {
        self.sprite = Some(sprite);
        self.sprite_width = sprite_width;
        self.sprite_height = sprite_height;
        self
    }

    pub fn sprite_offset(mut self, sprite_offset_x: i32, sprite_offset_y: i32) -> Builder {
        self.sprite_offset_x = sprite_offset_x;
        self.sprite_offset_y = sprite_offset_y;
        self
    }

    pub fn tooltip(mut self, tooltip: Component) -> Builder {
        self.tooltip = Some(tooltip);
        self
    }

    /// `withTootip()` (sic): the message as the tooltip.
    pub fn with_tooltip(mut self) -> Builder {
        self.tooltip = Some(self.message.clone());
        self
    }

    pub fn switch_to_loading_after_press(mut self) -> Builder {
        self.switch_to_loading_after_press = true;
        self
    }

    /// `build()`, which panics where Java throws for a missing sprite.
    pub fn build(self) -> Button {
        let sprite = self.sprite.expect("Sprite not set");
        let icon = SpriteIconButton::new(
            self.sprite_width,
            self.sprite_height,
            self.sprite_offset_x,
            self.sprite_offset_y,
            sprite,
            self.tooltip,
            self.switch_to_loading_after_press,
        );
        let tooltip = icon.default_tooltip();
        let kind = if self.icon_only {
            ButtonKind::CenteredIcon(icon)
        } else {
            ButtonKind::TextAndIcon(icon)
        };
        let mut button = Button::new(
            0,
            0,
            self.width,
            self.height,
            self.message,
            self.on_press,
            kind,
        );
        button.set_tooltip(tooltip);
        button
    }
}
