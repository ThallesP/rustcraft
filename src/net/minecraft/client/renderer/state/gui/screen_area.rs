//! net/minecraft/client/renderer/state/gui/ScreenArea.java

use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;

pub trait ScreenArea {
    fn bounds(&self) -> Option<ScreenRectangle>;
}
