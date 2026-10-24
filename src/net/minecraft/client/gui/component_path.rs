//! net/minecraft/client/gui/ComponentPath.java
//!
//! The way down from a container to the listener that has (or gets) the focus. Java's path holds
//! each container and the leaf; here the container a path is applied to is whoever applies it (a
//! screen, or a container a parent's path reached), so a step holds the child it goes into, and the
//! leaf is the last child reached.

use crate::net::minecraft::client::gui::components::events::gui_event_listener::{
    GuiEventListener, ListenerRef,
};

#[derive(Clone)]
pub enum ComponentPath {
    /// `ComponentPath.Leaf`: the listener the path was applied to.
    Leaf,
    /// `ComponentPath.Path`: the container's child, and the path in it.
    Path(ListenerRef, Box<ComponentPath>),
}

impl ComponentPath {
    pub fn leaf() -> ComponentPath {
        ComponentPath::Leaf
    }

    /// `path(container, childPath)`: through `child`, the container's child the path is in.
    pub fn path(child: ListenerRef, child_path: ComponentPath) -> ComponentPath {
        ComponentPath::Path(child, Box::new(child_path))
    }

    /// `applyFocus`, from `component`, the listener the path starts at.
    pub fn apply_focus(&self, component: &mut dyn GuiEventListener, focused: bool) {
        match self {
            ComponentPath::Leaf => component.set_focused(focused),
            ComponentPath::Path(child, child_path) => {
                let container = component
                    .as_container()
                    .expect("a path goes through a container");
                if focused {
                    container.set_focused_child(Some(child.clone()));
                } else {
                    container.set_focused_child(None);
                }
                child_path.apply_focus(&mut *child.borrow_mut(), focused);
            }
        }
    }

    /// `leafComponent()`: the last child the path goes into, or none for a leaf at its start.
    pub fn leaf_component(&self) -> Option<ListenerRef> {
        match self {
            ComponentPath::Leaf => None,
            ComponentPath::Path(child, child_path) => {
                child_path.leaf_component().or_else(|| Some(child.clone()))
            }
        }
    }
}
