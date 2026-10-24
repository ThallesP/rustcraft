//! net/minecraft/client/gui/screens/inventory/tooltip/BelowOrAboveWidgetTooltipPositioner.java

use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_positioner::ClientTooltipPositioner;

/// A focused widget's tooltip: under the widget, or over it when there's no room.
pub struct BelowOrAboveWidgetTooltipPositioner {
    screen_rectangle: ScreenRectangle,
}

impl BelowOrAboveWidgetTooltipPositioner {
    pub fn new(screen_rectangle: ScreenRectangle) -> BelowOrAboveWidgetTooltipPositioner {
        BelowOrAboveWidgetTooltipPositioner { screen_rectangle }
    }
}

impl ClientTooltipPositioner for BelowOrAboveWidgetTooltipPositioner {
    fn position_tooltip(
        &self,
        screen_width: i32,
        screen_height: i32,
        _x: i32,
        _y: i32,
        tooltip_width: i32,
        tooltip_height: i32,
    ) -> (i32, i32) {
        let mut result_x = self.screen_rectangle.left() + 3;
        let mut result_y = self.screen_rectangle.bottom() + 3 + 1;
        if result_y + tooltip_height + 3 > screen_height {
            result_y = self.screen_rectangle.top() - tooltip_height - 3 - 1;
        }
        if result_x + tooltip_width > screen_width {
            result_x = (self.screen_rectangle.right() - tooltip_width - 3).max(4);
        }
        (result_x, result_y)
    }
}
