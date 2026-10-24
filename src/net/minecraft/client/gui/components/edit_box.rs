//! net/minecraft/client/gui/components/EditBox.java
//!
//! A line of text to edit: its value (UTF-16 units, as Java's string's indexes count), the cursor
//! and the selection, the part shown when it's wider than the box, typing, deleting, moving by
//! characters or words, a suggestion after the cursor and a hint when it's empty. The cursor blinks
//! by the wall clock from when the box got the focus, like Java's. Its responder, which Java calls
//! as the value changes, is a flag the owner reads after the input it handled (`take_changed`),
//! since the owner is what handles the input; formatters are functions of the text and its offset.
//!
//! Not ported yet: the clipboard (copy, cut and paste), the input method's preedit overlay, the
//! text input area the window is told, narration, and the cursor's shape.

use std::rc::Rc;

use crate::net::minecraft::chat_formatting::ChatFormatting;
use crate::net::minecraft::client::gui::components::abstract_widget::{
    AbstractWidget, AbstractWidgetClass,
};
use crate::net::minecraft::client::gui::components::widget_sprites::WidgetSprites;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input::mouse_button_event::MouseButtonEvent;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::string_util::StringUtil;
use crate::net::minecraft::util::util::Util;

/// `EditBox.TextFormatter`: the text from an offset of the value, formatted, or none for the next
/// formatter.
pub type TextFormatter = Box<dyn Fn(&str, i32) -> Option<FormattedCharSequence>>;

pub struct EditBox {
    widget: AbstractWidget,
    font: Rc<Font>,
    value: Vec<u16>,
    max_length: i32,
    bordered: bool,
    can_lose_focus: bool,
    is_editable: bool,
    centered: bool,
    text_shadow: bool,
    invert_highlighted_text_color: bool,
    display_pos: i32,
    cursor_pos: i32,
    highlight_pos: i32,
    text_color: i32,
    text_color_uneditable: i32,
    suggestion: Option<String>,
    /// Whether the value changed since the owner last asked (Java's responder's calls).
    changed: bool,
    formatters: Vec<TextFormatter>,
    hint: Option<Component>,
    focused_time: i64,
    text_x: i32,
    text_y: i32,
}

impl EditBox {
    pub const BACKWARDS: i32 = -1;
    pub const FORWARDS: i32 = 1;
    pub const DEFAULT_TEXT_COLOR: i32 = -2039584;

    pub fn new(
        font: Rc<Font>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        narration: Component,
    ) -> EditBox {
        let mut edit_box = EditBox {
            widget: AbstractWidget::new(x, y, width, height, narration),
            font,
            value: Vec::new(),
            max_length: 32,
            bordered: true,
            can_lose_focus: true,
            is_editable: true,
            centered: false,
            text_shadow: true,
            invert_highlighted_text_color: true,
            display_pos: 0,
            cursor_pos: 0,
            highlight_pos: 0,
            text_color: -2039584,
            text_color_uneditable: -9408400,
            suggestion: None,
            changed: false,
            formatters: Vec::new(),
            hint: None,
            focused_time: Util::get_millis(),
            text_x: 0,
            text_y: 0,
        };
        edit_box.update_text_position();
        edit_box
    }

    fn sprites() -> WidgetSprites {
        WidgetSprites::focused(
            Identifier::with_default_namespace("widget/text_field"),
            Identifier::with_default_namespace("widget/text_field_highlighted"),
        )
    }

    fn string(units: &[u16]) -> String {
        String::from_utf16_lossy(units)
    }

    fn len(&self) -> i32 {
        self.value.len() as i32
    }

