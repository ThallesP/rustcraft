//! net/minecraft/client/gui/components/WidgetTooltipHolder.java
//!
//! A widget's tooltip, shown once it has been hovered (or focused from the keyboard) longer than
//! its delay, by the wall clock like Java's: with no delay, from the second frame on.

use std::time::Duration;

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::below_or_above_widget_tooltip_positioner::BelowOrAboveWidgetTooltipPositioner;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_positioner::ClientTooltipPositioner;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::menu_tooltip_positioner::MenuTooltipPositioner;
use crate::net::minecraft::client::input_type;
use crate::net::minecraft::util::util::Util;

#[derive(Default)]
pub struct WidgetTooltipHolder {
    tooltip: Option<Tooltip>,
    delay: Duration,
    display_start_time: i64,
    was_displayed: bool,
}

impl WidgetTooltipHolder {
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    pub fn set(&mut self, tooltip: Option<Tooltip>) {
        self.tooltip = tooltip;
    }

    pub fn get(&self) -> Option<&Tooltip> {
        self.tooltip.as_ref()
    }

    pub fn refresh_tooltip_for_next_render_pass(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        is_hovered: bool,
        is_focused: bool,
        screen_rectangle: ScreenRectangle,
    ) {
        let Some(tooltip) = self.tooltip.as_mut() else {
            self.was_displayed = false;
            return;
        };
        let should_display =
            is_hovered || (is_focused && input_type::last_input_type().is_keyboard());
        if should_display != self.was_displayed {
            if should_display {
                self.display_start_time = Util::get_millis();
            }
            self.was_displayed = should_display;
        }
        if should_display
            && Util::get_millis() - self.display_start_time > self.delay.as_millis() as i64
        {
            let font = std::rc::Rc::clone(graphics.font());
            let lines = tooltip.to_char_sequence(&font);
            let style = tooltip.style().cloned();
            graphics.set_tooltip_for_next_frame_positioned(
                &font,
                lines,
                WidgetTooltipHolder::create_tooltip_positioner(
                    screen_rectangle,
                    is_hovered,
                    is_focused,
                ),
                mouse_x,
                mouse_y,
                is_focused,
                style,
            );
        }
    }

    /// Below or above the widget focused from the keyboard, else by the mouse.
    fn create_tooltip_positioner(
        screen_rectangle: ScreenRectangle,
        is_hovered: bool,
        is_focused: bool,
    ) -> Box<dyn ClientTooltipPositioner> {
        if !is_hovered && is_focused && input_type::last_input_type().is_keyboard() {
            Box::new(BelowOrAboveWidgetTooltipPositioner::new(screen_rectangle))
        } else {
            Box::new(MenuTooltipPositioner::new(screen_rectangle))
        }
    }
}
