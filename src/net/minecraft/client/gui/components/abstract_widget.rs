//! net/minecraft/client/gui/components/AbstractWidget.java
//!
//! A widget's place, size, message and state (hovered, active, visible, its alpha and focus), its
//! tooltip, and how it takes clicks. Java's abstract class is a struct the widgets hold
//! (`AbstractWidget`), and its methods the widgets override are `AbstractWidgetClass`'s, whose
//! defaults are this module's functions; every `AbstractWidgetClass` is a `GuiEventListener`, a
//! `Renderable` and a `LayoutElement` through them, which `Widget` gathers, and a screen holds a
//! widget as a `WidgetRef`. `WithInactiveMessage` (a message greyed out while inactive) is
//! `AbstractWidget::inactive_message`, which a button's `get_message` reads.
//!
//! Not ported yet: narration (`updateNarration`, the narration priority), sounds (the click's
//! `playDownSound`), and the scrolling message over a widget's contents in a scissor
//! (`extractScrollingStringOverContents` draws through the text collector's).

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::net::minecraft::client::gui::active_text_collector::ActiveTextCollector;
use crate::net::minecraft::client::gui::component_path::ComponentPath;
use crate::net::minecraft::client::gui::components::events::container_event_handler::ContainerEventHandler;
use crate::net::minecraft::client::gui::components::events::gui_event_listener::GuiEventListener;
use crate::net::minecraft::client::gui::components::renderable::Renderable;
use crate::net::minecraft::client::gui::components::tab_ordered_element::TabOrderedElement;
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::components::widget_tooltip_holder::WidgetTooltipHolder;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::layouts::layout_element::LayoutElement;
use crate::net::minecraft::client::gui::navigation::focus_navigation_event::FocusNavigationEvent;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::input::mouse_button_info::MouseButtonInfo;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_utils::ComponentUtils;
use crate::net::minecraft::network::chat::style::Style;

/// `AbstractWidget`'s fields.
pub struct AbstractWidget {
    pub width: i32,
    pub height: i32,
    x: i32,
    y: i32,
    pub message: Component,
    pub is_hovered: bool,
    pub active: bool,
    pub visible: bool,
    pub alpha: f32,
    tab_order_group: i32,
    focused: bool,
    pub tooltip: WidgetTooltipHolder,
    /// `WithInactiveMessage.inactiveMessage`, for the widgets that are one.
    pub inactive_message: Option<Component>,
}

impl AbstractWidget {
    pub fn new(x: i32, y: i32, width: i32, height: i32, message: Component) -> AbstractWidget {
        AbstractWidget {
            width,
            height,
            x,
            y,
            message,
            is_hovered: false,
            active: true,
            visible: true,
            alpha: 1.0,
            tab_order_group: 0,
            focused: false,
            tooltip: WidgetTooltipHolder::default(),
            inactive_message: None,
        }
    }

    /// `WithInactiveMessage`'s constructor.
    pub fn with_inactive_message(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: Component,
    ) -> AbstractWidget {
        let inactive_message = Some(AbstractWidget::default_inactive_message(&message));
        AbstractWidget {
            inactive_message,
            ..AbstractWidget::new(x, y, width, height, message)
        }
    }

    /// `WithInactiveMessage.defaultInactiveMessage`: the message in grey.
    pub fn default_inactive_message(active_message: &Component) -> Component {
        ComponentUtils::merge_styles(active_message, &Style::EMPTY.with_color_rgb(-6250336))
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn get_tab_order_group(&self) -> i32 {
        self.tab_order_group
    }

    pub fn set_tab_order_group(&mut self, tab_order_group: i32) {
        self.tab_order_group = tab_order_group;
    }

    pub fn set_tooltip(&mut self, tooltip: Option<Tooltip>) {
        self.tooltip.set(tooltip);
    }

    pub fn set_tooltip_delay(&mut self, delay: Duration) {
        self.tooltip.set_delay(delay);
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }
}

/// `AbstractWidget`'s methods a widget can override, and the abstract ones.
pub trait AbstractWidgetClass: 'static {
    fn widget(&self) -> &AbstractWidget;

    fn widget_mut(&mut self) -> &mut AbstractWidget;

