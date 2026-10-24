//! net/minecraft/client/gui/layouts/Layout.java

use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::layouts::layout_element::{
    LayoutElement, LayoutElementRef,
};

pub trait Layout: LayoutElement {
    fn visit_children(&self, layout_element_visitor: &mut dyn FnMut(&LayoutElementRef));

    fn visit_widgets(&self, widget_visitor: &mut dyn FnMut(&WidgetRef)) {
        self.visit_children(&mut |child| child.visit_widgets(widget_visitor));
    }

    /// `arrangeElements`: the children that are layouts arrange theirs.
    fn arrange_elements(&mut self) {
        arrange_children(self);
    }

    fn remove_children(&mut self);
}

/// `Layout.super.arrangeElements()`, which a layout's own calls first.
pub fn arrange_children<L: Layout + ?Sized>(layout: &L) {
    layout.visit_children(&mut |child| {
        if let Some(layout) = child.as_layout() {
            layout.borrow_mut().arrange_elements();
        }
    });
}
