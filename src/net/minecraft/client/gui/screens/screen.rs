//! net/minecraft/client/gui/screens/Screen.java
//!
//! A screen over the game: its title, its widgets (the children that take input, the renderables
//! drawn in order), the focus among them, and what it draws behind them (the blurred level and the
//! menu background in a level, the transparent gradient for in-game UI). Java's abstract class is
//! `Screen`, the struct a screen holds, and the methods screens override are `ScreenClass`'s,
//! whose defaults are this module's functions. A screen's children are shared with its widget
//! lists like Java's, and its `minecraft` is a weak reference, which the buttons' callbacks keep.
//!
//! A screen without a level draws over the title screen's panorama, which a game driven by a test
//! holds still: Fabric's client game tests do (their `ScreenMixin` holds it as `extractPanorama`
//! begins), and so does the port's scenario driver.
//!
//! Not ported yet: narration (the narrator's state, `updateNarrationState`), click events' handling
//! (`defaultHandleClickEvent`), item tooltips, dropped files, and the screen's executor.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::net::minecraft::client::gui::component_path::ComponentPath;
use crate::net::minecraft::client::gui::components::abstract_widget::WidgetRef;
use crate::net::minecraft::client::gui::components::events::abstract_container_event_handler::AbstractContainerEventHandler;
use crate::net::minecraft::client::gui::components::events::container_event_handler::{
    self, ContainerEventHandler,
};
use crate::net::minecraft::client::gui::components::events::gui_event_listener::{
    GuiEventListener, ListenerRef,
};
use crate::net::minecraft::client::gui::components::renderable::RenderableRef;
use crate::net::minecraft::client::gui::components::tab_ordered_element::TabOrderedElement;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::navigation::focus_navigation_event::FocusNavigationEvent;
use crate::net::minecraft::client::gui::navigation::screen_direction::ScreenDirection;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::input_type;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;

/// A screen as the game holds it.
pub type ScreenRef = Rc<RefCell<dyn ScreenClass>>;

/// `Screen`'s fields, with `AbstractContainerEventHandler`'s.
pub struct Screen {
    pub title: Component,
    children: Vec<ListenerRef>,
    renderables: Vec<RenderableRef>,
    container: AbstractContainerEventHandler,
    minecraft: Weak<Minecraft>,
    initialized: bool,
    pub width: i32,
    pub height: i32,
    pub font: Rc<Font>,
}

pub const MENU_BACKGROUND: &str = "textures/gui/menu_background.png";
pub const HEADER_SEPARATOR: &str = "textures/gui/header_separator.png";
pub const FOOTER_SEPARATOR: &str = "textures/gui/footer_separator.png";
const INWORLD_MENU_BACKGROUND: &str = "textures/gui/inworld_menu_background.png";
pub const INWORLD_HEADER_SEPARATOR: &str = "textures/gui/inworld_header_separator.png";
pub const INWORLD_FOOTER_SEPARATOR: &str = "textures/gui/inworld_footer_separator.png";

impl Screen {
    /// `Screen(Component)`: the game's font.
    pub fn new(minecraft: &Rc<Minecraft>, title: Component) -> Screen {
        Screen {
            title,
            children: Vec::new(),
            renderables: Vec::new(),
            container: AbstractContainerEventHandler::default(),
            minecraft: Rc::downgrade(minecraft),
            initialized: false,
            width: 0,
            height: 0,
            font: minecraft.font(),
        }
    }

    /// `this.minecraft`.
    pub fn minecraft(&self) -> Rc<Minecraft> {
        self.minecraft
            .upgrade()
            .expect("the game outlives its screens")
    }

    pub fn minecraft_weak(&self) -> Weak<Minecraft> {
        self.minecraft.clone()
    }

    pub fn get_title(&self) -> &Component {
        &self.title
    }

    /// `addRenderableWidget`.
    pub fn add_renderable_widget<
        W: crate::net::minecraft::client::gui::components::abstract_widget::Widget + 'static,
    >(
        &mut self,
        widget: Rc<RefCell<W>>,
    ) -> Rc<RefCell<W>> {
        self.renderables.push(widget.clone());
        self.add_widget(widget)
    }

    /// `addRenderableWidget` of a widget a layout hands out (`visitWidgets`).
    pub fn add_renderable_widget_ref(&mut self, widget: &WidgetRef) {
        self.renderables.push(widget.clone());
        self.children.push(widget.clone());
    }

    /// `addRenderableOnly`.
    pub fn add_renderable_only(&mut self, renderable: RenderableRef) -> RenderableRef {
        self.renderables.push(renderable.clone());
        renderable
    }