    fn extract_widget_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    );

    fn extract_tooltip_for_next_render_pass(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
    ) {
        let (is_hovered, is_focused) = (self.is_hovered(), self.is_focused());
        let rectangle = self.get_rectangle();
        self.widget_mut()
            .tooltip
            .refresh_tooltip_for_next_render_pass(
                graphics, mouse_x, mouse_y, is_hovered, is_focused, rectangle,
            );
    }

    fn on_click(&mut self, _event: &MouseButtonEvent, _double_click: bool) {}

    fn on_release(&mut self, _event: &MouseButtonEvent) {}

    fn on_drag(&mut self, _event: &MouseButtonEvent, _dx: f64, _dy: f64) {}

    fn mouse_clicked(&mut self, event: &MouseButtonEvent, double_click: bool) -> bool {
        mouse_clicked(self, event, double_click)
    }

    fn mouse_released(&mut self, event: &MouseButtonEvent) -> bool {
        if self.is_valid_click_button(&event.button_info) {
            self.on_release(event);
            true
        } else {
            false
        }
    }

    fn mouse_dragged(&mut self, event: &MouseButtonEvent, dx: f64, dy: f64) -> bool {
        if self.is_valid_click_button(&event.button_info) {
            self.on_drag(event, dx, dy);
            true
        } else {
            false
        }
    }

    fn mouse_moved(&mut self, _x: f64, _y: f64) {}

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

    fn is_valid_click_button(&self, button_info: &MouseButtonInfo) -> bool {
        button_info.button == 1
    }

    fn next_focus_path(
        &mut self,
        _navigation_event: &FocusNavigationEvent,
    ) -> Option<ComponentPath> {
        if !self.is_active() || self.is_focused() {
            None
        } else {
            Some(ComponentPath::leaf())
        }
    }

    fn get_current_focus_path(&self) -> Option<ComponentPath> {
        self.is_focused().then(ComponentPath::leaf)
    }

    fn is_mouse_over(&self, mouse_x: f64, mouse_y: f64) -> bool {
        self.is_active() && are_coordinates_in_rectangle(self, mouse_x, mouse_y)
    }

    fn get_width(&self) -> i32 {
        self.widget().width
    }

    fn get_height(&self) -> i32 {
        self.widget().height
    }

    fn set_x(&mut self, x: i32) {
        self.widget_mut().set_x(x);
    }

    fn set_y(&mut self, y: i32) {
        self.widget_mut().set_y(y);
    }

    fn set_width(&mut self, width: i32) {
        self.widget_mut().width = width;
    }

    /// `getMessage`, a greyed message for an inactive `WithInactiveMessage`.
    fn get_message(&self) -> Component {
        let widget = self.widget();
        match &widget.inactive_message {
            Some(inactive_message) if !widget.active => inactive_message.clone(),
            _ => widget.message.clone(),
        }
    }

    fn set_message(&mut self, message: Component) {
        let widget = self.widget_mut();
        if widget.inactive_message.is_some() {
            widget.inactive_message = Some(AbstractWidget::default_inactive_message(&message));
        }
        widget.message = message;
    }

    fn is_focused(&self) -> bool {
        self.widget().is_focused()
    }

    fn set_focused(&mut self, focused: bool) {
        self.widget_mut().set_focused(focused);
    }

    fn is_hovered(&self) -> bool {
        self.widget().is_hovered
    }

    fn is_hovered_or_focused(&self) -> bool {
        self.is_hovered() || self.is_focused()
    }

    fn is_active(&self) -> bool {
        self.widget().visible && self.widget().active
    }

    fn should_take_focus_after_interaction(&self) -> bool {
        true
    }

    fn captures_input(&self) -> bool {
        false
    }

    fn get_rectangle(&self) -> ScreenRectangle {
        ScreenRectangle::new(
            self.widget().get_x(),
            self.widget().get_y(),
            self.get_width(),
            self.get_height(),
        )
    }

    /// `instanceof ContainerEventHandler`, for the widgets that hold others.
    fn as_container(&mut self) -> Option<&mut dyn ContainerEventHandler> {
        None
    }

    /// `GuiEventListener::cursor_blink_start`: an edit box's.
    fn cursor_blink_start(&self) -> Option<i64> {
        None
    }
}

/// `extractRenderState`, which is final: hovered when the mouse is over it within the scissor, then
/// the widget's own state and its tooltip.
pub fn extract_render_state<W: AbstractWidgetClass + ?Sized>(
    this: &mut W,
    graphics: &mut GuiGraphicsExtractor,
    mouse_x: i32,
    mouse_y: i32,
    a: f32,
) {
    if this.widget().visible {
        let is_hovered = graphics.contains_point_in_scissor(mouse_x, mouse_y)
            && are_coordinates_in_rectangle(this, f64::from(mouse_x), f64::from(mouse_y));
        this.widget_mut().is_hovered = is_hovered;
        this.extract_widget_render_state(graphics, mouse_x, mouse_y, a);
        this.extract_tooltip_for_next_render_pass(graphics, mouse_x, mouse_y);
    }
}

/// `mouseClicked`: an active widget takes a click of its button over it.
pub fn mouse_clicked<W: AbstractWidgetClass + ?Sized>(
    this: &mut W,
    event: &MouseButtonEvent,
    double_click: bool,
) -> bool {
    if !this.is_active() {
        return false;
    }
    if this.is_valid_click_button(&event.button_info) && this.is_mouse_over(event.x, event.y) {
        this.on_click(event, double_click);
        return true;
    }
    false
}

/// `areCoordinatesInRectangle`.
pub fn are_coordinates_in_rectangle<W: AbstractWidgetClass + ?Sized>(
    this: &W,
    x: f64,
    y: f64,
) -> bool {
    let (left, top) = (this.widget().get_x(), this.widget().get_y());
    x >= f64::from(left)
        && y >= f64::from(top)
        && x < f64::from(left + this.get_width())
        && y < f64::from(top + this.get_height())
}

