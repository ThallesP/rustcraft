//! net/minecraft/client/gui/navigation/ScreenDirection.java
//!
//! `coordinateValueComparator()`, a fastutil `IntComparator`, is `compare_coordinates`.

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

    /// `coordinateValueComparator()`: coordinates in the direction's order.
    pub fn compare_coordinates(self, k1: i32, k2: i32) -> std::cmp::Ordering {
        if k1 == k2 {
            std::cmp::Ordering::Equal
        } else if self.is_before(k1, k2) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
