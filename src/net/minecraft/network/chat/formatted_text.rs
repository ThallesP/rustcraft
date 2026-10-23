//! net/minecraft/network/chat/FormattedText.java
//!
//! Text that hands its pieces to a consumer, with or without their styles, until the consumer
//! returns a value (`STOP_ITERATION` is `Some`). Java's consumers are functional interfaces; here
//! they're `FnMut`s, and `visit` and `visit_styled` are the two `visit` overloads. `of(String)` is
//! `Of`, a plain string; `of(String, Style)`, `composite` and `EMPTY` come with their callers.

use crate::com::mojang::datafixers::util::unit::Unit;
use crate::net::minecraft::network::chat::style::Style;

pub trait FormattedText {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T>;

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T>;

    /// Every piece, joined.
    fn get_string(&self) -> String {
        let mut builder = String::new();
        self.visit::<Unit>(&mut |contents| {
            builder.push_str(contents);
            None
        });
        builder
    }
}

/// `FormattedText.of(String)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Of(pub String);

impl FormattedText for Of {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        output(&self.0)
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(current_style, &self.0)
    }
}
