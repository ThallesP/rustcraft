//! net/minecraft/util/FormattedCharSequence.java
//!
//! Text as codepoints in their styles, handed to a sink in visual order: what the font draws.
//! Java's interface is made of lambdas (`forward`, `backward`, `codepoint`, `composite`); here
//! they're an enum's variants, a string's being its UTF-16 units. `forward`'s and `backward`'s
//! overloads with an `Int2IntFunction` (mirroring right-to-left text) come with bidirectional text.

use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::string_decomposer::{FormattedCharSink, StringDecomposer};

#[derive(Clone, Debug)]
pub enum FormattedCharSequence {
    Empty,
    Codepoint(i32, Style),
    Forward(Vec<u16>, Style),
    Backward(Vec<u16>, Style),
    Composite(Vec<FormattedCharSequence>),
}

impl FormattedCharSequence {
    pub const EMPTY: FormattedCharSequence = FormattedCharSequence::Empty;

    pub fn accept(&self, output: &mut FormattedCharSink) -> bool {
        match self {
            FormattedCharSequence::Empty => true,
            FormattedCharSequence::Codepoint(codepoint, style) => output(0, style, *codepoint),
            FormattedCharSequence::Forward(text, style) => {
                StringDecomposer::iterate(text, style, output)
            }
            FormattedCharSequence::Backward(text, style) => {
                StringDecomposer::iterate_backwards(text, style, output)
            }
            FormattedCharSequence::Composite(parts) => parts.iter().all(|part| part.accept(output)),
        }
    }

    pub fn codepoint(codepoint: i32, style: Style) -> FormattedCharSequence {
        FormattedCharSequence::Codepoint(codepoint, style)
    }

    pub fn forward(plain_text: &str, style: Style) -> FormattedCharSequence {
        FormattedCharSequence::forward_units(plain_text.encode_utf16().collect(), style)
    }

    pub fn forward_units(plain_text: Vec<u16>, style: Style) -> FormattedCharSequence {
        if plain_text.is_empty() {
            FormattedCharSequence::Empty
        } else {
            FormattedCharSequence::Forward(plain_text, style)
        }
    }

    pub fn backward(plain_text: &str, style: Style) -> FormattedCharSequence {
        let units: Vec<u16> = plain_text.encode_utf16().collect();
        if units.is_empty() {
            FormattedCharSequence::Empty
        } else {
            FormattedCharSequence::Backward(units, style)
        }
    }

    /// `composite(List)`: none is `EMPTY`, one is itself, more are a list.
    pub fn composite(mut parts: Vec<FormattedCharSequence>) -> FormattedCharSequence {
        match parts.len() {
            0 => FormattedCharSequence::Empty,
            1 => parts.remove(0),
            _ => FormattedCharSequence::Composite(parts),
        }
    }
}
