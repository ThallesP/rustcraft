//! net/minecraft/client/gui/screens/inventory/tooltip/DefaultTooltipPositioner.java

use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_positioner::ClientTooltipPositioner;

/// Up and to the right of the mouse, kept on the screen. Java's `INSTANCE` is the unit struct.
pub struct DefaultTooltipPositioner;

impl ClientTooltipPositioner for DefaultTooltipPositioner {
    fn position_tooltip(
        &self,
        screen_width: i32,
        screen_height: i32,
        x: i32,
        y: i32,
        tooltip_width: i32,
        tooltip_height: i32,
    ) -> (i32, i32) {
        let (mut result_x, mut result_y) = (x + 12, y - 12);
        if result_x + tooltip_width > screen_width {
            result_x = (result_x - 24 - tooltip_width).max(4);
        }
        let padded_height = tooltip_height + 3;
        if result_y + padded_height > screen_height {
            result_y = screen_height - padded_height;
        }
        (result_x, result_y)
    }
}
