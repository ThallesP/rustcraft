//! net/minecraft/client/gui/components/events/ContainerEventHandler.java
//!
//! A listener with children, which hands the input to the one under the mouse or the focused one,
//! and moves the focus among them with Tab and the arrows. Java's default methods fill in
//! `GuiEventListener`'s, which a Rust trait can't, so they're this module's functions, which a
//! container's `GuiEventListener` methods forward to (`container_event_handler::key_pressed(self,
//! event)`). A child is borrowed while it handles an event, so a container's children are never
//! the container itself.

use std::cmp::Ordering;
use std::rc::Rc;

use crate::net::minecraft::client::gui::component_path::ComponentPath;
use crate::net::minecraft::client::gui::components::events::gui_event_listener::{
    GuiEventListener, ListenerRef,
};
use crate::net::minecraft::client::gui::navigation::focus_navigation_event::FocusNavigationEvent;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;
use crate::net::minecraft::client::gui::navigation::screen_position::ScreenPosition;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;

pub trait ContainerEventHandler: GuiEventListener {
    fn children(&self) -> Vec<ListenerRef>;

    fn is_dragging(&self) -> bool;

    fn set_dragging(&mut self, dragging: bool);

    fn get_focused(&self) -> Option<ListenerRef>;

    /// `setFocused(GuiEventListener)`.
    fn set_focused_child(&mut self, focused: Option<ListenerRef>);
}

fn same(a: &ListenerRef, b: &ListenerRef) -> bool {
    Rc::ptr_eq(a, b)
}

pub fn get_child_at(this: &dyn ContainerEventHandler, x: f64, y: f64) -> Option<ListenerRef> {
    this.children()
        .into_iter()
        .find(|child| child.borrow().is_mouse_over(x, y))
}

pub fn mouse_clicked(
    this: &mut dyn ContainerEventHandler,
    event: &MouseButtonEvent,
    double_click: bool,
) -> bool {
    let Some(widget) = get_child_at(this, event.x, event.y) else {
        return false;
    };
    let clicked = widget.borrow_mut().mouse_clicked(event, double_click);
    if clicked && widget.borrow().should_take_focus_after_interaction() {
        this.set_focused_child(Some(widget));
        if event.button() == 1 {
            this.set_dragging(true);
        }
    }
    true
}

pub fn mouse_released(this: &mut dyn ContainerEventHandler, event: &MouseButtonEvent) -> bool {
    if event.button() == 1 && this.is_dragging() {
        this.set_dragging(false);
        if let Some(focused) = this.get_focused() {
            return focused.borrow_mut().mouse_released(event);
        }
    }
    false
}

pub fn mouse_dragged(
    this: &mut dyn ContainerEventHandler,
    event: &MouseButtonEvent,
    dx: f64,
    dy: f64,
) -> bool {
    match this.get_focused() {
        Some(focused) if this.is_dragging() && event.button() == 1 => {
            focused.borrow_mut().mouse_dragged(event, dx, dy)
        }
        _ => false,
    }
}

pub fn mouse_scrolled(
    this: &mut dyn ContainerEventHandler,
    x: f64,
    y: f64,
    scroll_x: f64,
    scroll_y: f64,
) -> bool {
    get_child_at(this, x, y)
        .is_some_and(|child| child.borrow_mut().mouse_scrolled(x, y, scroll_x, scroll_y))
}

pub fn key_pressed(this: &mut dyn ContainerEventHandler, event: &KeyEvent) -> bool {
    this.get_focused()
        .is_some_and(|focused| focused.borrow_mut().key_pressed(event))
}

pub fn key_released(this: &mut dyn ContainerEventHandler, event: &KeyEvent) -> bool {
    this.get_focused()
        .is_some_and(|focused| focused.borrow_mut().key_released(event))
}

pub fn char_typed(this: &mut dyn ContainerEventHandler, event: &CharacterEvent) -> bool {
    this.get_focused()
        .is_some_and(|focused| focused.borrow_mut().char_typed(event))
}

