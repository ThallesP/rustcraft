//! net/minecraft/client/gui/layouts/AbstractLayout.java
//!
//! A layout's place and size, and `AbstractChildWrapper`, a child with its settings, which places
//! it in the space its cell has.

use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElementRef;
use crate::net::minecraft::client::gui::layouts::layout_settings::LayoutSettings;
use crate::net::minecraft::util::mth::Mth;

#[derive(Clone, Copy, Debug, Default)]
pub struct AbstractLayout {
    x: i32,
    y: i32,
    pub width: i32,
    pub height: i32,
}

impl AbstractLayout {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> AbstractLayout {
        AbstractLayout {
            x,
            y,
            width,
            height,
        }
    }

    /// `setX`: the children move with the layout.
    pub fn set_x(&mut self, x: i32, children: impl IntoIterator<Item = LayoutElementRef>) {
        for child in children {
            let new_child_x = child.get_x() + (x - self.x);
            child.set_x(new_child_x);
        }
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32, children: impl IntoIterator<Item = LayoutElementRef>) {
        for child in children {
            let new_child_y = child.get_y() + (y - self.y);
            child.set_y(new_child_y);
        }
        self.y = y;
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

/// `AbstractLayout.AbstractChildWrapper`: the child and its settings (`getExposed()` of what it was
/// added with, which a layout's later changes to its defaults reach).
#[derive(Clone)]
pub struct AbstractChildWrapper {
    pub child: LayoutElementRef,
    pub layout_settings: LayoutSettings,
}

impl AbstractChildWrapper {
    pub fn new(child: LayoutElementRef, layout_settings: LayoutSettings) -> AbstractChildWrapper {
        AbstractChildWrapper {
            child,
            layout_settings,
        }
    }

    pub fn get_height(&self) -> i32 {
        let settings = self.layout_settings.get_exposed();
        self.child.get_height() + settings.padding_top + settings.padding_bottom
    }

    pub fn get_width(&self) -> i32 {
        let settings = self.layout_settings.get_exposed();
        self.child.get_width() + settings.padding_left + settings.padding_right
    }

    pub fn set_x(&self, x: i32, available_space: i32) {
        let settings = self.layout_settings.get_exposed();
        let least_offset = settings.padding_left as f32;
        let most_offset =
            (available_space - self.child.get_width() - settings.padding_right) as f32;
        let offset = Mth::lerp_f32(settings.x_alignment, least_offset, most_offset) as i32;
        self.child.set_x(offset + x);
    }

    pub fn set_y(&self, y: i32, available_space: i32) {
        let settings = self.layout_settings.get_exposed();
        let least_offset = settings.padding_top as f32;
        let most_offset =
            (available_space - self.child.get_height() - settings.padding_bottom) as f32;
        let offset = java_math_round(Mth::lerp_f32(
            settings.y_alignment,
            least_offset,
            most_offset,
        ));
        self.child.set_y(offset + y);
    }
}

/// `Math.round(float)`: the nearest int, halves up, NaN 0, saturating.
pub fn java_math_round(value: f32) -> i32 {
    crate::java::lang::math::Math::round_f32(value)
}
