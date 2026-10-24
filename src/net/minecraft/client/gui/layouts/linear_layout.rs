//! net/minecraft/client/gui/layouts/LinearLayout.java
//!
//! Children in a row or a column: a grid of one row or one column.

use crate::net::minecraft::client::gui::layouts::grid_layout::GridLayout;
use crate::net::minecraft::client::gui::layouts::layout::Layout;
use crate::net::minecraft::client::gui::layouts::layout_element::{
    LayoutElement, LayoutElementRef,
};
use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;

/// `LinearLayout.Orientation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    HORIZONTAL,
    VERTICAL,
}

pub struct LinearLayout {
    wrapped: GridLayout,
    orientation: Orientation,
    next_child_index: i32,
}

impl LinearLayout {
    pub fn at(x: i32, y: i32, orientation: Orientation) -> LinearLayout {
        LinearLayout {
            wrapped: GridLayout::at(x, y),
            orientation,
            next_child_index: 0,
        }
    }

    pub fn vertical() -> LinearLayout {
        LinearLayout::at(0, 0, Orientation::VERTICAL)
    }

    pub fn horizontal() -> LinearLayout {
        LinearLayout::at(0, 0, Orientation::HORIZONTAL)
    }

    pub fn spacing(mut self, spacing: i32) -> LinearLayout {
        match self.orientation {
            Orientation::HORIZONTAL => self.wrapped.column_spacing(spacing),
            Orientation::VERTICAL => self.wrapped.row_spacing(spacing),
        };
        self
    }

    pub fn new_cell_settings(&self) -> LayoutSettings {
        self.wrapped.new_cell_settings()
    }

    pub fn default_cell_setting(&self) -> LayoutSettings {
        self.wrapped.default_cell_setting()
    }

    /// `addChild(T, LayoutSettings)`.
    pub fn add_child_with(
        &mut self,
        child: LayoutElementRef,
        cell_settings: LayoutSettings,
    ) -> LayoutElementRef {
        let index = self.next_child_index;
        self.next_child_index += 1;
        match self.orientation {
            Orientation::HORIZONTAL => self.wrapped.add_child_with(child, 0, index, cell_settings),
            Orientation::VERTICAL => self.wrapped.add_child_with(child, index, 0, cell_settings),
        }
    }

    pub fn add_child(&mut self, child: LayoutElementRef) -> LayoutElementRef {
        let settings = self.new_cell_settings();
        self.add_child_with(child, settings)
    }
}

impl LayoutElement for LinearLayout {
    fn set_x(&mut self, x: i32) {
        self.wrapped.set_x(x);
    }

    fn set_y(&mut self, y: i32) {
        self.wrapped.set_y(y);
    }

    fn get_x(&self) -> i32 {
        self.wrapped.get_x()
    }

    fn get_y(&self) -> i32 {
        self.wrapped.get_y()
    }

    fn get_width(&self) -> i32 {
        self.wrapped.get_width()
    }

    fn get_height(&self) -> i32 {
        self.wrapped.get_height()
    }
}

impl Layout for LinearLayout {
    fn visit_children(&self, layout_element_visitor: &mut dyn FnMut(&LayoutElementRef)) {
        self.wrapped.visit_children(layout_element_visitor);
    }

    fn arrange_elements(&mut self) {
        self.wrapped.arrange_elements();
    }

    fn remove_children(&mut self) {
        self.wrapped.remove_children();
        self.next_child_index = 0;
    }
}
