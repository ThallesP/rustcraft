//! net/minecraft/client/gui/components/AbstractSliderButton.java
//!
//! A slider: its track (`widget/slider`, highlighted while focused and the keys can't move it),
//! the handle at its value from 0 to 1 (highlighted while hovered or movable), and its message
//! over both. A click or a drag puts the value under the mouse; Enter, Space or the keypad's Enter
//! lets the arrows move it. Java's abstract class is `AbstractSliderButton`, the struct a slider
//! holds, and its abstract `updateMessage` and `applyValue` are `SliderClass`'s, which the
//! functions here call.
//!
//! Not ported yet: narration, the cursor's shapes (`handleCursor`), and the release's sound.

use crate::net::minecraft::client::gui::components::abstract_widget::{
    self, AbstractWidget, AbstractWidgetClass,
};
use crate::net::minecraft::client::gui::gui_graphics_extractor::{
    GuiGraphicsExtractor, HoveredTextEffects,
};
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::input_type::{self, InputType};
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;

pub const TEXT_MARGIN: i32 = 2;
pub const DEFAULT_HEIGHT: i32 = 20;
pub const HANDLE_WIDTH: i32 = 8;

/// `AbstractSliderButton`'s fields, with `WithInactiveMessage`'s in its widget.
pub struct AbstractSliderButton {
    pub widget: AbstractWidget,
    pub value: f64,
    pub can_change_value: bool,
    dragging: bool,
}

impl AbstractSliderButton {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: Component,
        initial_value: f64,
    ) -> AbstractSliderButton {
        AbstractSliderButton {
            widget: AbstractWidget::with_inactive_message(x, y, width, height, message),
            value: initial_value,
            can_change_value: false,
            dragging: false,
        }
    }
}

/// A slider's own methods: the abstract ones, and the slider it holds.
pub trait SliderClass: AbstractWidgetClass {
    fn slider(&self) -> &AbstractSliderButton;

    fn slider_mut(&mut self) -> &mut AbstractSliderButton;

    fn update_message(&mut self);

    fn apply_value(&mut self);
}

fn get_sprite<S: SliderClass + ?Sized>(this: &S) -> Identifier {
    let path = if this.is_active() && this.is_focused() && !this.slider().can_change_value {
        "widget/slider_highlighted"
    } else {
        "widget/slider"
    };
    Identifier::with_default_namespace(path)
}

fn get_handle_sprite<S: SliderClass + ?Sized>(this: &S) -> Identifier {
    let path = if !this.is_active() || !this.is_hovered() && !this.slider().can_change_value {
        "widget/slider_handle"
    } else {
        "widget/slider_handle_highlighted"
    };
    Identifier::with_default_namespace(path)
}

/// `extractWidgetRenderState`: the track, the handle at the value, and the message over them.
pub fn extract_widget_render_state<S: SliderClass + ?Sized>(
    this: &S,
    graphics: &mut GuiGraphicsExtractor,
) {
    let widget = this.widget();
    let (x, y) = (widget.get_x(), widget.get_y());
    let color = ARGB::white_f32(widget.alpha);
    graphics.blit_sprite_color(
        &RenderPipelines::get().GUI_TEXTURED,
        &get_sprite(this),
        x,
        y,
        this.get_width(),
        this.get_height(),
        color,
    );
    graphics.blit_sprite_color(
        &RenderPipelines::get().GUI_TEXTURED,
        &get_handle_sprite(this),
        x + (this.slider().value * f64::from(widget.width - HANDLE_WIDTH)) as i32,
        y,
        HANDLE_WIDTH,
        this.get_height(),
        color,
    );
    let message = this.get_message();
    let mut output = graphics.text_renderer_for_widget(widget.alpha, HoveredTextEffects::NONE);
    abstract_widget::extract_scrolling_string_over_contents(
        this,
        &mut output,
        &message,
        TEXT_MARGIN,
    );
    // handleCursor: the cursor's shapes aren't ported.
}

/// `onClick`: dragging, the value under the mouse.
pub fn on_click<S: SliderClass + ?Sized>(this: &mut S, event: &MouseButtonEvent) {
    this.slider_mut().dragging = this.widget().active;
    set_value_from_mouse(this, event);
}

/// `setFocused`: the keys can move the value once it's focused by the mouse or by Tab.
pub fn set_focused<S: SliderClass + ?Sized>(this: &mut S, focused: bool) {
    this.widget_mut().set_focused(focused);
    if !focused {
        this.slider_mut().can_change_value = false;
    } else {
        let last_input_type = input_type::last_input_type();
        if last_input_type == InputType::MOUSE || last_input_type == InputType::KEYBOARD_TAB {
            this.slider_mut().can_change_value = true;
        }
    }
}

/// `keyPressed`: a selection key toggles whether the arrows move the value, by a pixel.
pub fn key_pressed<S: SliderClass + ?Sized>(this: &mut S, event: &KeyEvent) -> bool {
    if event.is_selection() {
        this.slider_mut().can_change_value = !this.slider().can_change_value;
        return true;
    }
    if this.slider().can_change_value {
        let (left, right) = (event.is_left(), event.is_right());
        if left || right {
            let direction: f32 = if left { -1.0 } else { 1.0 };
            let width = this.widget().width;
            let value = this.slider().value + f64::from(direction / (width - 8) as f32);
            set_value(this, value);
            return true;
        }
    }
    false
}

fn set_value_from_mouse<S: SliderClass + ?Sized>(this: &mut S, event: &MouseButtonEvent) {
    let widget = this.widget();
    let value = (event.x - f64::from(widget.get_x() + 4)) / f64::from(widget.width - 8);
    set_value(this, value);
}

/// `setValue`: clamped, applied when it changed, and the message made again.
pub fn set_value<S: SliderClass + ?Sized>(this: &mut S, new_value: f64) {
    let old_value = this.slider().value;
    this.slider_mut().value = Mth::clamp_f64(new_value, 0.0, 1.0);
    if old_value != this.slider().value {
        this.apply_value();
    }
    this.update_message();
}

/// `onDrag`: the value under the mouse.
pub fn on_drag<S: SliderClass + ?Sized>(this: &mut S, event: &MouseButtonEvent) {
    set_value_from_mouse(this, event);
}

/// `onRelease`: no longer dragging (the release's sound isn't ported).
pub fn on_release<S: SliderClass + ?Sized>(this: &mut S) {
    this.slider_mut().dragging = false;
}
