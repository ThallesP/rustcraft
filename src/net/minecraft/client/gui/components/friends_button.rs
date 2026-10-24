//! net/minecraft/client/gui/components/FriendsButton.java
//!
//! The pause menu's friends button: the friends icon, and the count of friend requests over its
//! corner. Java reads the count from the player social manager's friend list, which isn't ported:
//! the list is off (`isFriendListEnabled`), so the count is 0, which is what a game with the list
//! off shows.

use crate::net::minecraft::client::gui::components::abstract_widget::AbstractWidgetClass;
use crate::net::minecraft::client::gui::components::button::{Button, ButtonKind, OnPress};
use crate::net::minecraft::client::gui::components::sprite_icon_button::SpriteIconButton;
use crate::net::minecraft::client::gui::components::widget_sprites::WidgetSprites;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct FriendsButton {
    incoming_request_count: i32,
}

impl FriendsButton {
    const SPRITE_SIZE: i32 = 15;

    /// The constructor: a centered icon, active (with its tooltip) when friends are available.
    pub fn create(width: i32, on_press: OnPress, friends_available: bool) -> Button {
        let icon = SpriteIconButton::new(
            FriendsButton::SPRITE_SIZE,
            FriendsButton::SPRITE_SIZE,
            0,
            0,
            WidgetSprites::of(Identifier::with_default_namespace("friends/friends")),
            friends_available.then(|| Component::translatable("gui.friends.open.tooltip").into()),
            false,
        );
        let tooltip = icon.default_tooltip();
        let mut button = Button::new(
            0,
            0,
            width,
            20,
            Component::translatable("gui.friends.open").into(),
            on_press,
            ButtonKind::Friends(
                icon,
                FriendsButton {
                    incoming_request_count: 0,
                },
            ),
        );
        button.set_tooltip(tooltip);
        button.widget_mut().active = friends_available;
        button
    }

    /// `refreshIncomingRequestCount`: no requests while the friend list is off.
    pub fn refresh_incoming_request_count(&mut self) {
        self.incoming_request_count = 0;
    }

    /// `extractContents`: the centered icon, then the count's notification icon.
    pub fn extract_contents(button: &Button, graphics: &mut GuiGraphicsExtractor) {
        SpriteIconButton::extract_centered_icon(button, graphics);
        let ButtonKind::Friends(_, friends) = &button.kind else {
            return;
        };
        if button.is_active() && friends.incoming_request_count > 0 {
            let icons = [
                "notification/1",
                "notification/2",
                "notification/3",
                "notification/4",
                "notification/5",
                "notification/more",
            ];
            let icon_index = (friends.incoming_request_count.min(6) - 1) as usize;
            let widget = button.widget();
            graphics.blit_sprite_alpha(
                &RenderPipelines::get().GUI_TEXTURED,
                &Identifier::with_default_namespace(icons[icon_index]),
                widget.get_x() + button.get_width() - 5,
                widget.get_y() - 3,
                8,
                8,
                widget.alpha,
            );
        }
    }
}