/// `extractScrollingStringOverContents`: the message between the margins, scrolling when it's too
/// wide.
pub fn extract_scrolling_string_over_contents<W: AbstractWidgetClass + ?Sized>(
    this: &W,
    output: &mut dyn ActiveTextCollector,
    message: &Component,
    margin: i32,
) {
    let left = this.widget().get_x() + margin;
    let right = this.widget().get_x() + this.get_width() - margin;
    let top = this.widget().get_y();
    let bottom = this.widget().get_y() + this.get_height();
    output.accept_scrolling_with_default_center(message, left, right, top, bottom);
}

/// A widget: everything a screen and a layout hold one as.
pub trait Widget: GuiEventListener + Renderable + LayoutElement {
    fn abstract_widget(&self) -> &AbstractWidget;

    fn abstract_widget_mut(&mut self) -> &mut AbstractWidget;

    fn get_message(&self) -> Component;
}

/// A widget as its holders share it.
pub type WidgetRef = Rc<RefCell<dyn Widget>>;

impl<W: AbstractWidgetClass> Widget for W {
    fn abstract_widget(&self) -> &AbstractWidget {
        self.widget()
    }

    fn abstract_widget_mut(&mut self) -> &mut AbstractWidget {
        self.widget_mut()
    }

    fn get_message(&self) -> Component {
        AbstractWidgetClass::get_message(self)
    }
}

impl<W: AbstractWidgetClass> TabOrderedElement for W {
    fn get_tab_order_group(&self) -> i32 {
        self.widget().get_tab_order_group()
    }
}

impl<W: AbstractWidgetClass> Renderable for W {
    fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        extract_render_state(self, graphics, mouse_x, mouse_y, a);
    }
}

impl<W: AbstractWidgetClass> LayoutElement for W {
    fn set_x(&mut self, x: i32) {
        AbstractWidgetClass::set_x(self, x);
    }

    fn set_y(&mut self, y: i32) {
        AbstractWidgetClass::set_y(self, y);
    }

    fn get_x(&self) -> i32 {
        self.widget().get_x()
    }

    fn get_y(&self) -> i32 {
        self.widget().get_y()
    }

    fn get_width(&self) -> i32 {
        AbstractWidgetClass::get_width(self)
    }

    fn get_height(&self) -> i32 {
        AbstractWidgetClass::get_height(self)
    }

    fn get_layout_rectangle(&self) -> ScreenRectangle {
        AbstractWidgetClass::get_rectangle(self)
    }
}

impl<W: AbstractWidgetClass> GuiEventListener for W {
    fn mouse_moved(&mut self, x: f64, y: f64) {
        AbstractWidgetClass::mouse_moved(self, x, y);
    }

    fn mouse_clicked(&mut self, event: &MouseButtonEvent, double_click: bool) -> bool {
        AbstractWidgetClass::mouse_clicked(self, event, double_click)
    }

    fn mouse_released(&mut self, event: &MouseButtonEvent) -> bool {
        AbstractWidgetClass::mouse_released(self, event)
    }

    fn mouse_dragged(&mut self, event: &MouseButtonEvent, dx: f64, dy: f64) -> bool {
        AbstractWidgetClass::mouse_dragged(self, event, dx, dy)
    }

    fn mouse_scrolled(&mut self, x: f64, y: f64, scroll_x: f64, scroll_y: f64) -> bool {
        AbstractWidgetClass::mouse_scrolled(self, x, y, scroll_x, scroll_y)
    }

    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        AbstractWidgetClass::key_pressed(self, event)
    }

    fn key_released(&mut self, event: &KeyEvent) -> bool {
        AbstractWidgetClass::key_released(self, event)
    }

    fn char_typed(&mut self, event: &CharacterEvent) -> bool {
        AbstractWidgetClass::char_typed(self, event)
    }

    fn next_focus_path(
        &mut self,
        navigation_event: &FocusNavigationEvent,
    ) -> Option<ComponentPath> {
        AbstractWidgetClass::next_focus_path(self, navigation_event)
    }

    fn is_mouse_over(&self, mouse_x: f64, mouse_y: f64) -> bool {
        AbstractWidgetClass::is_mouse_over(self, mouse_x, mouse_y)
    }

    fn set_focused(&mut self, focused: bool) {
        AbstractWidgetClass::set_focused(self, focused);
    }

    fn is_focused(&self) -> bool {
        AbstractWidgetClass::is_focused(self)
    }

    fn should_take_focus_after_interaction(&self) -> bool {
        AbstractWidgetClass::should_take_focus_after_interaction(self)
    }

    fn captures_input(&self) -> bool {
        AbstractWidgetClass::captures_input(self)
    }

    fn get_current_focus_path(&self) -> Option<ComponentPath> {
        AbstractWidgetClass::get_current_focus_path(self)
    }

    fn get_rectangle(&self) -> ScreenRectangle {
        AbstractWidgetClass::get_rectangle(self)
    }

    fn as_container(&mut self) -> Option<&mut dyn ContainerEventHandler> {
        AbstractWidgetClass::as_container(self)
    }

    fn cursor_blink_start(&self) -> Option<i64> {
        AbstractWidgetClass::cursor_blink_start(self)
    }
}
