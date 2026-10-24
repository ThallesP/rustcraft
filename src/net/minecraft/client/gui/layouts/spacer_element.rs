//! net/minecraft/client/gui/layouts/SpacerElement.java

use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElement;

/// Empty space of a size, which has no widgets.
pub struct SpacerElement {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl SpacerElement {
    pub fn new(width: i32, height: i32) -> SpacerElement {
        SpacerElement::at(0, 0, width, height)
    }

    pub fn at(x: i32, y: i32, width: i32, height: i32) -> SpacerElement {
        SpacerElement {
            x,
            y,
            width,
            height,
        }
    }

    pub fn width(width: i32) -> SpacerElement {
        SpacerElement::new(width, 0)
    }

    pub fn height(height: i32) -> SpacerElement {
        SpacerElement::new(0, height)
    }
}

impl LayoutElement for SpacerElement {
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    fn get_x(&self) -> i32 {
        self.x
    }

    fn get_y(&self) -> i32 {
        self.y
    }

    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }
}
