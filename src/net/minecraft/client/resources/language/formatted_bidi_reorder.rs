//! net/minecraft/client/resources/language/FormattedBidiReorder.java (with
//! net/minecraft/network/chat/SubStringSource.java)
//!
//! A text in visual order, which `ClientLanguage.getVisualOrder` makes for the font: the text's
//! codepoints with the `§` codes applied (`SubStringSource.create`), cut into ICU's bidirectional
//! runs, each a run of the same style handed on forwards (left to right) or backwards and
//! mirrored (right to left). ICU's `Bidi` and `ArabicShaping` aren't ported, so the text is one
//! left-to-right run, which is what they make of text with no right-to-left script in a
//! left-to-right language; right-to-left text isn't drawn like vanilla's yet. A run's positions
//! start at 0, as each `FormattedCharSequence.forward` does.

use crate::com::mojang::datafixers::util::unit::Unit;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::string_decomposer::StringDecomposer;

pub struct FormattedBidiReorder;

impl FormattedBidiReorder {
    /// `reorder(FormattedText, boolean)`: panics for a right-to-left language.
    pub fn reorder(
        text: &impl FormattedText,
        default_right_to_left: bool,
    ) -> FormattedCharSequence {
        assert!(
            !default_right_to_left,
            "right-to-left languages aren't ported"
        );
        // SubStringSource.create: the plain text, and each UTF-16 unit's style.
        let mut plain_text: Vec<u16> = Vec::new();
        let mut char_styles: Vec<Style> = Vec::new();
        text.visit_styled::<Unit>(
            &mut |style, contents| {
                let units: Vec<u16> = contents.encode_utf16().collect();
                StringDecomposer::iterate_formatted(
                    &units,
                    style,
                    &mut |_, char_style, codepoint| {
                        let mut buffer = [0u16; 2];
                        let encoded = char::from_u32(codepoint as u32)
                            .unwrap_or('\u{FFFD}')
                            .encode_utf16(&mut buffer);
                        for unit in encoded.iter() {
                            plain_text.push(*unit);
                            char_styles.push(char_style.clone());
                        }
                        true
                    },
                );
                None
            },
            &Style::EMPTY,
        );
        // One left-to-right run: substring(0, length, false).
        FormattedCharSequence::composite(sub_string(&plain_text, &char_styles, 0, plain_text.len()))
    }
}

/// `SubStringSource.substring(start, length, false)`: the runs of one style, forwards.
fn sub_string(
    plain_text: &[u16],
    char_styles: &[Style],
    start: usize,
    length: usize,
) -> Vec<FormattedCharSequence> {
    if length == 0 {
        return Vec::new();
    }
    let mut parts = Vec::new();
    let mut current_run_style = &char_styles[start];
    let mut current_run_start = start;
    for i in 1..length {
        let actual_index = start + i;
        let char_style = &char_styles[actual_index];
        if char_style != current_run_style {
            parts.push(FormattedCharSequence::forward_units(
                plain_text[current_run_start..actual_index].to_vec(),
                current_run_style.clone(),
            ));
            current_run_style = char_style;
            current_run_start = actual_index;
        }
    }
    if current_run_start < start + length {
        parts.push(FormattedCharSequence::forward_units(
            plain_text[current_run_start..start + length].to_vec(),
            current_run_style.clone(),
        ));
    }
    parts
}
