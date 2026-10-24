//! net/minecraft/client/gui/layouts/LayoutSettings.java
//!
//! A child's padding and alignment in its cell. Java's settings are an object a layout shares with
//! the children it adds with its defaults, so changing the defaults later changes theirs too: here
//! a `LayoutSettings` is a shared handle to the values (`LayoutSettingsImpl`), whose setters change
//! them in place and hand the handle back like Java's. Overloads by arity: `padding(int, int)` is
//! `padding_hv`, `padding(int, int, int, int)` `padding_ltrb`.

use std::cell::Cell;
use std::rc::Rc;

/// `LayoutSettings.LayoutSettingsImpl`'s values.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSettingsImpl {
    pub padding_left: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub x_alignment: f32,
    pub y_alignment: f32,
}

#[derive(Clone, Debug, Default)]
pub struct LayoutSettings(Rc<Cell<LayoutSettingsImpl>>);

impl LayoutSettings {
    /// `defaults()`.
    pub fn defaults() -> LayoutSettings {
        LayoutSettings::default()
    }

    fn update(&self, f: impl FnOnce(&mut LayoutSettingsImpl)) -> LayoutSettings {
        let mut values = self.0.get();
        f(&mut values);
        self.0.set(values);
        self.clone()
    }

    pub fn padding(&self, padding: i32) -> LayoutSettings {
        self.padding_hv(padding, padding)
    }

    pub fn padding_hv(&self, horizontal: i32, vertical: i32) -> LayoutSettings {
        self.padding_horizontal(horizontal)
            .padding_vertical(vertical)
    }

    pub fn padding_ltrb(&self, left: i32, top: i32, right: i32, bottom: i32) -> LayoutSettings {
        self.padding_left(left)
            .padding_right(right)
            .padding_top(top)
            .padding_bottom(bottom)
    }

    pub fn padding_left(&self, padding: i32) -> LayoutSettings {
        self.update(|values| values.padding_left = padding)
    }

    pub fn padding_top(&self, padding: i32) -> LayoutSettings {
        self.update(|values| values.padding_top = padding)
    }

    pub fn padding_right(&self, padding: i32) -> LayoutSettings {
        self.update(|values| values.padding_right = padding)
    }

    pub fn padding_bottom(&self, padding: i32) -> LayoutSettings {
        self.update(|values| values.padding_bottom = padding)
    }

    pub fn padding_horizontal(&self, padding: i32) -> LayoutSettings {
        self.padding_left(padding).padding_right(padding)
    }

    pub fn padding_vertical(&self, padding: i32) -> LayoutSettings {
        self.padding_top(padding).padding_bottom(padding)
    }

    pub fn align(&self, x_alignment: f32, y_alignment: f32) -> LayoutSettings {
        self.update(|values| {
            values.x_alignment = x_alignment;
            values.y_alignment = y_alignment;
        })
    }

    pub fn align_horizontally(&self, x_alignment: f32) -> LayoutSettings {
        self.update(|values| values.x_alignment = x_alignment)
    }

    pub fn align_vertically(&self, y_alignment: f32) -> LayoutSettings {
        self.update(|values| values.y_alignment = y_alignment)
    }

    pub fn align_horizontally_left(&self) -> LayoutSettings {
        self.align_horizontally(0.0)
    }

    pub fn align_horizontally_center(&self) -> LayoutSettings {
        self.align_horizontally(0.5)
    }

    pub fn align_horizontally_right(&self) -> LayoutSettings {
        self.align_horizontally(1.0)
    }

    pub fn align_vertically_top(&self) -> LayoutSettings {
        self.align_vertically(0.0)
    }

    pub fn align_vertically_middle(&self) -> LayoutSettings {
        self.align_vertically(0.5)
    }

    pub fn align_vertically_bottom(&self) -> LayoutSettings {
        self.align_vertically(1.0)
    }

    /// `copy()`: new settings with the same values.
    pub fn copy(&self) -> LayoutSettings {
        LayoutSettings(Rc::new(Cell::new(self.0.get())))
    }

    /// `getExposed()`: the values as they are now.
    pub fn get_exposed(&self) -> LayoutSettingsImpl {
        self.0.get()
    }
}
