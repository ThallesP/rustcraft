//! net/minecraft/client/gui/screens/inventory/tooltip/ClientTooltipPositioner.java

/// Where a tooltip goes, from the mouse and its size. JOML's `Vector2ic` result is `(x, y)`.
pub trait ClientTooltipPositioner {
    fn position_tooltip(
        &self,
        screen_width: i32,
        screen_height: i32,
        x: i32,
        y: i32,
        tooltip_width: i32,
        tooltip_height: i32,
    ) -> (i32, i32);
}
