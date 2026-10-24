//! net/minecraft/client/gui/components/StringWidget.java, with AbstractStringWidget.java's state
//!
//! A line of text, inactive (it takes no clicks), clipped with an ellipsis or scrolling when it's
//! wider than its maximum.

use std::rc::Rc;

use crate::net::minecraft::client::gui::active_text_collector::ActiveTextCollector;
use crate::net::minecraft::client::gui::components::abstract_string_widget;
use crate::net::minecraft::client::gui::components::abstract_widget::{
    self, AbstractWidget, AbstractWidgetClass,
};
use crate::net::minecraft::client::gui::components::component_render_utils::ComponentRenderUtils;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::component::Component;

/// `StringWidget.TextOverflow`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflow {
    CLAMPED,
    SCROLLING,
}

pub struct StringWidget {
    widget: AbstractWidget,
    font: Rc<Font>,
    max_width: i32,
    cached_width: std::cell::Cell<Option<i32>>,
    text_overflow: TextOverflow,
}

impl StringWidget {
    /// `StringWidget(Component, Font)`: as wide as the text, a line high.
    pub fn of(message: Component, font: Rc<Font>) -> StringWidget {
        let width = font.width_sequence(&Language::get_instance().get_visual_order(&message));
        StringWidget::new(0, 0, width, 9, message, font)
    }

    /// `StringWidget(int, int, Component, Font)`.
    pub fn sized(width: i32, height: i32, message: Component, font: Rc<Font>) -> StringWidget {
        StringWidget::new(0, 0, width, height, message, font)
    }

    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: Component,
        font: Rc<Font>,
    ) -> StringWidget {
        let mut widget = AbstractWidget::new(x, y, width, height, message);
        widget.active = false;
        StringWidget {
            widget,
            font,
            max_width: 0,
            cached_width: std::cell::Cell::new(None),
            text_overflow: TextOverflow::CLAMPED,
        }
    }

    pub fn set_max_width(mut self, max_width: i32, text_overflow: TextOverflow) -> StringWidget {
        self.max_width = max_width;
        self.text_overflow = text_overflow;
        self
    }

    pub fn get_font(&self) -> &Rc<Font> {
        &self.font
    }

    /// `visitLines`.
    pub fn visit_lines(&self, output: &mut dyn ActiveTextCollector) {
        let message = self.get_message();
        let font = &self.font;
        let max_width = if self.max_width > 0 {
            self.max_width
        } else {
            self.get_width()
        };
        let text_width = font.width_text(&message);
        let x = self.widget.get_x();
        let y = self.widget.get_y() + (self.get_height() - 9) / 2;
        if text_width > max_width {
            match self.text_overflow {
                TextOverflow::CLAMPED => {
                    output.accept_left(
                        x,
                        y,
                        &ComponentRenderUtils::clip_text(&message, font, max_width),
                    );
                }
                TextOverflow::SCROLLING => {
                    abstract_widget::extract_scrolling_string_over_contents(
                        self, output, &message, 2,
                    );
                }
            }
        } else {
            output.accept_left(x, y, &Language::get_instance().get_visual_order(&message));
        }
    }
}

impl AbstractWidgetClass for StringWidget {
    fn widget(&self) -> &AbstractWidget {
        &self.widget
    }

    fn widget_mut(&mut self) -> &mut AbstractWidget {
        &mut self.widget
    }

    /// `AbstractStringWidget.extractWidgetRenderState`.
    fn extract_widget_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
        let effects = abstract_string_widget::hovered_text_effects(self.is_hovered(), false);
        let mut output = graphics.text_renderer_for_widget(self.widget.alpha, effects);
        self.visit_lines(&mut output);
    }

    /// `getWidth`: the text's, up to the maximum, when there's one.
    fn get_width(&self) -> i32 {
        if self.max_width > 0 {
            match self.cached_width.get() {
                Some(width) => width,
                None => {
                    let width = self.max_width.min(self.font.width_sequence(
                        &Language::get_instance().get_visual_order(&self.widget.message),
                    ));
                    self.cached_width.set(Some(width));
                    width
                }
            }
        } else {
            self.widget.width
        }
    }

    /// `setMessage`: as wide as the new text (`AbstractStringWidget`'s), its cached width dirty.
    fn set_message(&mut self, message: Component) {
        let width = self
            .font
            .width_sequence(&Language::get_instance().get_visual_order(&message));
        self.widget.message = message;
        self.widget.width = width;
        self.cached_width.set(None);
    }
}