    /// `addWidget`: a child that takes input, which isn't drawn.
    pub fn add_widget<W: GuiEventListener + 'static>(
        &mut self,
        widget: Rc<RefCell<W>>,
    ) -> Rc<RefCell<W>> {
        self.children.push(widget.clone());
        widget
    }

    /// `removeWidget`.
    pub fn remove_widget(&mut self, widget: &ListenerRef) {
        let address = Rc::as_ptr(widget) as *const ();
        self.renderables
            .retain(|renderable| Rc::as_ptr(renderable) as *const () != address);
        let focused = self
            .container
            .get_focused()
            .is_some_and(|focused| Rc::ptr_eq(&focused, widget));
        if focused {
            self.clear_focus();
        }
        self.children.retain(|child| !Rc::ptr_eq(child, widget));
    }

    /// `clearWidgets`.
    pub fn clear_widgets(&mut self) {
        self.renderables.clear();
        self.children.clear();
    }

    /// `clearFocus`: the focus path's every step loses the focus.
    pub fn clear_focus(&mut self) {
        if let Some(path) = container_event_handler::get_current_focus_path(self) {
            path.apply_focus(self, false);
        }
    }

    /// `changeFocus`.
    pub fn change_focus(&mut self, component_path: &ComponentPath) {
        self.clear_focus();
        component_path.apply_focus(self, true);
    }

    /// `isInputCaptured`: whether the focused leaf takes keys the game would.
    pub fn is_input_captured(&self) -> bool {
        container_event_handler::get_current_focus_path(self)
            .and_then(|path| path.leaf_component())
            .is_some_and(|leaf| leaf.borrow().captures_input())
    }

    /// `setInitialFocus(GuiEventListener)`: the focus on `target`, which is one of the children.
    pub fn set_initial_focus_on(&mut self, target: &ListenerRef) {
        let path = target
            .borrow_mut()
            .next_focus_path(&FocusNavigationEvent::InitialFocus);
        if let Some(path) = path {
            self.change_focus(&ComponentPath::path(target.clone(), path));
        }
    }

    /// The renderables' `extractRenderState`, in the order they were added.
    pub fn extract_renderables(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        for renderable in &self.renderables {
            renderable
                .borrow_mut()
                .extract_render_state(graphics, mouse_x, mouse_y, a);
        }
    }

    /// `getRectangle`.
    pub fn get_screen_rectangle(&self) -> ScreenRectangle {
        ScreenRectangle::new(0, 0, self.width, self.height)
    }
}

impl TabOrderedElement for Screen {}

impl GuiEventListener for Screen {
    fn set_focused(&mut self, focused: bool) {
        container_event_handler::set_focused(self, focused);
    }

    fn is_focused(&self) -> bool {
        container_event_handler::is_focused(self)
    }

    fn is_mouse_over(&self, _mouse_x: f64, _mouse_y: f64) -> bool {
        true
    }

    fn next_focus_path(
        &mut self,
        navigation_event: &FocusNavigationEvent,
    ) -> Option<ComponentPath> {
        container_event_handler::next_focus_path(self, navigation_event)
    }

    fn get_current_focus_path(&self) -> Option<ComponentPath> {
        container_event_handler::get_current_focus_path(self)
    }

    fn get_rectangle(&self) -> ScreenRectangle {
        self.get_screen_rectangle()
    }

    fn get_border_for_arrow_navigation(&self, opposite: ScreenDirection) -> ScreenRectangle {
        container_event_handler::get_border_for_arrow_navigation(self, opposite)
    }

    fn as_container(&mut self) -> Option<&mut dyn ContainerEventHandler> {
        Some(self)
    }
}

impl ContainerEventHandler for Screen {
    fn children(&self) -> Vec<ListenerRef> {
        self.children.clone()
    }

    fn is_dragging(&self) -> bool {
        self.container.is_dragging()
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.container.set_dragging(dragging);
    }

    fn get_focused(&self) -> Option<ListenerRef> {
        self.container.get_focused()
    }

    fn set_focused_child(&mut self, focused: Option<ListenerRef>) {
        self.container.set_focused(focused);
    }
}

/// `Screen`'s methods a screen can override.
pub trait ScreenClass: Any {
    fn screen(&self) -> &Screen;

    fn screen_mut(&mut self) -> &mut Screen;

    fn as_any(&self) -> &dyn Any;

    /// `init()`: the widgets, made for the screen's size.
    fn init(&mut self) {}

    fn tick(&mut self) {}

    fn removed(&mut self) {}

    fn added(&mut self) {}

    fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        self.screen_mut()
            .extract_renderables(graphics, mouse_x, mouse_y, a);
    }

    fn extract_background(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        extract_background(self, graphics, mouse_x, mouse_y, a);
    }

    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        key_pressed(self, event)
    }

    fn key_released(&mut self, event: &KeyEvent) -> bool {
        container_event_handler::key_released(self.screen_mut(), event)
    }

    fn char_typed(&mut self, event: &CharacterEvent) -> bool {
        container_event_handler::char_typed(self.screen_mut(), event)
    }

    fn mouse_clicked(&mut self, event: &MouseButtonEvent, double_click: bool) -> bool {
        container_event_handler::mouse_clicked(self.screen_mut(), event, double_click)
    }

    fn mouse_released(&mut self, event: &MouseButtonEvent) -> bool {
        container_event_handler::mouse_released(self.screen_mut(), event)
    }

    fn mouse_dragged(&mut self, event: &MouseButtonEvent, dx: f64, dy: f64) -> bool {
        container_event_handler::mouse_dragged(self.screen_mut(), event, dx, dy)
    }

    fn mouse_scrolled(&mut self, x: f64, y: f64, scroll_x: f64, scroll_y: f64) -> bool {
        container_event_handler::mouse_scrolled(self.screen_mut(), x, y, scroll_x, scroll_y)
    }

    fn mouse_moved(&mut self, _x: f64, _y: f64) {}

    fn should_close_on_esc(&self) -> bool {
        true
    }

    /// `onClose`: back to the game.
    fn on_close(&mut self) {
        self.screen().minecraft().set_screen(None);
    }

    fn is_pause_screen(&self) -> bool {
        true
    }

    fn is_in_game_ui(&self) -> bool {
        false
    }

    fn is_allowed_in_portal(&self) -> bool {
        self.is_pause_screen()
    }

    /// `repositionElements`: the widgets made again.
    fn reposition_elements(&mut self) {
        rebuild_widgets(self);
    }

    /// `setInitialFocus()`: the first widget Tab reaches, after keyboard input.
    fn set_initial_focus(&mut self) {
        if input_type::last_input_type().is_keyboard() {
            let screen = self.screen_mut();
            let focus_path = container_event_handler::next_focus_path(
                screen,
                &FocusNavigationEvent::TabNavigation(true),
            );
            if let Some(focus_path) = focus_path {
                screen.change_focus(&focus_path);
            }
        }
    }

    /// `insertText`, which a suggested command's click fills in.
    fn insert_text(&mut self, _text: &str, _replace: bool) {}

    fn shows_active_effects(&self) -> bool {
        false
    }

    fn can_interrupt_with_another_screen(&self) -> bool {
        self.should_close_on_esc()
    }
}

/// `init(int, int)`, which is final: the widgets made the first time, placed again after.
pub fn init<S: ScreenClass + ?Sized>(this: &mut S, width: i32, height: i32) {
    let screen = this.screen_mut();
    screen.width = width;
    screen.height = height;
    if !screen.initialized {
        this.init();
        this.set_initial_focus();
    } else {
        this.reposition_elements();
    }
    this.screen_mut().initialized = true;
}

/// `rebuildWidgets`.
pub fn rebuild_widgets<S: ScreenClass + ?Sized>(this: &mut S) {
    this.screen_mut().clear_widgets();
    this.screen_mut().clear_focus();
    this.init();
    this.set_initial_focus();
}

/// `resize`.
pub fn resize<S: ScreenClass + ?Sized>(this: &mut S, width: i32, height: i32) {
    this.screen_mut().width = width;
    this.screen_mut().height = height;
    this.reposition_elements();
}

/// `extractRenderStateWithTooltipAndSubtitles`, which is final: the background a stratum under the
/// rest, and the tooltip over it.
pub fn extract_render_state_with_tooltip_and_subtitles<S: ScreenClass + ?Sized>(
    this: &mut S,
    graphics: &mut GuiGraphicsExtractor,
    mouse_x: i32,
    mouse_y: i32,
    a: f32,
) {
    graphics.next_stratum();
    this.extract_background(graphics, mouse_x, mouse_y, a);
    graphics.next_stratum();
    this.extract_render_state(graphics, mouse_x, mouse_y, a);
    graphics.extract_deferred_elements(mouse_x, mouse_y, a);
}

/// `extractBackground`: in-game UI over the level as a gradient, other screens over the blurred
/// level and the menu background. The subtitles aren't ported.
pub fn extract_background<S: ScreenClass + ?Sized>(
    this: &mut S,
    graphics: &mut GuiGraphicsExtractor,
    _mouse_x: i32,
    _mouse_y: i32,
    _a: f32,
) {
    if this.is_in_game_ui() {
        extract_transparent_background(this.screen(), graphics);
    } else {
        if this.screen().minecraft().level.borrow().is_none() {
            extract_panorama(this.screen(), graphics, _a);
        }
        extract_blurred_background(this.screen(), graphics);
        extract_menu_background(this.screen(), graphics);
    }
}

