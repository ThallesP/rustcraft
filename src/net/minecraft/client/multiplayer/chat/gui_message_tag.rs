//! net/minecraft/client/multiplayer/chat/GuiMessageTag.java
//!
//! What marks a chat message at its left: the indicator's color, an icon, the text its tooltip
//! shows and the log's tag. The constants are functions, which make the same records.

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Clone, Debug, PartialEq)]
pub struct GuiMessageTag {
    pub indicator_color: i32,
    pub icon: Option<Icon>,
    pub text: Option<Component>,
    pub log_tag: Option<String>,
}

impl GuiMessageTag {
    const CHAT_NOT_SECURE_INDICATOR_COLOR: i32 = 13684944;
    const CHAT_MODIFIED_INDICATOR_COLOR: i32 = 6316128;

    fn new(indicator_color: i32, icon: Option<Icon>, text: &str, log_tag: &str) -> GuiMessageTag {
        GuiMessageTag {
            indicator_color,
            icon,
            text: Some(Component::translatable(text).into()),
            log_tag: Some(log_tag.to_string()),
        }
    }

    pub fn system() -> GuiMessageTag {
        GuiMessageTag::new(13684944, None, "chat.tag.system", "System")
    }

    pub fn system_single_player() -> GuiMessageTag {
        GuiMessageTag::new(13684944, None, "chat.tag.system_single_player", "System")
    }

    pub fn chat_not_secure() -> GuiMessageTag {
        GuiMessageTag::new(
            GuiMessageTag::CHAT_NOT_SECURE_INDICATOR_COLOR,
            None,
            "chat.tag.not_secure",
            "Not Secure",
        )
    }

    pub fn chat_error() -> GuiMessageTag {
        GuiMessageTag::new(16733525, None, "chat.tag.error", "Chat Error")
    }

    /// `chatModified`, whose text holds the original message (in grey) isn't ported: its color and
    /// icon.
    pub fn chat_modified_indicator_color() -> i32 {
        GuiMessageTag::CHAT_MODIFIED_INDICATOR_COLOR
    }
}

/// `GuiMessageTag.Icon`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    CHAT_MODIFIED,
}

impl Icon {
    pub fn sprite(self) -> Identifier {
        Identifier::with_default_namespace("icon/chat_modified")
    }

    pub fn width(self) -> i32 {
        9
    }

    pub fn height(self) -> i32 {
        9
    }

    pub fn extract_render_state(self, graphics: &mut GuiGraphicsExtractor, x: i32, y: i32) {
        graphics.blit_sprite(
            &RenderPipelines::get().GUI_TEXTURED,
            &self.sprite(),
            x,
            y,
            self.width(),
            self.height(),
        );
    }
}
