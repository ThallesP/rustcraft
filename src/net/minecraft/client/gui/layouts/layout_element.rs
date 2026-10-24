//! net/minecraft/client/gui/layouts/LayoutElement.java
//!
//! Something a layout places: a widget, another layout or a spacer. Layouts hold their children
//! by reference like Java's, and `visitWidgets` hands a screen the widgets themselves, so a child
//! is a `LayoutElementRef`: the shared widget, layout or other element.

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::layouts::layout::Layout;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;

pub trait LayoutElement {
    fn set_x(&mut self, x: i32);

    fn set_y(&mut self, y: i32);

    fn get_x(&self) -> i32;

    fn get_y(&self) -> i32;

    fn get_width(&self) -> i32;

    fn get_height(&self) -> i32;

    fn get_layout_rectangle(&self) -> ScreenRectangle {
        ScreenRectangle::new(
            self.get_x(),
            self.get_y(),
            self.get_width(),
            self.get_height(),
        )
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.set_x(x);
        self.set_y(y);
    }
}

/// A layout's child, as Java's reference to it.
#[derive(Clone)]
pub enum LayoutElementRef {
    Widget(WidgetRef),
    Layout(Rc<RefCell<dyn Layout>>),
    Element(Rc<RefCell<dyn LayoutElement>>),
}

impl LayoutElementRef {
    pub fn widget(widget: WidgetRef) -> LayoutElementRef {
        LayoutElementRef::Widget(widget)
    }

    pub fn layout(layout: Rc<RefCell<dyn Layout>>) -> LayoutElementRef {
        LayoutElementRef::Layout(layout)
    }

    fn with<R>(&self, f: impl FnOnce(&dyn LayoutElement) -> R) -> R {
        match self {
            LayoutElementRef::Widget(widget) => f(&*widget.borrow()),
            LayoutElementRef::Layout(layout) => f(&*layout.borrow()),
            LayoutElementRef::Element(element) => f(&*element.borrow()),
        }
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut dyn LayoutElement) -> R) -> R {
        match self {
            LayoutElementRef::Widget(widget) => f(&mut *widget.borrow_mut()),
            LayoutElementRef::Layout(layout) => f(&mut *layout.borrow_mut()),
            LayoutElementRef::Element(element) => f(&mut *element.borrow_mut()),
        }
    }

    pub fn set_x(&self, x: i32) {
        self.with_mut(|element| element.set_x(x));
    }

    pub fn set_y(&self, y: i32) {
        self.with_mut(|element| element.set_y(y));
    }

    pub fn get_x(&self) -> i32 {
        self.with(|element| element.get_x())
    }

    pub fn get_y(&self) -> i32 {
        self.with(|element| element.get_y())
    }

    pub fn get_width(&self) -> i32 {
        self.with(|element| element.get_width())
    }

    pub fn get_height(&self) -> i32 {
        self.with(|element| element.get_height())
    }

    pub fn get_rectangle(&self) -> ScreenRectangle {
        self.with(|element| element.get_layout_rectangle())
    }

    pub fn set_position(&self, x: i32, y: i32) {
        self.with_mut(|element| element.set_position(x, y));
    }

    /// `visitWidgets`: the widget itself, a layout's widgets, a spacer's none.
    pub fn visit_widgets(&self, widget_visitor: &mut dyn FnMut(&WidgetRef)) {
        match self {
            LayoutElementRef::Widget(widget) => widget_visitor(widget),
            LayoutElementRef::Layout(layout) => layout.borrow().visit_widgets(widget_visitor),
            LayoutElementRef::Element(_) => {}
        }
    }

    /// `instanceof Layout`.
    pub fn as_layout(&self) -> Option<&Rc<RefCell<dyn Layout>>> {
        match self {
            LayoutElementRef::Layout(layout) => Some(layout),
            _ => None,
        }
    }
}
