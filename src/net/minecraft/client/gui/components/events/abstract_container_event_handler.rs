//! net/minecraft/client/gui/components/events/AbstractContainerEventHandler.java
//!
//! The focused child and the dragging flag, which a container holds and its `ContainerEventHandler`
//! methods hand out.

use std::rc::Rc;

use crate::net::minecraft::client::gui::components::events::gui_event_listener::ListenerRef;

#[derive(Default)]
pub struct AbstractContainerEventHandler {
    focused: Option<ListenerRef>,
    is_dragging: bool,
}

impl AbstractContainerEventHandler {
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    pub fn get_focused(&self) -> Option<ListenerRef> {
        self.focused.clone()
    }

    /// `setFocused(GuiEventListener)`: the old child loses the focus and the new one gets it.
    pub fn set_focused(&mut self, focused: Option<ListenerRef>) {
        let same = match (&self.focused, &focused) {
            (Some(old), Some(new)) => Rc::ptr_eq(old, new),
            (None, None) => true,
            _ => false,
        };
        if !same {
            if let Some(old) = &self.focused {
                old.borrow_mut().set_focused(false);
            }
            if let Some(new) = &focused {
                new.borrow_mut().set_focused(true);
            }
            self.focused = focused;
        }
    }
}
