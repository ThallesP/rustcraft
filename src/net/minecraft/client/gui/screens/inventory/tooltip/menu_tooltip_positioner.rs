//! net/minecraft/client/gui/screens/inventory/tooltip/MenuTooltipPositioner.java

use crate::java::lang::math::Math;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_positioner::ClientTooltipPositioner;
use crate::net::minecraft::util::mth::Mth;

/// A widget's tooltip: right of the mouse, below the widget or above it when there's no room.
pub struct MenuTooltipPositioner {
    screen_rectangle: ScreenRectangle,
}

impl MenuTooltipPositioner {
    pub const MAX_OVERLAP_WITH_WIDGET: i32 = 3;
    pub const MAX_DISTANCE_TO_WIDGET: i32 = 5;

    pub fn new(screen_rectangle: ScreenRectangle) -> MenuTooltipPositioner {
        MenuTooltipPositioner { screen_rectangle }
    }

    fn get_offset(mouse_y: i32, widget_y: i32, widget_height: i32) -> i32 {
        let distance = Math::abs_i32(mouse_y - widget_y).min(widget_height);
        Math::round_f32(Mth::lerp_f32(
            distance as f32 / widget_height as f32,
            (widget_height - 3) as f32,
            5.0,
        ))
    }
}

impl ClientTooltipPositioner for MenuTooltipPositioner {
    fn position_tooltip(
        &self,
        screen_width: i32,
        screen_height: i32,
        x: i32,
        y: i32,
        tooltip_width: i32,
        tooltip_height: i32,
    ) -> (i32, i32) {
        let (mut result_x, mut result_y) = (x + 12, y);
        if result_x + tooltip_width > screen_width - 5 {
            result_x = (x - 12 - tooltip_width).max(9);
        }
        result_y += 3;
        let padded_height = tooltip_height + 3 + 3;
        let lowest_possible_y = self.screen_rectangle.bottom()
            + 3
            + MenuTooltipPositioner::get_offset(0, 0, self.screen_rectangle.height);
        let max_y = screen_height - 5;
        if lowest_possible_y + padded_height <= max_y {
            result_y += MenuTooltipPositioner::get_offset(
                result_y,
                self.screen_rectangle.top(),
                self.screen_rectangle.height,
            );
        } else {
            result_y -= padded_height
                + MenuTooltipPositioner::get_offset(
                    result_y,
                    self.screen_rectangle.bottom(),
                    self.screen_rectangle.height,
                );
        }
        (result_x, result_y)
    }
}
