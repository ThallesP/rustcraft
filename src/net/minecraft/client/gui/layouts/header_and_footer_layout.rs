//! net/minecraft/client/gui/layouts/HeaderAndFooterLayout.java
//!
//! A screen's three frames: the header at the top, the footer at the bottom, both centering their
//! children across the screen, and the contents between them, 30 pixels under the header or as low
//! as fits above the footer. Java's layout holds its screen and reads its size; here the screen
//! tells the layout its size before arranging it (`set_screen_size`), the back-reference's part
//! the layout reads.

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::components::string_widget::StringWidget;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::layouts::frame_layout::FrameLayout;
use crate::net::minecraft::client::gui::layouts::layout::Layout;
use crate::net::minecraft::client::gui::layouts::layout_element::{
    LayoutElement, LayoutElementRef,
};
use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;
use crate::net::minecraft::network::chat::component::Component;

pub struct HeaderAndFooterLayout {
    header_frame: FrameLayout,
    footer_frame: FrameLayout,
    contents_frame: FrameLayout,
    /// The screen's `width` and `height`.
    screen_width: i32,
    screen_height: i32,
    header_height: i32,
    footer_height: i32,
}

impl HeaderAndFooterLayout {
    pub const MAGIC_PADDING: i32 = 13;
    pub const DEFAULT_HEADER_AND_FOOTER_HEIGHT: i32 = 33;
    const CONTENT_MARGIN_TOP: i32 = 30;

    /// `HeaderAndFooterLayout(Screen)`.
    pub fn new() -> HeaderAndFooterLayout {
        HeaderAndFooterLayout::with_heights(
            HeaderAndFooterLayout::DEFAULT_HEADER_AND_FOOTER_HEIGHT,
            HeaderAndFooterLayout::DEFAULT_HEADER_AND_FOOTER_HEIGHT,
        )
    }

    /// `HeaderAndFooterLayout(Screen, int, int)`.
    pub fn with_heights(header_height: i32, footer_height: i32) -> HeaderAndFooterLayout {
        let layout = HeaderAndFooterLayout {
            header_frame: FrameLayout::new(),
            footer_frame: FrameLayout::new(),
            contents_frame: FrameLayout::new(),
            screen_width: 0,
            screen_height: 0,
            header_height,
            footer_height,
        };
        layout
            .header_frame
            .default_child_layout_setting()
            .align(0.5, 0.5);
        layout
            .footer_frame
            .default_child_layout_setting()
            .align(0.5, 0.5);
        layout
    }

    /// The screen's size, which `arrangeElements` and the getters read.
    pub fn set_screen_size(&mut self, width: i32, height: i32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn get_footer_height(&self) -> i32 {
        self.footer_height
    }

    pub fn set_footer_height(&mut self, footer_height: i32) {
        self.footer_height = footer_height;
    }

    pub fn set_header_height(&mut self, header_height: i32) {
        self.header_height = header_height;
    }

    pub fn get_header_height(&self) -> i32 {
        self.header_height
    }

    pub fn get_content_height(&self) -> i32 {
        self.screen_height - self.get_header_height() - self.get_footer_height()
    }

    pub fn add_to_header(&mut self, child: LayoutElementRef) -> LayoutElementRef {
        self.header_frame.add_child(child)
    }

    /// `addToHeader(T, Consumer<LayoutSettings>)`: a copy of the header's defaults, adjusted.
    pub fn add_to_header_with(
        &mut self,
        child: LayoutElementRef,
        layout_settings_adjustments: impl FnOnce(&LayoutSettings),
    ) -> LayoutElementRef {
        let settings = self.header_frame.new_child_layout_settings();
        layout_settings_adjustments(&settings);
        self.header_frame.add_child_with(child, settings)
    }

    /// `addTitleHeader`.
    pub fn add_title_header(&mut self, component: Component, font: Rc<Font>) {
        let title: WidgetRef = Rc::new(RefCell::new(StringWidget::of(component, font)));
        self.header_frame.add_child(LayoutElementRef::widget(title));
    }

    pub fn add_to_footer(&mut self, child: LayoutElementRef) -> LayoutElementRef {
        self.footer_frame.add_child(child)
    }

    pub fn add_to_footer_with(
        &mut self,
        child: LayoutElementRef,
        layout_settings_adjustments: impl FnOnce(&LayoutSettings),
    ) -> LayoutElementRef {
        let settings = self.footer_frame.new_child_layout_settings();
        layout_settings_adjustments(&settings);
        self.footer_frame.add_child_with(child, settings)
    }

    pub fn add_to_contents(&mut self, child: LayoutElementRef) -> LayoutElementRef {
        self.contents_frame.add_child(child)
    }

    pub fn add_to_contents_with(
        &mut self,
        child: LayoutElementRef,
        layout_settings_adjustments: impl FnOnce(&LayoutSettings),
    ) -> LayoutElementRef {
        let settings = self.contents_frame.new_child_layout_settings();
        layout_settings_adjustments(&settings);
        self.contents_frame.add_child_with(child, settings)
    }
}

impl Default for HeaderAndFooterLayout {
    fn default() -> HeaderAndFooterLayout {
        HeaderAndFooterLayout::new()
    }
}

impl LayoutElement for HeaderAndFooterLayout {
    fn set_x(&mut self, _x: i32) {}

    fn set_y(&mut self, _y: i32) {}

    fn get_x(&self) -> i32 {
        0
    }

    fn get_y(&self) -> i32 {
        0
    }

    fn get_width(&self) -> i32 {
        self.screen_width
    }

    fn get_height(&self) -> i32 {
        self.screen_height
    }
}

impl Layout for HeaderAndFooterLayout {
    fn visit_children(&self, layout_element_visitor: &mut dyn FnMut(&LayoutElementRef)) {
        self.header_frame.visit_children(layout_element_visitor);
        self.contents_frame.visit_children(layout_element_visitor);
        self.footer_frame.visit_children(layout_element_visitor);
    }

    fn remove_children(&mut self) {
        self.header_frame.remove_children();
        self.contents_frame.remove_children();
        self.footer_frame.remove_children();
    }

    fn arrange_elements(&mut self) {
        let header_height = self.get_header_height();
        let footer_height = self.get_footer_height();
        self.header_frame.set_min_width(self.screen_width);
        self.header_frame.set_min_height(header_height);
        self.header_frame.set_position(0, 0);
        self.header_frame.arrange_elements();
        self.footer_frame.set_min_width(self.screen_width);
        self.footer_frame.set_min_height(footer_height);
        self.footer_frame.arrange_elements();
        self.footer_frame.set_y(self.screen_height - footer_height);
        self.contents_frame.set_min_width(self.screen_width);
        self.contents_frame.arrange_elements();
        let preferred_content_y = header_height + HeaderAndFooterLayout::CONTENT_MARGIN_TOP;
        let max_content_y = self.screen_height - footer_height - self.contents_frame.get_height();
        self.contents_frame
            .set_position(0, preferred_content_y.min(max_content_y));
    }
}
