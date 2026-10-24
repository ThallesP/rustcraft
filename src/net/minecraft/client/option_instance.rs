//! net/minecraft/client/OptionInstance.java
//!
//! An option as the screens show it: its caption, how its value reads (`toString`, from the
//! caption and the value), its tooltip for a value, the values it takes, and the button that sets
//! it. Java's instance holds the option's value, and `Options` holds the instances; the port's
//! `Options` holds the values as plain fields, which the rest of the client reads, so an instance
//! is made by its `Options` getter (`Options::fov`) and reaches its field through it (`get`, `set`
//! taking the options, Java's back-reference). Its value sets are `ValueSet`'s kinds.
//!
//! Not ported yet: the cycle buttons' value sets (`Enum`, `AltEnum`, `LazyEnum`, booleans,
//! `ClampingLazyMaxIntRange`), `UnitDouble`, `SliderableEnum`, the codecs `options.txt` is read and
//! written with, `xmap`, and narration.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use crate::net::minecraft::client::gui::components::abstract_option_slider_button;
use crate::net::minecraft::client::gui::components::abstract_slider_button::{
    self, AbstractSliderButton, SliderClass,
};
use crate::net::minecraft::client::gui::components::abstract_widget::{
    AbstractWidget, AbstractWidgetClass, WidgetRef,
};
use crate::net::minecraft::client::gui::components::tooltip::Tooltip;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::util::mth::Mth;

/// `TooltipSupplier`: the tooltip for a value, if any.
pub type TooltipSupplier<T> = Rc<dyn Fn(&T) -> Option<Tooltip>>;

/// `CaptionBasedToString`: the value as it reads, from the caption.
pub type CaptionBasedToString<T> = Rc<dyn Fn(&Component, &T) -> Component>;

/// `ValueUpdateListener`.
pub type ValueUpdateListener<T> = Rc<dyn Fn(&T)>;

/// The option's field in `Options`, which holds its value.
pub struct OptionField<T> {
    pub get: fn(&Options) -> T,
    pub set: fn(&mut Options, T),
}

impl<T> Clone for OptionField<T> {
    fn clone(&self) -> OptionField<T> {
        OptionField {
            get: self.get,
            set: self.set,
        }
    }
}

/// `ValueSet`, by its kinds ported.
pub enum ValueSet<T> {
    /// `SliderableValueSet`: values along a slider.
    Sliderable(Rc<dyn SliderableValueSet<T>>),
}

impl<T> Clone for ValueSet<T> {
    fn clone(&self) -> ValueSet<T> {
        match self {
            ValueSet::Sliderable(values) => ValueSet::Sliderable(Rc::clone(values)),
        }
    }
}

impl<T: Clone + PartialEq + 'static> ValueSet<T> {
    pub fn validate_value(&self, value: &T) -> Option<T> {
        match self {
            ValueSet::Sliderable(values) => values.validate_value(value),
        }
    }

    /// `createButton`, applied to the instance: the widget that sets its value.
    #[allow(clippy::too_many_arguments)]
    fn create_button(
        &self,
        tooltip: &TooltipSupplier<T>,
        minecraft: &Weak<Minecraft>,
        x: i32,
        y: i32,
        width: i32,
        on_value_changed: ValueUpdateListener<T>,
        instance: &OptionInstance<T>,
    ) -> WidgetRef {
        match self {
            ValueSet::Sliderable(values) => Rc::new(RefCell::new(OptionInstanceSliderButton::new(
                minecraft.clone(),
                x,
                y,
                width,
                20,
                instance.clone(),
                Rc::clone(values),
                Rc::clone(tooltip),
                on_value_changed,
                values.apply_value_immediately(),
            ))),
        }
    }
}

/// `SliderableValueSet`.
pub trait SliderableValueSet<T> {
    fn validate_value(&self, value: &T) -> Option<T>;

    fn to_slider_value(&self, value: &T) -> f64;

    fn next(&self, _current: &T) -> Option<T> {
        None
    }

    fn previous(&self, _current: &T) -> Option<T> {
        None
    }

    fn from_slider_value(&self, slider: f64) -> T;

    fn apply_value_immediately(&self) -> bool {
        true
    }
}

/// `IntRange`, with `IntRangeBase`'s methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntRange {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
    pub apply_value_immediately: bool,
}

impl IntRange {
    /// `IntRange(int, int)`: applied as the slider moves.
    pub fn new(min_inclusive: i32, max_inclusive: i32) -> IntRange {
        IntRange {
            min_inclusive,
            max_inclusive,
            apply_value_immediately: true,
        }
    }
}

impl SliderableValueSet<i32> for IntRange {
    fn validate_value(&self, value: &i32) -> Option<i32> {
        (*value >= self.min_inclusive && *value <= self.max_inclusive).then_some(*value)
    }

    fn to_slider_value(&self, value: &i32) -> f64 {
        if *value == self.min_inclusive {
            0.0
        } else if *value == self.max_inclusive {
            1.0
        } else {
            Mth::map_f64(
                f64::from(*value) + 0.5,
                f64::from(self.min_inclusive),
                f64::from(self.max_inclusive) + 1.0,
                0.0,
                1.0,
            )
        }
    }