/// `extractPanorama`: the game's panorama, held still in a game a test drives.
pub fn extract_panorama(screen: &Screen, graphics: &mut GuiGraphicsExtractor, _a: f32) {
    let minecraft = screen.minecraft();
    let mut panorama = minecraft.panorama.borrow_mut();
    if minecraft.is_game_test() {
        panorama.hold_spin();
    }
    let panorama_speed = minecraft.options.borrow().panorama_speed;
    panorama.extract_render_state(
        graphics,
        screen.width,
        screen.height,
        minecraft.realtime_delta_ticks(),
        panorama_speed,
    );
}

/// `extractBlurredBackground`: the level blurred under the next strata, when the menu blur is on.
pub fn extract_blurred_background(screen: &Screen, graphics: &mut GuiGraphicsExtractor) {
    let blur_radius = screen
        .minecraft()
        .options
        .borrow()
        .get_menu_background_blurriness() as f32;
    if blur_radius >= 1.0 {
        graphics.blur_before_this_stratum();
    }
}

/// `extractMenuBackground()`: over the whole screen.
pub fn extract_menu_background(screen: &Screen, graphics: &mut GuiGraphicsExtractor) {
    extract_menu_background_in(screen, graphics, 0, 0, screen.width, screen.height);
}

/// `extractMenuBackground(GuiGraphicsExtractor, int, int, int, int)`: the in-world background in a
/// level, the other one without.
pub fn extract_menu_background_in(
    screen: &Screen,
    graphics: &mut GuiGraphicsExtractor,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    let in_level = screen.minecraft().level.borrow().is_some();
    let background = Identifier::with_default_namespace(if in_level {
        INWORLD_MENU_BACKGROUND
    } else {
        MENU_BACKGROUND
    });
    extract_menu_background_texture(graphics, &background, x, y, 0.0, 0.0, width, height);
}

/// `extractMenuBackgroundTexture`: the 32-pixel texture tiled.
#[allow(clippy::too_many_arguments)]
pub fn extract_menu_background_texture(
    graphics: &mut GuiGraphicsExtractor,
    menu_background: &Identifier,
    x: i32,
    y: i32,
    u: f32,
    v: f32,
    width: i32,
    height: i32,
) {
    graphics.blit(
        &RenderPipelines::get().GUI_TEXTURED,
        menu_background,
        x,
        y,
        u,
        v,
        width,
        height,
        32,
        32,
    );
}

/// `extractTransparentBackground`.
pub fn extract_transparent_background(screen: &Screen, graphics: &mut GuiGraphicsExtractor) {
    graphics.fill_gradient(0, 0, screen.width, screen.height, -1072689136, -804253680);
}

/// `keyPressed`: Escape closes the screen, then the focused child's keys, then Tab and the arrows
/// move the focus.
pub fn key_pressed<S: ScreenClass + ?Sized>(this: &mut S, event: &KeyEvent) -> bool {
    if event.is_escape() && this.should_close_on_esc() {
        this.on_close();
        return true;
    }
    if container_event_handler::key_pressed(this.screen_mut(), event) {
        return true;
    }

    let navigation_event = match event.shortcut_key() {
        9 => Some(FocusNavigationEvent::TabNavigation(!event.has_shift_down())),
        1073741903 => Some(FocusNavigationEvent::arrow(ScreenDirection::RIGHT)),
        1073741904 => Some(FocusNavigationEvent::arrow(ScreenDirection::LEFT)),
        1073741905 => Some(FocusNavigationEvent::arrow(ScreenDirection::DOWN)),
        1073741906 => Some(FocusNavigationEvent::arrow(ScreenDirection::UP)),
        _ => None,
    };
    if let Some(navigation_event) = navigation_event {
        let screen = this.screen_mut();
        let mut focus_path = container_event_handler::next_focus_path(screen, &navigation_event);
        if focus_path.is_none()
            && matches!(navigation_event, FocusNavigationEvent::TabNavigation(_))
        {
            screen.clear_focus();
            focus_path = container_event_handler::next_focus_path(screen, &navigation_event);
        }
        if let Some(focus_path) = focus_path {
            screen.change_focus(&focus_path);
        }
    }
    false
}

/// `Screen`'s `isValidCharacterForName`.
pub fn is_valid_character_for_name(current_name: &str, new_char: i32, cursor_pos: i32) -> bool {
    let units: Vec<u16> = current_name.encode_utf16().collect();
    let index_of = |c: u16| {
        units
            .iter()
            .position(|&unit| unit == c)
            .map_or(-1, |i| i as i32)
    };
    let colon_pos = index_of(58);
    let slash_pos = index_of(47);
    if new_char == 58 {
        (slash_pos == -1 || cursor_pos <= slash_pos) && colon_pos == -1
    } else if new_char == 47 {
        cursor_pos > colon_pos
    } else {
        new_char == 95
            || new_char == 45
            || (97..=122).contains(&new_char)
            || (48..=57).contains(&new_char)
            || new_char == 46
    }
}
