//! net/minecraft/client/gui/components/ComponentRenderUtils.java
//!
//! A chat message's lines (`wrapComponents`: its pieces, their `§` codes kept or stripped as the
//! `chatColors` option says, split at the width, each line after the first that's the rest of a
//! wrapped one indented by a space) and text clipped with an ellipsis (`clipText`). Java reads the
//! option from `Minecraft.getInstance()`; here it's an argument.

use crate::net::minecraft::client::component_collector::ComponentCollector;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::formatted_text::{Composite, FormattedText, OfStyled};
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

pub struct ComponentRenderUtils;

impl ComponentRenderUtils {
    /// `stripColor`: the text, or without its `§` codes when chat colors are off
    /// (`ChatFormatting.stripFormatting`, `(?i)§[0-9A-FK-OR]`).
    fn strip_color(input: &str, chat_colors: bool) -> String {
        if chat_colors {
            return input.to_string();
        }
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '§'
                && chars.peek().is_some_and(|code| {
                    matches!(code.to_ascii_lowercase(), '0'..='9' | 'a'..='f' | 'k'..='o' | 'r')
                })
            {
                chars.next();
            } else {
                result.push(c);
            }
        }
        result
    }

    /// `wrapComponents`.
    pub fn wrap_components(
        message: &impl FormattedText,
        max_width: i32,
        font: &Font,
        chat_colors: bool,
    ) -> Vec<FormattedCharSequence> {
        let mut collector = ComponentCollector::new();
        message.visit_styled::<()>(
            &mut |style, contents| {
                collector.append(OfStyled(
                    ComponentRenderUtils::strip_color(contents, chat_colors),
                    style.clone(),
                ));
                None
            },
            &Style::EMPTY,
        );
        let language = Language::get_instance();
        let mut result = Vec::new();
        font.get_splitter().split_lines_text_with(
            &collector.get_result_or_empty(),
            max_width,
            &Style::EMPTY,
            &mut |text, wrapped| {
                let reordered_text = language.get_visual_order(&text);
                result.push(if wrapped {
                    FormattedCharSequence::composite(vec![
                        FormattedCharSequence::codepoint(32, Style::EMPTY),
                        reordered_text,
                    ])
                } else {
                    reordered_text
                });
            },
        );
        if result.is_empty() {
            vec![FormattedCharSequence::Empty]
        } else {
            result
        }
    }

    /// `clipText`: as much of the text as fits with `CommonComponents.ELLIPSIS` after it.
    pub fn clip_text(text: &impl FormattedText, font: &Font, width: i32) -> FormattedCharSequence {
        let ellipsis = "...";
        let mut clipped_text = font.get_splitter().head_by_width_text(
            text,
            width - font.width(ellipsis),
            &Style::EMPTY,
        );
        clipped_text
            .0
            .push(OfStyled(ellipsis.to_string(), Style::EMPTY));
        Language::get_instance().get_visual_order(&Composite(clipped_text.0))
    }
}