    /// Whether the value changed since the last call: the responder's calls.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    pub fn add_formatter(&mut self, formatter: TextFormatter) {
        self.formatters.push(formatter);
    }

    pub fn set_value(&mut self, value: &str) {
        let units: Vec<u16> = value.encode_utf16().collect();
        self.value = if units.len() as i32 > self.max_length {
            units[..self.max_length as usize].to_vec()
        } else {
            units
        };
        self.move_cursor_to_end(false);
        self.set_highlight_pos(self.cursor_pos);
        self.on_value_change();
    }

    pub fn get_value(&self) -> String {
        EditBox::string(&self.value)
    }

    pub fn get_highlighted(&self) -> String {
        let start = self.cursor_pos.min(self.highlight_pos) as usize;
        let end = self.cursor_pos.max(self.highlight_pos) as usize;
        EditBox::string(&self.value[start..end])
    }

    /// `insertText`: the filtered text over the selection, as much as fits.
    pub fn insert_text(&mut self, input: &str) {
        let start = self.cursor_pos.min(self.highlight_pos);
        let end = self.cursor_pos.max(self.highlight_pos);
        let mut max_insertion_length = self.max_length - self.len() - (start - end);
        if max_insertion_length > 0 {
            let mut text: Vec<u16> = StringUtil::filter_text(input).encode_utf16().collect();
            let mut insertion_length = text.len() as i32;
            if max_insertion_length < insertion_length {
                if is_high_surrogate(text[(max_insertion_length - 1) as usize]) {
                    max_insertion_length -= 1;
                }
                text.truncate(max_insertion_length as usize);
                insertion_length = max_insertion_length;
            }
            self.value.splice(start as usize..end as usize, text);
            self.set_cursor_position(start + insertion_length);
            self.set_highlight_pos(self.cursor_pos);
            self.on_value_change();
        }
    }

    fn on_value_change(&mut self) {
        self.changed = true;
        self.update_text_position();
    }

    fn delete_text(&mut self, dir: i32, whole_word: bool) {
        if whole_word {
            self.delete_words(dir);
        } else {
            self.delete_chars(dir);
        }
    }

    pub fn delete_words(&mut self, dir: i32) {
        if !self.value.is_empty() {
            if self.highlight_pos != self.cursor_pos {
                self.insert_text("");
            } else {
                let position = self.get_word_position(dir);
                self.delete_chars_to_pos(position);
            }
        }
    }

    pub fn delete_chars(&mut self, dir: i32) {
        let position = self.get_cursor_pos(dir);
        self.delete_chars_to_pos(position);
    }

    pub fn delete_chars_to_pos(&mut self, pos: i32) {
        if !self.value.is_empty() {
            if self.highlight_pos != self.cursor_pos {
                self.insert_text("");
            } else {
                let start = pos.min(self.cursor_pos);
                let end = pos.max(self.cursor_pos);
                if start != end {
                    self.value.drain(start as usize..end as usize);
                    self.set_cursor_position(start);
                    self.on_value_change();
                    self.move_cursor_to(start, false);
                }
            }
        }
    }

    pub fn get_word_position(&self, dir: i32) -> i32 {
        self.get_word_position_from(dir, self.cursor_pos, true)
    }

    fn get_word_position_from(&self, dir: i32, from: i32, strip_spaces: bool) -> i32 {
        let space = u16::from(b' ');
        let mut result = from;
        let reverse = dir < 0;
        for _ in 0..dir.abs() {
            if !reverse {
                let length = self.len();
                match self.value[result.min(length) as usize..]
                    .iter()
                    .position(|&unit| unit == space)
                {
                    None => result = length,
                    Some(offset) => {
                        result += offset as i32;
                        while strip_spaces
                            && result < length
                            && self.value[result as usize] == space
                        {
                            result += 1;
                        }
                    }
                }
            } else {
                while strip_spaces && result > 0 && self.value[(result - 1) as usize] == space {
                    result -= 1;
                }
                while result > 0 && self.value[(result - 1) as usize] != space {
                    result -= 1;
                }
            }
        }
        result
    }

    pub fn move_cursor(&mut self, dir: i32, has_shift_down: bool) {
        let position = self.get_cursor_pos(dir);
        self.move_cursor_to(position, has_shift_down);
    }

    /// `getCursorPos`: `Util.offsetByCodepoints` from the cursor.
    fn get_cursor_pos(&self, dir: i32) -> i32 {
        let length = self.len();
        let mut pos = self.cursor_pos;
        if dir >= 0 {
            let mut i = 0;
            while pos < length && i < dir {
                pos += 1;
                if is_high_surrogate(self.value[(pos - 1) as usize])
                    && pos < length
                    && is_low_surrogate(self.value[pos as usize])
                {
                    pos += 1;
                }
                i += 1;
            }
        } else {
            let mut i = dir;
            while pos > 0 && i < 0 {
                pos -= 1;
                if is_low_surrogate(self.value[pos as usize])
                    && pos > 0
                    && is_high_surrogate(self.value[(pos - 1) as usize])
                {
                    pos -= 1;
                }
                i += 1;
            }
        }
        pos
    }

    pub fn move_cursor_to(&mut self, pos: i32, extend_selection: bool) {
        self.set_cursor_position(pos);
        if !extend_selection {
            self.set_highlight_pos(self.cursor_pos);
        }
        self.update_text_position();
    }

    pub fn set_cursor_position(&mut self, pos: i32) {
        self.cursor_pos = Mth::clamp_i32(pos, 0, self.len());
        self.scroll_to(self.cursor_pos);
    }

    pub fn move_cursor_to_start(&mut self, has_shift_down: bool) {
        self.move_cursor_to(0, has_shift_down);
    }

    pub fn move_cursor_to_end(&mut self, has_shift_down: bool) {
        self.move_cursor_to(self.len(), has_shift_down);
    }

    pub fn can_consume_input(&self) -> bool {
        self.is_active() && self.is_focused() && self.is_editable
    }

    /// `findClickedPositionInText`.
    fn find_clicked_position_in_text(&self, event: &MouseButtonEvent) -> i32 {
        let position_in_text = (Mth::floor_f64(event.x) - self.text_x).min(self.get_inner_width());
        let displayed = EditBox::string(&self.value[self.display_pos as usize..]);
        self.display_pos
            + self
                .font
                .plain_substr_by_width(&displayed, position_in_text)
                .encode_utf16()
                .count() as i32
    }

    fn select_word(&mut self, event: &MouseButtonEvent) {
        let clicked_position = self.find_clicked_position_in_text(event);
        let word_start = self.get_word_position_from(-1, clicked_position, true);
        let word_end = self.get_word_position_from(1, clicked_position, true);
        self.move_cursor_to(word_start, false);
        self.move_cursor_to(word_end, true);
    }

    /// `applyFormat`: the first formatter's, else the text plain.
    fn apply_format(&self, text: &str, offset: i32) -> FormattedCharSequence {
        self.formatters
            .iter()
            .find_map(|formatter| formatter(text, offset))
            .unwrap_or_else(|| FormattedCharSequence::forward(text, Style::EMPTY))
    }

    fn update_text_position(&mut self) {
        let displayed = self.font.plain_substr_by_width(
            &EditBox::string(&self.value[self.display_pos as usize..]),
            self.get_inner_width(),
        );
        self.text_x = self.widget.get_x()
            + if self.centered {
                (self.get_width() - self.font.width(&displayed)) / 2
            } else if self.bordered {
                4
            } else {
                0
            };
        self.text_y = if self.bordered {
            self.widget.get_y() + (self.widget.height - 8) / 2
        } else {
            self.widget.get_y()
        };
    }

    pub fn set_max_length(&mut self, max_length: i32) {
        self.max_length = max_length;
        if self.len() > max_length {
            self.value.truncate(max_length as usize);
            self.on_value_change();
        }
    }

    pub fn get_cursor_position(&self) -> i32 {
        self.cursor_pos
    }

    pub fn is_bordered(&self) -> bool {
        self.bordered
    }

    pub fn set_bordered(&mut self, bordered: bool) {
        self.bordered = bordered;
        self.update_text_position();
    }

    pub fn set_text_color(&mut self, text_color: i32) {
        self.text_color = text_color;
    }

    pub fn set_editable(&mut self, is_editable: bool) {
        self.is_editable = is_editable;
    }

    pub fn set_centered(&mut self, centered: bool) {
        self.centered = centered;
        self.update_text_position();
    }

    pub fn set_text_shadow(&mut self, text_shadow: bool) {
        self.text_shadow = text_shadow;
    }

    pub fn get_inner_width(&self) -> i32 {
        if self.bordered {
            self.widget.width - 8
        } else {
            self.widget.width
        }
    }

    pub fn set_highlight_pos(&mut self, pos: i32) {
        self.highlight_pos = Mth::clamp_i32(pos, 0, self.len());
        self.scroll_to(self.highlight_pos);
    }

    /// `scrollTo`: the part shown moved so `pos` is in it.
    fn scroll_to(&mut self, pos: i32) {
        self.display_pos = self.display_pos.min(self.len());
        let inner_width = self.get_inner_width();
        let displayed = self.font.plain_substr_by_width(
            &EditBox::string(&self.value[self.display_pos as usize..]),
            inner_width,
        );
        let last_pos = displayed.encode_utf16().count() as i32 + self.display_pos;
        if pos == self.display_pos {
            self.display_pos -= self
                .font
                .plain_substr_by_width_reverse(&EditBox::string(&self.value), inner_width, true)
                .encode_utf16()
                .count() as i32;
        }
        if pos > last_pos {
            self.display_pos += pos - last_pos;
        } else if pos <= self.display_pos {
            // Java's `displayPos - (displayPos - pos)`.
            #[allow(clippy::misrefactored_assign_op)]
            {
                self.display_pos -= self.display_pos - pos;
            }
        }
        self.display_pos = Mth::clamp_i32(self.display_pos, 0, self.len());
    }

    pub fn set_can_lose_focus(&mut self, can_lose_focus: bool) {
        self.can_lose_focus = can_lose_focus;
    }

    pub fn set_suggestion(&mut self, suggestion: Option<String>) {
        self.suggestion = suggestion;
    }

    /// `getScreenX`.
    pub fn get_screen_x(&self, char_index: i32) -> i32 {
        if char_index > self.len() {
            self.widget.get_x()
        } else {
            self.widget.get_x()
                + self
                    .font
                    .width(&EditBox::string(&self.value[..char_index as usize]))
        }
    }

    /// `setHint`: grey when it has no style of its own.
    pub fn set_hint(&mut self, hint: Component) {
        let has_no_style = *hint.get_style() == Style::EMPTY;
        self.hint = Some(if has_no_style {
            let mut hint = hint.copy();
            hint.set_style(Style::EMPTY.with_color_formatting(ChatFormatting::DARK_GRAY));
            hint.into()
        } else {
            hint
        });
    }
}

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

