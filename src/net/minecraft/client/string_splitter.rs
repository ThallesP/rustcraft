//! net/minecraft/client/StringSplitter.java
//!
//! Widths of text and where to cut it, by a width for each codepoint in its style (the font's
//! advances). Positions are UTF-16 indexes, as Java's strings count, and the strings a method
//! hands back are cut at them. Splitting a `FormattedText` into lines (`splitLines(FormattedText,
//! …)`, `headByWidth`, with `ComponentCollector` and the line components) comes with the chat,
//! which wraps its messages with it.

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::string_decomposer::StringDecomposer;

/// `StringSplitter.WidthProvider`: a codepoint's width in a style.
pub type WidthProvider = Box<dyn Fn(i32, &Style) -> f32>;

pub struct StringSplitter {
    width_provider: WidthProvider,
}

/// `Character.charCount`.
fn char_count(codepoint: i32) -> i32 {
    if codepoint >= 0x10000 { 2 } else { 1 }
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

impl StringSplitter {
    pub fn new(width_provider: WidthProvider) -> StringSplitter {
        StringSplitter { width_provider }
    }

    /// `stringWidth(String)`: the advances summed, `§` codes applied; 0 for none (Java's null).
    pub fn string_width(&self, str: Option<&str>) -> f32 {
        let Some(str) = str else {
            return 0.0;
        };
        let mut result = 0.0_f32;
        StringDecomposer::iterate_formatted(
            &units(str),
            &Style::EMPTY,
            &mut |_, style, codepoint| {
                result += (self.width_provider)(codepoint, style);
                true
            },
        );
        result
    }

    /// `stringWidth(FormattedText)`.
    pub fn string_width_text(&self, text: &impl FormattedText) -> f32 {
        let mut result = 0.0_f32;
        StringDecomposer::iterate_formatted_text(
            text,
            &Style::EMPTY,
            &mut |_, style, codepoint| {
                result += (self.width_provider)(codepoint, style);
                true
            },
        );
        result
    }

    /// `stringWidth(FormattedCharSequence)`.
    pub fn string_width_sequence(&self, text: &FormattedCharSequence) -> f32 {
        let mut result = 0.0_f32;
        text.accept(&mut |_, style, codepoint| {
            result += (self.width_provider)(codepoint, style);
            true
        });
        result
    }

    /// `plainIndexAtWidth`: how far into the string, with its codes as characters, fits the
    /// width.
    pub fn plain_index_at_width(&self, str: &str, max_width: i32, style: &Style) -> i32 {
        let mut output = WidthLimitedCharSink::new(max_width as f32);
        StringDecomposer::iterate(&units(str), style, &mut |position, style, codepoint| {
            output.accept(self, position, style, codepoint)
        });
        output.position
    }

    pub fn plain_head_by_width(&self, str: &str, max_width: i32, style: &Style) -> String {
        let units = units(str);
        let end = self.plain_index_at_width(str, max_width, style) as usize;
        String::from_utf16_lossy(&units[..end])
    }

    pub fn plain_tail_by_width(&self, str: &str, max_width: i32, style: &Style) -> String {
        let units = units(str);
        let mut current_width = 0.0_f32;
        let mut result = units.len() as i32;
        StringDecomposer::iterate_backwards(&units, style, &mut |position, s, codepoint| {
            current_width += (self.width_provider)(codepoint, s);
            if current_width > max_width as f32 {
                return false;
            }
            result = position;
            true
        });
        String::from_utf16_lossy(&units[result as usize..])
    }

    /// `findLineBreak`: where the line starting the string breaks to fit the width.
    pub fn find_line_break(&self, input: &str, max: i32, initial_style: &Style) -> i32 {
        let mut finder = LineBreakFinder::new(max as f32);
        StringDecomposer::iterate_formatted(
            &units(input),
            initial_style,
            &mut |position, style, codepoint| finder.accept(self, position, style, codepoint),
        );
        finder.get_split_position()
    }

    /// `getWordPosition`: `dir` words on from `from` (back when negative), past the spaces and
    /// newlines around them when `strip_spaces`.
    pub fn get_word_position(text: &str, dir: i32, from: i32, strip_spaces: bool) -> i32 {
        let text = units(text);
        let is_break = |unit: u16| unit == u16::from(b' ') || unit == u16::from(b'\n');
        let mut result = from;
        let reverse = dir < 0;
        let abs = dir.wrapping_abs();
        for _ in 0..abs {
            if reverse {
                while strip_spaces && result > 0 && is_break(text[(result - 1) as usize]) {
                    result -= 1;
                }
                while result > 0 && !is_break(text[(result - 1) as usize]) {
                    result -= 1;
                }
            } else {
                let length = text.len() as i32;
                let index_of = |target: u8| {
                    text.iter()
                        .enumerate()
                        .skip(result.max(0) as usize)
                        .find(|(_, unit)| **unit == u16::from(target))
                        .map_or(-1, |(index, _)| index as i32)
                };
                let index1 = index_of(b' ');
                let index2 = index_of(b'\n');
                result = if index1 == -1 && index2 == -1 {
                    -1
                } else if index1 != -1 && index2 != -1 {
                    index1.min(index2)
                } else if index1 != -1 {
                    index1
                } else {
                    index2
                };
                if result == -1 {
                    result = length;
                } else {
                    while strip_spaces && result < length && is_break(text[result as usize]) {
                        result += 1;
                    }
                }
            }
        }
        result
    }

    /// `splitLines(String, int, Style, boolean, LinePosConsumer)`: each line's style, start and
    /// end, a line ending at a newline, or at the last space (or character) that fits.
    pub fn split_lines_with(
        &self,
        input: &str,
        max_width: i32,
        initial_style: &Style,
        include_all: bool,
        output: &mut dyn FnMut(&Style, i32, i32),
    ) {
        let units = units(input);
        let mut start = 0;
        let size = units.len() as i32;
        let mut work_style = initial_style.clone();
        while start < size {
            let mut finder = LineBreakFinder::new(max_width as f32);
            let end_of_text = StringDecomposer::iterate_formatted_from(
                &units,
                start as usize,
                &work_style,
                initial_style,
                &mut |position, style, codepoint| finder.accept(self, position, style, codepoint),
            );
            if end_of_text {
                output(&work_style, start, size);
                break;
            }
            let line_break = finder.get_split_position();
            let first_tail_char = units[line_break as usize];
            let adjusted_break =
                if first_tail_char != u16::from(b'\n') && first_tail_char != u16::from(b' ') {
                    line_break
                } else {
                    line_break + 1
                };
            output(
                &work_style,
                start,
                if include_all {
                    adjusted_break
                } else {
                    line_break
                },
            );
            start = adjusted_break;
            work_style = finder.line_break_style.clone();
        }
    }

    /// `splitLines(String, int, Style)`: each line's text and style (`FormattedText.of`).
    pub fn split_lines(
        &self,
        input: &str,
        max_width: i32,
        initial_style: &Style,
    ) -> Vec<(String, Style)> {
        let units = units(input);
        let mut result = Vec::new();
        self.split_lines_with(
            input,
            max_width,
            initial_style,
            false,
            &mut |style, start, end| {
                result.push((
                    String::from_utf16_lossy(&units[start as usize..end as usize]),
                    style.clone(),
                ));
            },
        );
        result
    }
}

/// `StringSplitter.LineBreakFinder`.
struct LineBreakFinder {
    max_width: f32,
    line_break: i32,
    line_break_style: Style,
    had_non_zero_width_char: bool,
    width: f32,
    last_space: i32,
    last_space_style: Style,
    next_char: i32,
    offset: i32,
}

impl LineBreakFinder {
    fn new(max_width: f32) -> LineBreakFinder {
        LineBreakFinder {
            max_width: max_width.max(1.0),
            line_break: -1,
            line_break_style: Style::EMPTY,
            had_non_zero_width_char: false,
            width: 0.0,
            last_space: -1,
            last_space_style: Style::EMPTY,
            next_char: 0,
            offset: 0,
        }
    }

    fn accept(
        &mut self,
        splitter: &StringSplitter,
        position: i32,
        style: &Style,
        codepoint: i32,
    ) -> bool {
        let adjusted_position = position + self.offset;
        if codepoint == 10 {
            return self.finish_iteration(adjusted_position, style);
        }
        if codepoint == 32 {
            self.last_space = adjusted_position;
            self.last_space_style = style.clone();
        }
        let char_width = (splitter.width_provider)(codepoint, style);
        self.width += char_width;
        if !self.had_non_zero_width_char || !(self.width > self.max_width) {
            self.had_non_zero_width_char |= char_width != 0.0;
            self.next_char = adjusted_position + char_count(codepoint);
            true
        } else if self.last_space != -1 {
            let style = self.last_space_style.clone();
            self.finish_iteration(self.last_space, &style)
        } else {
            self.finish_iteration(adjusted_position, style)
        }
    }

    fn finish_iteration(&mut self, line_break: i32, style: &Style) -> bool {
        self.line_break = line_break;
        self.line_break_style = style.clone();
        false
    }

    fn get_split_position(&self) -> i32 {
        if self.line_break != -1 {
            self.line_break
        } else {
            self.next_char
        }
    }
}

/// `StringSplitter.WidthLimitedCharSink`.
struct WidthLimitedCharSink {
    max_width: f32,
    position: i32,
}

impl WidthLimitedCharSink {
    fn new(max_width: f32) -> WidthLimitedCharSink {
        WidthLimitedCharSink {
            max_width,
            position: 0,
        }
    }

    fn accept(
        &mut self,
        splitter: &StringSplitter,
        position: i32,
        style: &Style,
        codepoint: i32,
    ) -> bool {
        self.max_width -= (splitter.width_provider)(codepoint, style);
        if self.max_width >= 0.0 {
            self.position = position + char_count(codepoint);
            true
        } else {
            false
        }
    }
}
