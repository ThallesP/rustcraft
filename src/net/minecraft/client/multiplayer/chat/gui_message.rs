//! net/minecraft/client/multiplayer/chat/GuiMessage.java
//!
//! A chat message as the chat keeps it: the GUI tick it came at, its text, its source and tag. The
//! message's signature (for deleting it) isn't ported: signed chat isn't.

use std::rc::Rc;

use crate::net::minecraft::client::gui::components::component_render_utils::ComponentRenderUtils;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::multiplayer::chat::gui_message_source::GuiMessageSource;
use crate::net::minecraft::client::multiplayer::chat::gui_message_tag::GuiMessageTag;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

#[derive(Clone, Debug)]
pub struct GuiMessage {
    pub added_time: i32,
    pub content: Component,
    pub source: GuiMessageSource,
    pub tag: Option<GuiMessageTag>,
}

impl GuiMessage {
    const MESSAGE_TAG_MARGIN_LEFT: i32 = 4;

    /// `splitLines`: the message's lines at the width, less its tag's icon's when it has one.
    pub fn split_lines(
        &self,
        font: &Font,
        mut max_width: i32,
        chat_colors: bool,
    ) -> Vec<FormattedCharSequence> {
        if let Some(icon) = self.tag.as_ref().and_then(|tag| tag.icon) {
            max_width -= icon.width() + GuiMessage::MESSAGE_TAG_MARGIN_LEFT + 2;
        }
        ComponentRenderUtils::wrap_components(&self.content, max_width, font, chat_colors)
    }
}

/// `GuiMessage.Line`: one line of a message, and whether it's the message's last.
#[derive(Clone, Debug)]
pub struct Line {
    pub parent: Rc<GuiMessage>,
    pub content: FormattedCharSequence,
    pub end_of_entry: bool,
}

impl Line {
    pub fn get_tag_icon_left(&self, font: &Font) -> i32 {
        font.width_sequence(&self.content) + 4
    }

    pub fn tag(&self) -> Option<&GuiMessageTag> {
        self.parent.tag.as_ref()
    }

    pub fn added_time(&self) -> i32 {
        self.parent.added_time
    }
}