    fn next(&self, current: &i32) -> Option<i32> {
        Some(current + 1)
    }

    fn previous(&self, current: &i32) -> Option<i32> {
        Some(current - 1)
    }

    fn from_slider_value(&self, slider: f64) -> i32 {
        // 0.99999F, a float.
        let slider = if slider >= 1.0 {
            f64::from(0.99999_f32)
        } else {
            slider
        };
        Mth::floor_f64(Mth::map_f64(
            slider,
            0.0,
            1.0,
            f64::from(self.min_inclusive),
            f64::from(self.max_inclusive) + 1.0,
        ))
    }

    fn apply_value_immediately(&self) -> bool {
        self.apply_value_immediately
    }
}

pub struct OptionInstance<T> {
    caption: Component,
    tooltip: TooltipSupplier<T>,
    to_string: CaptionBasedToString<T>,
    values: ValueSet<T>,
    initial_value: T,
    on_value_update: ValueUpdateListener<T>,
    field: OptionField<T>,
}

impl<T: Clone> Clone for OptionInstance<T> {
    fn clone(&self) -> OptionInstance<T> {
        OptionInstance {
            caption: self.caption.clone(),
            tooltip: Rc::clone(&self.tooltip),
            to_string: Rc::clone(&self.to_string),
            values: self.values.clone(),
            initial_value: self.initial_value.clone(),
            on_value_update: Rc::clone(&self.on_value_update),
            field: self.field.clone(),
        }
    }
}

impl<T: Clone + PartialEq + 'static> OptionInstance<T> {
    /// `OptionInstance(String, TooltipSupplier, CaptionBasedToString, ValueSet, Codec, T,
    /// ValueUpdateListener)`, its value in `field`.
    pub fn new(
        caption_id: &str,
        tooltip: TooltipSupplier<T>,
        to_string: CaptionBasedToString<T>,
        values: ValueSet<T>,
        initial_value: T,
        on_value_update: ValueUpdateListener<T>,
        field: OptionField<T>,
    ) -> OptionInstance<T> {
        OptionInstance {
            caption: Component::translatable(caption_id).into(),
            tooltip,
            to_string,
            values,
            initial_value,
            on_value_update,
            field,
        }
    }

    /// `noTooltip`.
    pub fn no_tooltip() -> TooltipSupplier<T> {
        Rc::new(|_| None)
    }

    /// `cachedConstantTooltip`.
    pub fn cached_constant_tooltip(tooltip_component: Component) -> TooltipSupplier<T> {
        Rc::new(move |_| Some(Tooltip::create(tooltip_component.clone())))
    }

    /// `NO_ACTION`.
    pub fn no_action() -> ValueUpdateListener<T> {
        Rc::new(|_| {})
    }

    /// `createButton(Options)`: 150 wide at the corner.
    pub fn create_button(&self, minecraft: &Weak<Minecraft>) -> WidgetRef {
        self.create_button_at(minecraft, 0, 0, 150)
    }

    /// `createButton(Options, int, int, int)`.
    pub fn create_button_at(
        &self,
        minecraft: &Weak<Minecraft>,
        x: i32,
        y: i32,
        width: i32,
    ) -> WidgetRef {
        self.create_button_with(minecraft, x, y, width, OptionInstance::no_action())
    }

    /// `createButton(Options, int, int, int, ValueUpdateListener)`.
    pub fn create_button_with(
        &self,
        minecraft: &Weak<Minecraft>,
        x: i32,
        y: i32,
        width: i32,
        on_value_changed: ValueUpdateListener<T>,
    ) -> WidgetRef {
        self.values.create_button(
            &self.tooltip,
            minecraft,
            x,
            y,
            width,
            on_value_changed,
            self,
        )
    }

    /// `get`: the value in its field.
    pub fn get(&self, options: &Options) -> T {
        (self.field.get)(options)
    }

    /// `set`: a valid value, or the initial one (which Java logs), told to the listener when it
    /// changed. The game is running whenever a screen sets an option.
    pub fn set(&self, options: &mut Options, value: T) {
        let new_value = self.values.validate_value(&value).unwrap_or_else(|| {
            eprintln!("Illegal option value for {}", self.caption.get_string());
            self.initial_value.clone()
        });
        if self.get(options) != new_value {
            (self.field.set)(options, new_value.clone());
            (self.on_value_update)(&new_value);
        }
    }

    pub fn caption(&self) -> &Component {
        &self.caption
    }

    /// `toString`'s function of a value: how the value reads.
    pub fn value_to_string(&self, value: &T) -> Component {
        (self.to_string)(&self.caption, value)
    }
}

/// `OptionInstanceSliderButton`: a slider over an option's values, which sets the option as it
/// moves, or 600 milliseconds after it stopped for a value set that isn't applied immediately
/// (`Util.getMillis()`, the wall clock).
pub struct OptionInstanceSliderButton<N> {
    slider: AbstractSliderButton,
    minecraft: Weak<Minecraft>,
    instance: OptionInstance<N>,
    values: Rc<dyn SliderableValueSet<N>>,
    tooltip_supplier: TooltipSupplier<N>,
    on_value_changed: ValueUpdateListener<N>,
    delayed_apply_at: Option<Instant>,
    apply_value_immediately: bool,
}

