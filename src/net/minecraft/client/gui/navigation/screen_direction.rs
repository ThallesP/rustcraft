//! net/minecraft/client/gui/navigation/ScreenDirection.java
//!
//! `coordinateValueComparator()`, a fastutil `IntComparator`, comes with focus navigation.

use crate::net::minecraft::client::gui::navigation::screen_axis::ScreenAxis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScreenDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl ScreenDirection {
    pub const VALUES: [ScreenDirection; 4] = [
        ScreenDirection::UP,
        ScreenDirection::DOWN,
        ScreenDirection::LEFT,
        ScreenDirection::RIGHT,
    ];

    pub fn get_axis(self) -> ScreenAxis {
        match self {
            ScreenDirection::UP | ScreenDirection::DOWN => ScreenAxis::VERTICAL,
            ScreenDirection::LEFT | ScreenDirection::RIGHT => ScreenAxis::HORIZONTAL,
        }
    }

    pub fn get_opposite(self) -> ScreenDirection {
        match self {
            ScreenDirection::UP => ScreenDirection::DOWN,
            ScreenDirection::DOWN => ScreenDirection::UP,
            ScreenDirection::LEFT => ScreenDirection::RIGHT,
            ScreenDirection::RIGHT => ScreenDirection::LEFT,
        }
    }

    pub fn is_positive(self) -> bool {
        match self {
            ScreenDirection::UP | ScreenDirection::LEFT => false,
            ScreenDirection::DOWN | ScreenDirection::RIGHT => true,
        }
    }

    pub fn is_after(self, a: i32, b: i32) -> bool {
        if self.is_positive() { a > b } else { b > a }
    }

    pub fn is_before(self, a: i32, b: i32) -> bool {
        if self.is_positive() { a < b } else { b < a }
    }
}
