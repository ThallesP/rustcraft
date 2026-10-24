//! net/minecraft/client/gui/layouts/FrameLayout.java
//!
//! Children over one another in one frame, as big as the largest of them or its minimum size, each
//! aligned in it (centered by default); and `alignInRectangle`, which places an element in a
//! rectangle.

use crate::net::minecraft::client::gui::layouts::abstract_layout::{
    AbstractChildWrapper, AbstractLayout,
};
use crate::net::minecraft::client::gui::layouts::layout::{self, Layout};
use crate::net::minecraft::client::gui::layouts::layout_element::{
    LayoutElement, LayoutElementRef,
};
use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::util::mth::Mth;

pub struct FrameLayout {
    base: AbstractLayout,
    children: Vec<AbstractChildWrapper>,
    min_width: i32,
    min_height: i32,
    default_child_layout_settings: LayoutSettings,
}

impl Default for FrameLayout {
    fn default() -> FrameLayout {
        FrameLayout::new()
    }
}

impl FrameLayout {
    pub fn new() -> FrameLayout {
        FrameLayout::at(0, 0, 0, 0)
    }

    pub fn with_min(min_width: i32, min_height: i32) -> FrameLayout {
        FrameLayout::at(0, 0, min_width, min_height)
    }

    pub fn at(x: i32, y: i32, min_width: i32, min_height: i32) -> FrameLayout {
        FrameLayout {
            base: AbstractLayout::new(x, y, min_width, min_height),
            children: Vec::new(),
            min_width,
            min_height,
            default_child_layout_settings: LayoutSettings::defaults().align(0.5, 0.5),
        }
    }

    pub fn set_min_dimensions(&mut self, min_width: i32, min_height: i32) -> &mut FrameLayout {
        self.set_min_width(min_width).set_min_height(min_height)
    }

    pub fn set_min_height(&mut self, min_height: i32) -> &mut FrameLayout {
        self.min_height = min_height;
        self
    }

    pub fn set_min_width(&mut self, min_width: i32) -> &mut FrameLayout {
        self.min_width = min_width;
        self
    }

    pub fn new_child_layout_settings(&self) -> LayoutSettings {
        self.default_child_layout_settings.copy()
    }

    pub fn default_child_layout_setting(&self) -> LayoutSettings {
        self.default_child_layout_settings.clone()
    }

    pub fn add_child(&mut self, child: LayoutElementRef) -> LayoutElementRef {
        let settings = self.new_child_layout_settings();
        self.add_child_with(child, settings)
    }

    pub fn add_child_with(
        &mut self,
        child: LayoutElementRef,
        child_layout_settings: LayoutSettings,
    ) -> LayoutElementRef {
        self.children.push(AbstractChildWrapper::new(
            child.clone(),
            child_layout_settings,
        ));
        child
    }

    pub fn center_in_rectangle(widget: &LayoutElementRef, x: i32, y: i32, width: i32, height: i32) {
        FrameLayout::align_in_rectangle(widget, x, y, width, height, 0.5, 0.5);
    }

    /// `centerInRectangle(LayoutElement, ScreenRectangle)`.
    pub fn center_in_screen_rectangle(widget: &LayoutElementRef, rectangle: &ScreenRectangle) {
        FrameLayout::center_in_rectangle(
            widget,
            rectangle.position.x,
            rectangle.position.y,
            rectangle.width,
            rectangle.height,
        );
    }

    /// `alignInRectangle(LayoutElement, ScreenRectangle, float, float)`.
    pub fn align_in_screen_rectangle(
        widget: &LayoutElementRef,
        rectangle: &ScreenRectangle,
        align_x: f32,
        align_y: f32,
    ) {
        FrameLayout::align_in_rectangle(
            widget,
            rectangle.left(),
            rectangle.top(),
            rectangle.width,
            rectangle.height,
            align_x,
            align_y,
        );
    }

    pub fn align_in_rectangle(
        widget: &LayoutElementRef,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        align_x: f32,
        align_y: f32,
    ) {
        FrameLayout::align_in_dimension(
            x,
            width,
            widget.get_width(),
            &mut |pos| widget.set_x(pos),
            align_x,
        );
        FrameLayout::align_in_dimension(
            y,
            height,
            widget.get_height(),
            &mut |pos| widget.set_y(pos),
            align_y,
        );
    }

    pub fn align_in_dimension(
        pos: i32,
        length: i32,
        widget_length: i32,
        set_widget_pos: &mut dyn FnMut(i32),
        align: f32,
    ) {
        let offset = Mth::lerp_f32(align, 0.0, (length - widget_length) as f32) as i32;
        set_widget_pos(pos + offset);
    }

    fn child_refs(&self) -> Vec<LayoutElementRef> {
        self.children
            .iter()
            .map(|child| child.child.clone())
            .collect()
    }
}

impl LayoutElement for FrameLayout {
    fn set_x(&mut self, x: i32) {
        let children = self.child_refs();
        self.base.set_x(x, children);
    }

    fn set_y(&mut self, y: i32) {
        let children = self.child_refs();
        self.base.set_y(y, children);
    }

    fn get_x(&self) -> i32 {
        self.base.get_x()
    }

    fn get_y(&self) -> i32 {
        self.base.get_y()
    }

    fn get_width(&self) -> i32 {
        self.base.width
    }

    fn get_height(&self) -> i32 {
        self.base.height
    }
}

impl Layout for FrameLayout {
    fn visit_children(&self, layout_element_visitor: &mut dyn FnMut(&LayoutElementRef)) {
        for child in &self.children {
            layout_element_visitor(&child.child);
        }
    }

    fn arrange_elements(&mut self) {
        layout::arrange_children(self);
        let mut result_width = self.min_width;
        let mut result_height = self.min_height;
        for child in &self.children {
            result_width = result_width.max(child.get_width());
            result_height = result_height.max(child.get_height());
        }
        for child in &self.children {
            child.set_x(self.base.get_x(), result_width);
            child.set_y(self.base.get_y(), result_height);
        }
        self.base.width = result_width;
        self.base.height = result_height;
    }

    fn remove_children(&mut self) {
        self.children.clear();
    }
}
