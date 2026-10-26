//! net/minecraft/client/gui/components/CommonButtons.java
//!
//! The friends, language and accessibility buttons. Their narration isn't ported.

use crate::net::minecraft::client::gui::components::button::{Button, OnPress};
use crate::net::minecraft::client::gui::components::friends_button::FriendsButton;
use crate::net::minecraft::client::gui::components::sprite_icon_button::SpriteIconButton;
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct CommonButtons;

impl CommonButtons {
    pub fn language(width: i32, on_press: OnPress, icon_only: bool) -> Button {
        let mut button = SpriteIconButton::builder(
            Component::translatable("options.language").into(),
            on_press,
            icon_only,
        )
        .width(width)
        .sprite(Identifier::with_default_namespace("icon/language"), 15, 15)
        .build();
        button.set_tooltip(Some(Tooltip::create(
            Component::translatable("options.language.tooltip").into(),
        )));
        button
    }

    pub fn accessibility(width: i32, on_press: OnPress, icon_only: bool) -> Button {
        let text = if icon_only {
            Component::translatable("options.accessibility")
        } else {
            Component::translatable("accessibility.onboarding.accessibility.button")
        };
        let mut button = SpriteIconButton::builder(text.into(), on_press, icon_only)
            .width(width)
            .sprite(
                Identifier::with_default_namespace("icon/accessibility"),
                15,
                15,
            )
            .build();
        button.set_tooltip(Some(Tooltip::create(
            Component::translatable("options.accessibility.tooltip").into(),
        )));
        button
    }

    pub fn friends(width: i32, on_press: OnPress, friends_available: bool) -> Button {
        FriendsButton::create(width, on_press, friends_available)
    }
}
