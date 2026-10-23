//! net/minecraft/client/gui/navigation/ScreenAxis.java

use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScreenAxis {
    HORIZONTAL,
    VERTICAL,
}

impl ScreenAxis {
    pub const VALUES: [ScreenAxis; 2] = [ScreenAxis::HORIZONTAL, ScreenAxis::VERTICAL];

    pub fn orthogonal(self) -> ScreenAxis {
        match self {
            ScreenAxis::HORIZONTAL => ScreenAxis::VERTICAL,
            ScreenAxis::VERTICAL => ScreenAxis::HORIZONTAL,
        }
    }

    pub fn get_positive(self) -> ScreenDirection {
        match self {
            ScreenAxis::HORIZONTAL => ScreenDirection::RIGHT,
            ScreenAxis::VERTICAL => ScreenDirection::DOWN,
        }
    }

    pub fn get_negative(self) -> ScreenDirection {
        match self {
            ScreenAxis::HORIZONTAL => ScreenDirection::LEFT,
            ScreenAxis::VERTICAL => ScreenDirection::UP,
        }
    }

    pub fn get_direction(self, positive: bool) -> ScreenDirection {
        if positive {
            self.get_positive()
        } else {
            self.get_negative()
        }
    }
}
