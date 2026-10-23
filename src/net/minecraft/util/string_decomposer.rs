//! net/minecraft/util/StringDecomposer.java (with FormattedCharSink.java)
//!
//! A string's codepoints handed to a sink with their positions (UTF-16 indexes, as Java's strings
//! count) and styles, forwards, backwards, or with the `§` codes applied to the style and left
//! out. A lone surrogate is U+FFFD; a Rust string holds none, so only a string cut inside a pair
//! (which the port's callers never make) would show one. `FormattedCharSink` is any
//! `FnMut(position, style, codepoint) -> bool` that returns false to stop.

use crate::com::mojang::datafixers::util::unit::Unit;
use crate::net::minecraft::chat_formatting::ChatFormatting;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

/// `FormattedCharSink`.
pub type FormattedCharSink<'a> = dyn FnMut(i32, &Style, i32) -> bool + 'a;

pub struct StringDecomposer;

const REPLACEMENT_CHAR: i32 = 0xFFFD;

fn is_high_surrogate(ch: u16) -> bool {
    (0xD800..=0xDBFF).contains(&ch)
}

fn is_low_surrogate(ch: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&ch)
}

fn to_code_point(high: u16, low: u16) -> i32 {
    ((i32::from(high) - 0xD800) << 10) + (i32::from(low) - 0xDC00) + 0x10000
}

impl StringDecomposer {
    fn feed_char(style: &Style, output: &mut FormattedCharSink, pos: i32, ch: u16) -> bool {
        if (0xD800..=0xDFFF).contains(&ch) {
            output(pos, style, REPLACEMENT_CHAR)
        } else {
            output(pos, style, i32::from(ch))
        }
    }

    /// `iterate(String, Style, FormattedCharSink)`, over the string's UTF-16 units.
    pub fn iterate(string: &[u16], style: &Style, output: &mut FormattedCharSink) -> bool {
        let size = string.len();
        let mut i = 0;
        while i < size {
            let ch = string[i];
            if is_high_surrogate(ch) {
                if i + 1 >= size {
                    if !output(i as i32, style, REPLACEMENT_CHAR) {
                        return false;
                    }
                    break;
                }
                let low = string[i + 1];
                if is_low_surrogate(low) {
                    if !output(i as i32, style, to_code_point(ch, low)) {
                        return false;
                    }
                    i += 1;
                } else if !output(i as i32, style, REPLACEMENT_CHAR) {
                    return false;
                }
            } else if !StringDecomposer::feed_char(style, output, i as i32, ch) {
                return false;
            }
            i += 1;
        }
        true
    }

    pub fn iterate_backwards(
        string: &[u16],
        style: &Style,
        output: &mut FormattedCharSink,
    ) -> bool {
        let size = string.len() as i32;
        let mut i = size - 1;
        while i >= 0 {
            let ch = string[i as usize];
            if is_low_surrogate(ch) {
                if i - 1 < 0 {
                    if !output(0, style, REPLACEMENT_CHAR) {
                        return false;
                    }
                    break;
                }
                let high = string[(i - 1) as usize];
                if is_high_surrogate(high) {
                    i -= 1;
                    if !output(i, style, to_code_point(high, ch)) {
                        return false;
                    }
                } else if !output(i, style, REPLACEMENT_CHAR) {
                    return false;
                }
            } else if !StringDecomposer::feed_char(style, output, i, ch) {
                return false;
            }
            i -= 1;
        }
        true
    }

    /// `iterateFormatted(String, Style, FormattedCharSink)`.
    pub fn iterate_formatted(
        string: &[u16],
        style: &Style,
        output: &mut FormattedCharSink,
    ) -> bool {
        StringDecomposer::iterate_formatted_from(string, 0, style, style, output)
    }

    /// `iterateFormatted(String, int, Style, Style, FormattedCharSink)`: from `offset`, a `§`
    /// code changing the style (reset back to `reset_style`).
    pub fn iterate_formatted_from(
        string: &[u16],
        offset: usize,
        current_style: &Style,
        reset_style: &Style,
        output: &mut FormattedCharSink,
    ) -> bool {
        let size = string.len();
        let mut style = current_style.clone();
        let mut i = offset;
        while i < size {
            let ch = string[i];
            if ch == 167 {
                if i + 1 >= size {
                    break;
                }
                let code = string[i + 1];
                if let Some(formatting) = ChatFormatting::get_by_code(code) {
                    style = if formatting == ChatFormatting::RESET {
                        reset_style.clone()
                    } else {
                        style.apply_legacy_format(formatting)
                    };
                }
                i += 1;
            } else if is_high_surrogate(ch) {
                if i + 1 >= size {
                    if !output(i as i32, &style, REPLACEMENT_CHAR) {
                        return false;
                    }
                    break;
                }
                let low = string[i + 1];
                if is_low_surrogate(low) {
                    if !output(i as i32, &style, to_code_point(ch, low)) {
                        return false;
                    }
                    i += 1;
                } else if !output(i as i32, &style, REPLACEMENT_CHAR) {
                    return false;
                }
            } else if !StringDecomposer::feed_char(&style, output, i as i32, ch) {
                return false;
            }
            i += 1;
        }
        true
    }

    /// `iterateFormatted(FormattedText, Style, FormattedCharSink)`: each piece in its style.
    pub fn iterate_formatted_text(
        component: &impl FormattedText,
        root_style: &Style,
        output: &mut FormattedCharSink,
    ) -> bool {
        component
            .visit_styled::<Unit>(
                &mut |style, contents| {
                    let units: Vec<u16> = contents.encode_utf16().collect();
                    if StringDecomposer::iterate_formatted_from(&units, 0, style, style, output) {
                        None
                    } else {
                        Some(Unit::INSTANCE)
                    }
                },
                root_style,
            )
            .is_none()
    }

    pub fn filter_broken_surrogates(input: &str) -> String {
        let units: Vec<u16> = input.encode_utf16().collect();
        let mut builder = String::new();
        StringDecomposer::iterate(&units, &Style::EMPTY, &mut |_, _, codepoint| {
            builder.push(char::from_u32(codepoint as u32).unwrap_or('\u{FFFD}'));
            true
        });
        builder
    }

    pub fn get_plain_text(input: &impl FormattedText) -> String {
        let mut builder = String::new();
        StringDecomposer::iterate_formatted_text(input, &Style::EMPTY, &mut |_, _, codepoint| {
            builder.push(char::from_u32(codepoint as u32).unwrap_or('\u{FFFD}'));
            true
        });
        builder
    }
}