pub fn get_border_for_arrow_navigation(
    this: &dyn ContainerEventHandler,
    opposite: ScreenDirection,
) -> ScreenRectangle {
    match this.get_focused() {
        Some(focused) => focused.borrow().get_border_for_arrow_navigation(opposite),
        None => this.get_rectangle().get_border(opposite),
    }
}

/// `setFocused(boolean)`: losing the focus clears the focused child.
pub fn set_focused(this: &mut dyn ContainerEventHandler, focused: bool) {
    if !focused {
        this.set_focused_child(None);
    }
}

pub fn is_focused(this: &dyn ContainerEventHandler) -> bool {
    this.get_focused().is_some()
}

pub fn get_current_focus_path(this: &dyn ContainerEventHandler) -> Option<ComponentPath> {
    let focused = this.get_focused()?;
    let child_path = focused.borrow().get_current_focus_path()?;
    Some(ComponentPath::path(focused, child_path))
}

pub fn next_focus_path(
    this: &mut dyn ContainerEventHandler,
    navigation_event: &FocusNavigationEvent,
) -> Option<ComponentPath> {
    if let Some(focus) = this.get_focused() {
        let focus_path = focus.borrow_mut().next_focus_path(navigation_event);
        if let Some(focus_path) = focus_path {
            return Some(ComponentPath::path(focus, focus_path));
        }
    }

    match *navigation_event {
        FocusNavigationEvent::TabNavigation(forward) => handle_tab_navigation(this, forward),
        FocusNavigationEvent::ArrowNavigation(direction, previous_focus) => {
            handle_arrow_navigation(this, direction, previous_focus)
        }
        FocusNavigationEvent::InitialFocus => None,
    }
}

fn handle_tab_navigation(
    this: &mut dyn ContainerEventHandler,
    forward: bool,
) -> Option<ComponentPath> {
    let focus = this.get_focused();
    let mut sorted_children = this.children();
    // Collections.sort is stable, like sort_by_key.
    sorted_children.sort_by_key(|child| child.borrow().get_tab_order_group());
    let index = focus
        .as_ref()
        .and_then(|focus| sorted_children.iter().position(|child| same(child, focus)));
    let new_index = match index {
        Some(index) => index + usize::from(forward),
        None if forward => 0,
        None => sorted_children.len(),
    };
    let event = FocusNavigationEvent::TabNavigation(forward);
    let candidates: Box<dyn Iterator<Item = &ListenerRef>> = if forward {
        Box::new(sorted_children[new_index..].iter())
    } else {
        Box::new(sorted_children[..new_index].iter().rev())
    };
    for child in candidates {
        let focus_path = child.borrow_mut().next_focus_path(&event);
        if let Some(focus_path) = focus_path {
            return Some(ComponentPath::path(child.clone(), focus_path));
        }
    }
    None
}

fn handle_arrow_navigation(
    this: &mut dyn ContainerEventHandler,
    direction: ScreenDirection,
    previous_focus: Option<ScreenRectangle>,
) -> Option<ComponentPath> {
    let event = FocusNavigationEvent::ArrowNavigation(direction, previous_focus);
    match this.get_focused() {
        None => {
            let border = match previous_focus {
                Some(previous_focus) => previous_focus,
                None => get_border_for_arrow_navigation(this, direction.get_opposite()),
            };
            next_focus_path_in_direction(this, &border, direction, None, &event)
        }
        Some(focus) => {
            let focused_rectangle = focus.borrow().get_border_for_arrow_navigation(direction);
            next_focus_path_in_direction(
                this,
                &focused_rectangle,
                direction,
                Some(&focus),
                &FocusNavigationEvent::ArrowNavigation(direction, Some(focused_rectangle)),
            )
        }
    }
}