impl<N: Clone + PartialEq + 'static> OptionInstanceSliderButton<N> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        minecraft: Weak<Minecraft>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        instance: OptionInstance<N>,
        values: Rc<dyn SliderableValueSet<N>>,
        tooltip_supplier: TooltipSupplier<N>,
        on_value_changed: ValueUpdateListener<N>,
        apply_value_immediately: bool,
    ) -> OptionInstanceSliderButton<N> {
        let initial_value = {
            let game = minecraft.upgrade().expect("the game outlives its screens");
            let options = game.options.borrow();
            values.to_slider_value(&instance.get(&options))
        };
        let mut button = OptionInstanceSliderButton {
            slider: abstract_option_slider_button::new(x, y, width, height, initial_value),
            minecraft,
            instance,
            values,
            tooltip_supplier,
            on_value_changed,
            delayed_apply_at: None,
            apply_value_immediately,
        };
        button.update_message();
        button
    }

    /// `instance.get()`.
    fn current(&self) -> N {
        let minecraft = self
            .minecraft
            .upgrade()
            .expect("the game outlives its screens");
        let options = minecraft.options.borrow();
        self.instance.get(&options)
    }

    /// `applyUnsavedValue`: the slider's value set on the option, when it's another.
    pub fn apply_unsaved_value(&mut self) {
        let slider_value = self.values.from_slider_value(self.slider.value);
        if slider_value != self.current() {
            let minecraft = self
                .minecraft
                .upgrade()
                .expect("the game outlives its screens");
            let value = {
                let mut options = minecraft.options.borrow_mut();
                self.instance.set(&mut options, slider_value);
                self.instance.get(&options)
            };
            (self.on_value_changed)(&value);
        }
    }

    /// `resetValue`: back to the option's value.
    pub fn reset_value(&mut self) {
        let value = self.values.to_slider_value(&self.current());
        if self.slider.value != value {
            self.slider.value = value;
            self.delayed_apply_at = None;
            self.update_message();
        }
    }
}

impl<N: Clone + PartialEq + 'static> SliderClass for OptionInstanceSliderButton<N> {
    fn slider(&self) -> &AbstractSliderButton {
        &self.slider
    }

    fn slider_mut(&mut self) -> &mut AbstractSliderButton {
        &mut self.slider
    }

    fn update_message(&mut self) {
        let value = self.values.from_slider_value(self.slider.value);
        let message = self.instance.value_to_string(&value);
        self.set_message(message);
        let tooltip = (self.tooltip_supplier)(&value);
        self.slider.widget.set_tooltip(tooltip);
    }

    fn apply_value(&mut self) {
        if self.apply_value_immediately {
            self.apply_unsaved_value();
        } else {
            self.delayed_apply_at = Some(Instant::now() + Duration::from_millis(600));
        }
    }
}

impl<N: Clone + PartialEq + 'static> AbstractWidgetClass for OptionInstanceSliderButton<N> {
    fn widget(&self) -> &AbstractWidget {
        &self.slider.widget
    }

    fn widget_mut(&mut self) -> &mut AbstractWidget {
        &mut self.slider.widget
    }

    /// The slider, then the value applied once its delay is over.
    fn extract_widget_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
        abstract_slider_button::extract_widget_render_state(self, graphics);
        if self
            .delayed_apply_at
            .is_some_and(|apply_at| Instant::now() >= apply_at)
        {
            self.delayed_apply_at = None;
            self.apply_unsaved_value();
            self.reset_value();
        }
    }

    fn on_click(&mut self, event: &MouseButtonEvent, _double_click: bool) {
        abstract_slider_button::on_click(self, event);
    }

    fn on_drag(&mut self, event: &MouseButtonEvent, _dx: f64, _dy: f64) {
        abstract_slider_button::on_drag(self, event);
    }

    fn on_release(&mut self, _event: &MouseButtonEvent) {
        abstract_slider_button::on_release(self);
        if self.apply_value_immediately {
            self.reset_value();
        }
    }

    fn set_focused(&mut self, focused: bool) {
        abstract_slider_button::set_focused(self, focused);
    }

    /// The arrows go to the value set's previous or next value when it has one, and otherwise
    /// move the slider by a pixel.
    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        if event.is_selection() {
            self.slider.can_change_value = !self.slider.can_change_value;
            return true;
        }
        if self.slider.can_change_value {
            let current = self.values.from_slider_value(self.slider.value);
            let step = if event.is_left() {
                self.values.previous(&current)
            } else if event.is_right() {
                self.values.next(&current)
            } else {
                None
            };
            if let Some(value) = step {
                let slider_value = self.values.to_slider_value(&value);
                abstract_slider_button::set_value(self, slider_value);
                return true;
            }
        }
        abstract_slider_button::key_pressed(self, event)
    }
}
