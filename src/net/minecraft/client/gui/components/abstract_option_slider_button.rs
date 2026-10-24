//! net/minecraft/client/gui/components/AbstractOptionSliderButton.java
//!
//! A slider over one of the options, with an empty message until its subclass makes one. Java's
//! holds the `Options`; here a slider reaches the game's options through the game it keeps
//! (`OptionInstanceSliderButton`'s), since the options live in the game's cell.

use crate::net::minecraft::client::gui::components::abstract_slider_button::AbstractSliderButton;
use crate::net::minecraft::network::chat::component::Component;

/// `AbstractOptionSliderButton(Options, int, int, int, int, double)`: the slider's fields.
pub fn new(x: i32, y: i32, width: i32, height: i32, initial_value: f64) -> AbstractSliderButton {
    AbstractSliderButton::new(
        x,
        y,
        width,
        height,
        Component::empty().into(),
        initial_value,
    )
}
