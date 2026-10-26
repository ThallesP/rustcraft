//! net/minecraft/client/gui/components/events/GuiEventListener.java
//!
//! What a screen's children take of the input. Listeners are shared by reference in Java (a
//! screen's children, renderables and focus hold the same widget), so here they're
//! `Rc<RefCell<…>>`s, compared with `Rc::ptr_eq`. A container (`ContainerEventHandler`) answers
//! `as_container`, Java's `instanceof`. Not ported yet: the input method's preedit
//! (`preeditUpdated`).

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::component_path::ComponentPath;
use crate::net::minecraft::client::gui::components::events::container_event_handler::ContainerEventHandler;
use crate::net::minecraft::client::gui::components::tab_ordered_element::TabOrderedElement;
use crate::net::minecraft::client::gui::navigation::focus_navigation_event::FocusNavigationEvent;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;

/// A listener as its holders share it.
pub type ListenerRef = Rc<RefCell<dyn GuiEventListener>>;

pub trait GuiEventListener: TabOrderedElement {
    fn mouse_moved(&mut self, _x: f64, _y: f64) {}

    fn mouse_clicked(&mut self, _event: &MouseButtonEvent, _double_click: bool) -> bool {
        false
    }

    fn mouse_released(&mut self, _event: &MouseButtonEvent) -> bool {
        false
    }

    fn mouse_dragged(&mut self, _event: &MouseButtonEvent, _dx: f64, _dy: f64) -> bool {
        false
    }

    fn mouse_scrolled(&mut self, _x: f64, _y: f64, _scroll_x: f64, _scroll_y: f64) -> bool {
        false
    }

    fn key_pressed(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn key_released(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn char_typed(&mut self, _event: &CharacterEvent) -> bool {
        false
    }

    /// `nextFocusPath`: the way to the listener that would get the focus, from this one.
    fn next_focus_path(
        &mut self,
        _navigation_event: &FocusNavigationEvent,
    ) -> Option<ComponentPath> {
        None
    }

    fn is_mouse_over(&self, _mouse_x: f64, _mouse_y: f64) -> bool {
        false
    }

    fn set_focused(&mut self, focused: bool);

    fn is_focused(&self) -> bool;

    fn should_take_focus_after_interaction(&self) -> bool {
        true
    }

    fn captures_input(&self) -> bool {
        false
    }

    /// `getCurrentFocusPath`: a leaf when this one has the focus.
    fn get_current_focus_path(&self) -> Option<ComponentPath> {
        self.is_focused().then(ComponentPath::leaf)
    }

    fn get_rectangle(&self) -> ScreenRectangle {
        ScreenRectangle::empty()
    }

    fn get_border_for_arrow_navigation(&self, opposite: ScreenDirection) -> ScreenRectangle {
        self.get_rectangle().get_border(opposite)
    }

    /// `instanceof ContainerEventHandler`.
    fn as_container(&mut self) -> Option<&mut dyn ContainerEventHandler> {
        None
    }

    /// When an edit box's cursor started blinking (`EditBox.focusedTime`), which a scenario's
    /// screenshot waits on; none for anything else.
    fn cursor_blink_start(&self) -> Option<i64> {
        None
    }
}
