//! net/minecraft/client/gui/components/CommonButtons.java
//!
//! Only the friends button; the language and accessibility buttons come with the title screen's
//! options.

use crate::net::minecraft::client::gui::components::button::{Button, OnPress};
use crate::net::minecraft::client::gui::components::friends_button::FriendsButton;

pub struct CommonButtons;

impl CommonButtons {
    pub fn friends(width: i32, on_press: OnPress, friends_available: bool) -> Button {
        FriendsButton::create(width, on_press, friends_available)
    }
}