impl AbstractWidgetClass for EditBox {
    fn widget(&self) -> &AbstractWidget {
        &self.widget
    }

    fn widget_mut(&mut self) -> &mut AbstractWidget {
        &mut self.widget
    }

    fn set_x(&mut self, x: i32) {
        self.widget.set_x(x);
        self.update_text_position();
    }

    fn set_y(&mut self, y: i32) {
        self.widget.set_y(y);
        self.update_text_position();
    }

    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        if !(self.is_active() && self.is_focused()) {
            return false;
        }
        match event.shortcut_key() {
            8 => {
                if self.is_editable {
                    self.delete_text(-1, event.has_control_down_with_quirk());
                }
                true
            }
            127 => {
                if self.is_editable {
                    self.delete_text(1, event.has_control_down_with_quirk());
                }
                true
            }
            1073741898 => {
                self.move_cursor_to_start(event.has_shift_down());
                true
            }
            1073741901 => {
                self.move_cursor_to_end(event.has_shift_down());
                true
            }
            1073741903 => {
                if event.has_control_down_with_quirk() {
                    let position = self.get_word_position(1);
                    self.move_cursor_to(position, event.has_shift_down());
                } else {
                    self.move_cursor(1, event.has_shift_down());
                }
                true
            }
            1073741904 => {
                if event.has_control_down_with_quirk() {
                    let position = self.get_word_position(-1);
                    self.move_cursor_to(position, event.has_shift_down());
                } else {
                    self.move_cursor(-1, event.has_shift_down());
                }
                true
            }
            _ => {
                if event.is_select_all() {
                    self.move_cursor_to_end(false);
                    self.set_highlight_pos(0);
                    true
                } else {
                    // The clipboard's copy, cut and paste aren't ported.
                    event.is_copy() || event.is_paste() || event.is_cut()
                }
            }
        }
    }

    fn captures_input(&self) -> bool {
        self.can_consume_input()
    }

    fn char_typed(&mut self, event: &CharacterEvent) -> bool {
        if !self.can_consume_input() {
            return false;
        }
        if event.is_allowed_chat_character() {
            if self.is_editable {
                self.insert_text(&event.codepoint_as_string());
            }
            true
        } else {
            false
        }
    }

    fn on_click(&mut self, event: &MouseButtonEvent, double_click: bool) {
        if double_click {
            self.select_word(event);
        } else {
            let position = self.find_clicked_position_in_text(event);
            self.move_cursor_to(position, event.button_info.modifiers & 3 != 0);
        }
    }

    fn on_drag(&mut self, event: &MouseButtonEvent, _dx: f64, _dy: f64) {
        let position = self.find_clicked_position_in_text(event);
        self.move_cursor_to(position, true);
    }

    fn extract_widget_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        _mouse_x: i32,
        _mouse_y: i32,
        _a: f32,
    ) {
        if !self.widget.visible {
            return;
        }
        if self.bordered {
            let sprite = EditBox::sprites()
                .get(self.is_active(), self.is_focused())
                .clone();
            graphics.blit_sprite(
                &RenderPipelines::get().GUI_TEXTURED,
                &sprite,
                self.widget.get_x(),
                self.widget.get_y(),
                self.get_width(),
                self.get_height(),
            );
        }

        let font = Rc::clone(&self.font);
        let color = if self.is_editable {
            self.text_color
        } else {
            self.text_color_uneditable
        };
        let rel_cursor_pos = self.cursor_pos - self.display_pos;
        let displayed: Vec<u16> = font
            .plain_substr_by_width(
                &EditBox::string(&self.value[self.display_pos as usize..]),
                self.get_inner_width(),
            )
            .encode_utf16()
            .collect();
        let displayed_length = displayed.len() as i32;
        let cursor_on_screen = rel_cursor_pos >= 0 && rel_cursor_pos <= displayed_length;
        let show_cursor = self.is_focused()
            && (Util::get_millis() - self.focused_time) / 300 % 2 == 0
            && cursor_on_screen;
        let mut draw_x = self.text_x;
        let rel_highlight_pos =
            Mth::clamp_i32(self.highlight_pos - self.display_pos, 0, displayed_length);
        if !displayed.is_empty() {
            let half = if cursor_on_screen {
                &displayed[..rel_cursor_pos as usize]
            } else {
                &displayed[..]
            };
            let char_sequence = self.apply_format(&EditBox::string(half), self.display_pos);
            graphics.text_sequence(
                &font,
                &char_sequence,
                draw_x,
                self.text_y,
                color,
                self.text_shadow,
            );
            draw_x += font.width_sequence(&char_sequence) + 1;
        }

        let insert = self.cursor_pos < self.len() || self.len() >= self.max_length;
        let mut cursor_x = draw_x;
        if !cursor_on_screen {
            cursor_x = if rel_cursor_pos > 0 {
                self.text_x + self.widget.width
            } else {
                self.text_x
            };
        } else if insert {
            cursor_x -= 1;
            draw_x -= 1;
        }

        if !displayed.is_empty() && cursor_on_screen && rel_cursor_pos < displayed_length {
            let rest = self.apply_format(
                &EditBox::string(&displayed[rel_cursor_pos as usize..]),
                self.cursor_pos,
            );
            graphics.text_sequence(&font, &rest, draw_x, self.text_y, color, self.text_shadow);
        }

        if let Some(hint) = &self.hint
            && displayed.is_empty()
            && !self.is_focused()
        {
            graphics.text_component(&font, hint, draw_x, self.text_y, color, true);
        }

        if !insert && let Some(suggestion) = &self.suggestion {
            graphics.text_str(
                &font,
                Some(suggestion),
                cursor_x - 1,
                self.text_y,
                -8355712,
                self.text_shadow,
            );
        }

        if rel_highlight_pos != rel_cursor_pos {
            let highlight_x = self.text_x
                + font.width(&EditBox::string(&displayed[..rel_highlight_pos as usize]));
            let right = self.widget.get_x() + self.widget.width;
            graphics.text_highlight(
                cursor_x.min(right),
                self.text_y - 1,
                (highlight_x - 1).min(right),
                self.text_y + 1 + 9,
                self.invert_highlighted_text_color,
            );
        }

        if show_cursor {
            if insert {
                // TextCursorUtils.extractInsertCursor.
                graphics.fill(
                    cursor_x,
                    self.text_y - 1,
                    cursor_x + 1,
                    self.text_y + 9 + 1,
                    color,
                );
            } else {
                // TextCursorUtils.extractAppendCursor.
                graphics.text_str(
                    &font,
                    Some("_"),
                    cursor_x,
                    self.text_y,
                    color,
                    self.text_shadow,
                );
            }
        }
    }

    /// `setFocused`: the cursor's blink starts over. A box that can't lose the focus keeps it.
    fn set_focused(&mut self, focused: bool) {
        if self.can_lose_focus || focused {
            self.widget.set_focused(focused);
            if focused {
                self.focused_time = Util::get_millis();
            }
        }
    }
}
