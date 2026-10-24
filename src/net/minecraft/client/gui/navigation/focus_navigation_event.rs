//! net/minecraft/client/gui/navigation/FocusNavigationEvent.java
//!
//! The interface and its three classes are an enum of them.

use crate::net::minecraft::client::gui::navigation::screen_axis::ScreenAxis;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusNavigationEvent {
    /// `ArrowNavigation(direction, previousFocus)`.
    ArrowNavigation(ScreenDirection, Option<ScreenRectangle>),
    InitialFocus,
    /// `TabNavigation(forward)`.
    TabNavigation(bool),
}

impl FocusNavigationEvent {
    pub fn arrow(direction: ScreenDirection) -> FocusNavigationEvent {
        FocusNavigationEvent::ArrowNavigation(direction, None)
    }

    pub fn get_vertical_direction_for_initial_focus(&self) -> ScreenDirection {
        match *self {
            FocusNavigationEvent::ArrowNavigation(direction, _) => {
                if direction.get_axis() == ScreenAxis::VERTICAL {
                    direction
                } else {
                    ScreenDirection::DOWN
                }
            }
            FocusNavigationEvent::InitialFocus => ScreenDirection::DOWN,
            FocusNavigationEvent::TabNavigation(forward) => {
                if forward {
                    ScreenDirection::DOWN
                } else {
                    ScreenDirection::UP
                }
            }
        }
    }
}
