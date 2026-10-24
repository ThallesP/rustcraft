//! net/minecraft/client/gui/screens/ChatScreen.java
//!
//! The chat opened with T or /: an edit box at the bottom over the chat's every line, which Enter
//! sends (a command, its slash dropped, or a message), the up and down keys going through the
//! messages sent, the page keys scrolling the chat, and the text kept as a draft when the screen is
//! closed by something else.
//!
//! Not ported yet: command suggestions (`CommandSuggestions`: the command tree from the server's
//! `commands` packet, the suggestions it asks the server for, the usage it draws), which leave
//! every input allowed; sending chat messages (signed chat, `ServerboundChatPacket`), which logs
//! that it isn't ported; clicks on the chat's text (its click events and insertions); the chat's
//! restrictions (`ChatAbilities`: the player has none); narration.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::net::minecraft::chat_formatting::ChatFormatting;
use crate::net::minecraft::client::gui::components::chat_component::{ChatContext, DisplayMode};
use crate::net::minecraft::client::gui::components::edit_box::EditBox;
use crate::net::minecraft::client::gui::components::events::gui_event_listener::ListenerRef;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::screen::{self, Screen, ScreenClass};
use crate::net::minecraft::client::input::character_event::CharacterEvent;
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::string_util::StringUtil;
use crate::org::apache::commons::lang3::string_utils::StringUtils;

/// `ChatScreen.ExitReason`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExitReason {
    INTENTIONAL,
    INTERRUPTED,
    DONE,
}

pub struct ChatScreen {
    screen: Screen,
    close_on_submit: bool,
    history_buffer: String,
    history_pos: i32,
    input: Option<Rc<RefCell<EditBox>>>,
    initial: String,
    /// `isDraft`, which the input's formatter reads too.
    is_draft: Rc<Cell<bool>>,
    display_mode: DisplayMode,
    exit_reason: ExitReason,
}

impl ChatScreen {
    pub const MOUSE_SCROLL_SPEED: f64 = 7.0;

    pub fn new(minecraft: &Rc<Minecraft>, initial: String, is_draft: bool) -> ChatScreen {
        ChatScreen {
            screen: Screen::new(
                minecraft,
                Component::translatable("chat_screen.title").into(),
            ),
            close_on_submit: true,
            history_buffer: String::new(),
            history_pos: -1,
            input: None,
            initial,
            is_draft: Rc::new(Cell::new(is_draft)),
            display_mode: DisplayMode::FOREGROUND,
            exit_reason: ExitReason::INTERRUPTED,
        }
    }

    fn input(&self) -> &Rc<RefCell<EditBox>> {
        self.input.as_ref().expect("made by init")
    }

    /// The input's responder, `onEdited`, for the changes it made while it handled an event.
    fn after_input(&mut self) {
        let changed = self
            .input
            .as_ref()
            .is_some_and(|input| input.borrow_mut().take_changed());
        if changed {
            self.is_draft.set(false);
        }
    }

    fn should_discard_draft(&self) -> bool {
        // saveChatDrafts is off by default.
        self.exit_reason != ExitReason::INTERRUPTED
    }

    /// `moveInHistory`.
    pub fn move_in_history(&mut self, dir: i32) {
        let minecraft = self.screen.minecraft();
        let recent_chat: Vec<String> = minecraft
            .gui
            .borrow()
            .hud
            .chat
            .get_recent_chat()
            .iter()
            .cloned()
            .collect();
        let max = recent_chat.len() as i32;
        let new_pos = Mth::clamp_i32(self.history_pos + dir, 0, max);
        if new_pos != self.history_pos {
            if new_pos == max {
                self.history_pos = max;
                let buffer = self.history_buffer.clone();
                self.input().borrow_mut().set_value(&buffer);
            } else {
                if self.history_pos == max {
                    let value = self.input().borrow().get_value();
                    self.history_buffer = value;
                }
                self.input()
                    .borrow_mut()
                    .set_value(&recent_chat[new_pos as usize]);
                self.history_pos = new_pos;
            }
        }
    }

    /// `handleChatInput`: the message normalized and sent, a command without its slash.
    pub fn handle_chat_input(&self, msg: &str, add_to_recent: bool) {
        let msg = ChatScreen::normalize_chat_message(msg);
        if msg.is_empty() {
            return;
        }
        let minecraft = self.screen.minecraft();
        if add_to_recent {
            minecraft.gui.borrow_mut().hud.chat.add_recent_chat(&msg);
        }
        let Some(connection) = minecraft.get_connection() else {
            return;
        };
        match msg.strip_prefix('/') {
            Some(command) => connection.borrow().send_command(command),
            None => eprintln!("sending a chat message isn't ported yet: {msg}"),
        }
    }

    /// `normalizeChatMessage`.
    pub fn normalize_chat_message(message: &str) -> String {
        let trimmed = crate::org::apache::commons::lang3::string_utils::java_trim(
            &message.encode_utf16().collect::<Vec<_>>(),
        )
        .to_vec();
        StringUtil::trim_chat_message(&StringUtils::normalize_space(&String::from_utf16_lossy(
            &trimmed,
        )))
    }
}

impl ScreenClass for ChatScreen {
    fn screen(&self) -> &Screen {
        &self.screen
    }

    fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn init(&mut self) {
        let minecraft = self.screen.minecraft();
        self.history_pos = minecraft.gui.borrow().hud.chat.get_recent_chat().len() as i32;
        let mut input = EditBox::new(
            minecraft.font_filter_fishy(),
            4,
            self.screen.height - 12,
            self.screen.width - 4,
            12,
            Component::translatable("chat.editBox").into(),
        );
        input.set_max_length(256);
        input.set_bordered(false);
        input.set_value(&self.initial);
        // The responder is set after the value: the initial value isn't an edit.
        input.take_changed();
        let is_draft = Rc::clone(&self.is_draft);
        input.add_formatter(Box::new(move |text, _offset| {
            is_draft.get().then(|| {
                FormattedCharSequence::forward(
                    text,
                    Style::EMPTY
                        .with_color_formatting(ChatFormatting::GRAY)
                        .with_italic(Some(true)),
                )
            })
        }));
        input.set_can_lose_focus(false);
        let input = Rc::new(RefCell::new(input));
        self.screen.add_renderable_widget(Rc::clone(&input));
        self.input = Some(input);
        // The player's chat abilities have no restrictions.
        self.display_mode = DisplayMode::FOREGROUND;
    }

    /// `setInitialFocus`: the input.
    fn set_initial_focus(&mut self) {
        let input: ListenerRef = self.input().clone();
        self.screen.set_initial_focus_on(&input);
    }

    fn on_close(&mut self) {
        self.exit_reason = ExitReason::INTENTIONAL;
        self.screen.minecraft().set_screen(None);
    }

    fn removed(&mut self) {
        let minecraft = self.screen.minecraft();
        let mut gui = minecraft.gui.borrow_mut();
        gui.hud.chat.reset_chat_scroll();
        let value = self.input().borrow().get_value();
        self.initial = value;
        if self.should_discard_draft() || StringUtils::is_blank(&self.initial) {
            gui.hud.chat.discard_draft();
        } else if !self.is_draft.get() {
            gui.hud.chat.save_as_draft(&self.initial);
        }
    }

    fn key_pressed(&mut self, event: &KeyEvent) -> bool {
        if self.is_draft.get() && event.shortcut_key() == 8 {
            self.input().borrow_mut().set_value("");
            self.input().borrow_mut().take_changed();
            self.is_draft.set(false);
            return true;
        }
        let handled = screen::key_pressed(self, event);
        self.after_input();
        if handled {
            return true;
        }

        if event.is_confirmation() {
            let value = self.input().borrow().get_value();
            self.handle_chat_input(&value, true);
            if self.close_on_submit {
                self.exit_reason = ExitReason::DONE;
                self.screen.minecraft().set_screen(None);
            } else {
                self.input().borrow_mut().set_value("");
                self.after_input();
                self.screen
                    .minecraft()
                    .gui
                    .borrow_mut()
                    .hud
                    .chat
                    .reset_chat_scroll();
            }
            return true;
        }

        let minecraft = self.screen.minecraft();
        match event.shortcut_key() {
            1073741899 | 1073741902 => {
                let dir = if event.shortcut_key() == 1073741899 {
                    1
                } else {
                    -1
                };
                minecraft.chat_context(|context: &ChatContext| {
                    let lines = minecraft.gui.borrow().hud.chat.get_lines_per_page(context);
                    minecraft
                        .gui
                        .borrow_mut()
                        .hud
                        .chat
                        .scroll_chat(dir * (lines - 1), context);
                });
            }
            1073741905 => self.move_in_history(1),
            1073741906 => self.move_in_history(-1),
            _ => return false,
        }
        self.after_input();
        true
    }

    fn char_typed(&mut self, event: &CharacterEvent) -> bool {
        let handled = crate::net::minecraft::client::gui::components::events::container_event_handler::char_typed(
            &mut self.screen,
            event,
        );
        self.after_input();
        handled
    }

    fn mouse_scrolled(&mut self, _x: f64, _y: f64, _scroll_x: f64, scroll_y: f64) -> bool {
        let scroll_y = Mth::clamp_f64(scroll_y, -1.0, 1.0) * ChatScreen::MOUSE_SCROLL_SPEED;
        let minecraft = self.screen.minecraft();
        minecraft.chat_context(|context| {
            minecraft
                .gui
                .borrow_mut()
                .hud
                .chat
                .scroll_chat(scroll_y as i32, context);
        });
        true
    }

    fn insert_text(&mut self, text: &str, replace: bool) {
        if replace {
            self.input().borrow_mut().set_value(text);
        } else {
            self.input().borrow_mut().insert_text(text);
        }
        self.after_input();
    }

    fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        mouse_x: i32,
        mouse_y: i32,
        a: f32,
    ) {
        let minecraft = self.screen.minecraft();
        let background = minecraft.options.borrow().get_background_color_or(i32::MIN);
        graphics.fill(
            2,
            self.screen.height - 14,
            self.screen.width - 2,
            self.screen.height - 2,
            background,
        );
        let display_mode = self.display_mode;
        minecraft.chat_context(|context| {
            let ticks = context.gui_ticks;
            minecraft.gui.borrow().hud.chat.extract_render_state(
                graphics,
                context,
                ticks,
                mouse_x,
                mouse_y,
                display_mode,
                false,
            );
        });
        self.screen
            .extract_renderables(graphics, mouse_x, mouse_y, a);
    }

    fn extract_background(
        &mut self,
        _graphics: &mut GuiGraphicsExtractor,
        _x: i32,
        _y: i32,
        _a: f32,
    ) {
    }

    fn is_pause_screen(&self) -> bool {
        false
    }

    fn is_allowed_in_portal(&self) -> bool {
        true
    }
}
