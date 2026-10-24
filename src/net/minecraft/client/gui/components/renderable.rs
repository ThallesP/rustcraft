//! net/minecraft/client/gui/components/Renderable.java

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;

pub trait Renderable {
    fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    );
}

/// A renderable as a screen's list shares it.
pub type RenderableRef = Rc<RefCell<dyn Renderable>>;