fn next_focus_path_in_direction(
    this: &mut dyn ContainerEventHandler,
    focused_rectangle: &ScreenRectangle,
    direction: ScreenDirection,
    excluded: Option<&ListenerRef>,
    navigation_event: &FocusNavigationEvent,
) -> Option<ComponentPath> {
    let axis = direction.get_axis();
    let other_axis = axis.orthogonal();
    let positive_direction_other_axis = other_axis.get_positive();
    let focused_first_bound = focused_rectangle.get_bound_in_direction(direction.get_opposite());
    let mut potential_children = Vec::new();
    for child in this.children() {
        if excluded.is_some_and(|excluded| same(&child, excluded)) {
            continue;
        }
        let child_rectangle = child.borrow().get_rectangle();
        if child_rectangle.overlaps_in_axis(focused_rectangle, other_axis) {
            let child_first_bound =
                child_rectangle.get_bound_in_direction(direction.get_opposite());
            if direction.is_after(child_first_bound, focused_first_bound)
                || (child_first_bound == focused_first_bound
                    && direction.is_after(
                        child_rectangle.get_bound_in_direction(direction),
                        focused_rectangle.get_bound_in_direction(direction),
                    ))
            {
                potential_children.push(child);
            }
        }
    }

    potential_children.sort_by(|a, b| {
        let (a, b) = (a.borrow().get_rectangle(), b.borrow().get_rectangle());
        direction
            .compare_coordinates(
                a.get_bound_in_direction(direction.get_opposite()),
                b.get_bound_in_direction(direction.get_opposite()),
            )
            .then_with(|| {
                positive_direction_other_axis.compare_coordinates(
                    a.get_bound_in_direction(positive_direction_other_axis.get_opposite()),
                    b.get_bound_in_direction(positive_direction_other_axis.get_opposite()),
                )
            })
    });
    for child in potential_children {
        let component_path = child.borrow_mut().next_focus_path(navigation_event);
        if let Some(component_path) = component_path {
            return Some(ComponentPath::path(child, component_path));
        }
    }
    next_focus_path_vaguely_in_direction(
        this,
        focused_rectangle,
        direction,
        excluded,
        navigation_event,
    )
}

fn next_focus_path_vaguely_in_direction(
    this: &mut dyn ContainerEventHandler,
    focused_rectangle: &ScreenRectangle,
    direction: ScreenDirection,
    excluded: Option<&ListenerRef>,
    navigation_event: &FocusNavigationEvent,
) -> Option<ComponentPath> {
    let axis = direction.get_axis();
    let other_axis = axis.orthogonal();
    let focused_side_center = ScreenPosition::of(
        axis,
        focused_rectangle.get_bound_in_direction(direction),
        focused_rectangle.get_center_in_axis(other_axis),
    );
    let mut potential_children: Vec<(ListenerRef, i64)> = Vec::new();
    for child in this.children() {
        if excluded.is_some_and(|excluded| same(&child, excluded)) {
            continue;
        }
        let child_rectangle = child.borrow().get_rectangle();
        let child_opposing_side_center = ScreenPosition::of(
            axis,
            child_rectangle.get_bound_in_direction(direction.get_opposite()),
            child_rectangle.get_center_in_axis(other_axis),
        );
        if direction.is_after(
            child_opposing_side_center.get_coordinate(axis),
            focused_side_center.get_coordinate(axis),
        ) {
            // Vector2i.distanceSquared: the differences as longs.
            let dx = i64::from(focused_side_center.x) - i64::from(child_opposing_side_center.x);
            let dy = i64::from(focused_side_center.y) - i64::from(child_opposing_side_center.y);
            potential_children.push((child, dx * dx + dy * dy));
        }
    }
    // comparingDouble of the squared distances, a stable sort.
    potential_children.sort_by(|a, b| {
        (a.1 as f64)
            .partial_cmp(&(b.1 as f64))
            .unwrap_or(Ordering::Equal)
    });
    for (child, _) in potential_children {
        let component_path = child.borrow_mut().next_focus_path(navigation_event);
        if let Some(component_path) = component_path {
            return Some(ComponentPath::path(child, component_path));
        }
    }
    None
}
