//! net/minecraft/client/gui/navigation/ScreenPosition.java

use crate::net::minecraft::client::gui::navigation::screen_axis::ScreenAxis;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}

impl ScreenPosition {
    pub fn new(x: i32, y: i32) -> ScreenPosition {
        ScreenPosition { x, y }
    }

    pub fn of(axis: ScreenAxis, primary_value: i32, secondary_value: i32) -> ScreenPosition {
        match axis {
            ScreenAxis::HORIZONTAL => ScreenPosition::new(primary_value, secondary_value),
            ScreenAxis::VERTICAL => ScreenPosition::new(secondary_value, primary_value),
        }
    }

    pub fn step(&self, direction: ScreenDirection) -> ScreenPosition {
        match direction {
            ScreenDirection::DOWN => ScreenPosition::new(self.x, self.y.wrapping_add(1)),
            ScreenDirection::UP => ScreenPosition::new(self.x, self.y.wrapping_sub(1)),
            ScreenDirection::LEFT => ScreenPosition::new(self.x.wrapping_sub(1), self.y),
            ScreenDirection::RIGHT => ScreenPosition::new(self.x.wrapping_add(1), self.y),
        }
    }

    pub fn get_coordinate(&self, axis: ScreenAxis) -> i32 {
        match axis {
            ScreenAxis::HORIZONTAL => self.x,
            ScreenAxis::VERTICAL => self.y,
        }
    }
}
