//! net/minecraft/network/chat/CommonComponents.java
//!
//! The components screens share. Java's are constants; a `Component` is made at run time here,
//! so they're functions. Only the ones the ported screens use are here.

use crate::net::minecraft::network::chat::component::Component;

pub struct CommonComponents;

impl CommonComponents {
    /// `GUI_DONE`.
    pub fn gui_done() -> Component {
        Component::translatable("gui.done").into()
    }

    /// `EMPTY`.
    pub fn empty() -> Component {
        Component::empty().into()
    }
}
