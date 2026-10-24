//! net/minecraft/client/gui/components/ChatComponent.java
//!
//! The chat: the messages (the newest first, at most 100) and their lines at the chat's width,
//! drawn over the HUD as they fade 9 to 10 seconds after they came (by the GUI's tick count), or
//! all of them in the chat screen; the chat's scroll, the messages sent (for the up and down keys),
//! the draft a closed chat screen keeps, and the screen a sleeping player's chat leaves behind.
//!
//! Java reads the game's options, font, GUI ticks and screen from its `Minecraft`; here what a
//! method reads is its `ChatContext`. Not ported yet: deleting signed messages (signatures, the
//! deletion queue), the visible message filter (all are visible), the command history file
//! (`CommandHistory`, the recent chat starts empty), restricted chat's prompt, and the active text
//! a click or hover finds (the tags' tooltips and icons on hover).

use std::collections::VecDeque;
use std::rc::Rc;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::gui::active_text_collector::ActiveTextCollector;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::{
    GuiGraphicsExtractor, HoveredTextEffects,
};
use crate::net::minecraft::client::gui::screens::screen::ScreenRef;
use crate::net::minecraft::client::gui::text_alignment::TextAlignment;
use crate::net::minecraft::client::multiplayer::chat::chat_listener;
use crate::net::minecraft::client::multiplayer::chat::gui_message::{GuiMessage, Line};
use crate::net::minecraft::client::multiplayer::chat::gui_message_source::GuiMessageSource;
use crate::net::minecraft::client::multiplayer::chat::gui_message_tag::GuiMessageTag;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;

/// What the chat reads of the game for one call.
pub struct ChatContext<'a> {
    pub options: &'a Options,
    pub font: &'a Font,
    /// `minecraft.gui.hud.getGuiTicks()`.
    pub gui_ticks: i32,
    /// `isChatFocused()`: the chat screen is shown.
    pub chat_focused: bool,
}

/// `ChatComponent.DisplayMode`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    BACKGROUND,
    FOREGROUND,
    FOREGROUND_RESTRICTED,
}

impl DisplayMode {
    pub fn foreground(self) -> bool {
        self != DisplayMode::BACKGROUND
    }

    pub fn show_restricted_prompt(self) -> bool {
        self == DisplayMode::FOREGROUND_RESTRICTED
    }
}

/// `ChatComponent.ChatMethod`: a message, or a command (the chat opened with its slash).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatMethod {
    MESSAGE,
    COMMAND,
}

impl ChatMethod {
    pub fn prefix(self) -> &'static str {
        match self {
            ChatMethod::MESSAGE => "",
            ChatMethod::COMMAND => "/",
        }
    }

    pub fn is_draft_restorable(self, draft: &Draft) -> bool {
        match self {
            ChatMethod::MESSAGE => true,
            ChatMethod::COMMAND => self == draft.chat_method,
        }
    }
}

/// `ChatComponent.Draft`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    pub chat_method: ChatMethod,
}

#[derive(Default)]
pub struct ChatComponent {
    recent_chat: VecDeque<String>,
    all_messages: VecDeque<Rc<GuiMessage>>,
    trimmed_messages: VecDeque<Line>,
    chat_scrollbar_pos: i32,
    new_message_since_scroll: bool,
    latest_draft: Option<Draft>,
    preserved_screen: Option<ScreenRef>,
}

const MAX_CHAT_HISTORY: usize = 100;
pub const MESSAGE_BOTTOM_TO_MESSAGE_TOP: i32 = 8;

impl ChatComponent {
    pub fn new() -> ChatComponent {
        ChatComponent::default()
    }

    /// `tick`: the deletion queue, which isn't ported.
    pub fn tick(&mut self) {}

    /// `forEachLine`: the lines on the page from the bottom up, with their alpha, skipping the
    /// invisible ones; how many there were.
    fn for_each_line(
        &self,
        context: &ChatContext,
        alpha_calculator: &dyn Fn(&Line) -> f32,
        line_consumer: &mut dyn FnMut(&Line, i32, f32),
    ) -> i32 {
        let per_page = self.get_lines_per_page(context);
        let mut count = 0;
        let mut i =
            (self.trimmed_messages.len() as i32 - self.chat_scrollbar_pos).min(per_page) - 1;
        while i >= 0 {
            let message_index = i + self.chat_scrollbar_pos;
            let message = &self.trimmed_messages[message_index as usize];
            let alpha = alpha_calculator(message);
            if alpha > 1.0E-5 {
                count += 1;
                line_consumer(message, i, alpha);
            }
            i -= 1;
        }
        count
    }

    /// `extractRenderState(GuiGraphicsExtractor, Font, int, int, int, DisplayMode, boolean)`.
    #[allow(clippy::too_many_arguments)]
    pub fn extract_render_state(
        &self,
        graphics: &mut GuiGraphicsExtractor,
        context: &ChatContext,
        ticks: i32,
        _mouse_x: i32,
        _mouse_y: i32,
        display_mode: DisplayMode,
        _change_cursor_on_insertions: bool,
    ) {
        graphics.pose().push_matrix();
        let effects = if display_mode.foreground() {
            HoveredTextEffects::TOOLTIP_AND_CURSOR
        } else {
            HoveredTextEffects::NONE
        };
        self.extract(
            graphics,
            effects,
            graphics.gui_height(),
            ticks,
            display_mode,
            context,
        );
        graphics.pose().pop_matrix();
    }

    /// The private `extractRenderState(ChatGraphicsAccess, int, int, DisplayMode)` with the graphics
    /// access drawing (`DrawingBackgroundGraphicsAccess`, or `DrawingFocusedGraphicsAccess`, whose
    /// hovered styles aren't ported).
    fn extract(
        &self,
        graphics: &mut GuiGraphicsExtractor,
        effects: HoveredTextEffects,
        screen_height: i32,
        ticks: i32,
        display_mode: DisplayMode,
        context: &ChatContext,
    ) {
        let is_foreground = display_mode.foreground();
        let is_restricted = display_mode.show_restricted_prompt();
        let total = self.trimmed_messages.len() as i32;
        if total <= 0 && !is_restricted {
            return;
        }
        if is_restricted {
            panic!("the chat's restricted prompt isn't ported");
        }
        let options = context.options;
        let scale = self.get_scale(context) as f32;
        let max_width = Mth::ceil_f32(self.get_width(context) as f32 / scale);
        let chat_bottom = Mth::floor_f32((screen_height - 40) as f32 / scale);
        let text_opacity = options.chat_opacity as f32 * 0.9 + 0.1;
        let background_opacity = options.text_background_opacity as f32;
        let message_height = 9;
        let chat_line_spacing = options.chat_line_spacing;
        let entry_height = (f64::from(message_height) * (chat_line_spacing + 1.0)) as i32;
        let entry_bottom_to_message_y =
            Math::round_f64(8.0 * (chat_line_spacing + 1.0) - 4.0 * chat_line_spacing) as i32;
        let queue_size = chat_listener::queue_size();
        let alpha_calculator: Box<dyn Fn(&Line) -> f32> = if is_foreground {
            Box::new(|_| 1.0)
        } else {
            Box::new(move |message| time_based_alpha(ticks, message))
        };

        // updatePose: the scale and the indent.
        graphics.pose().scale(scale, scale);
        graphics.pose().translate(4.0, 0.0);
        let mut parameters = graphics.text_renderer(effects).default_parameters();
        parameters = parameters.with_pose(**graphics.pose());

        let mut fills = Vec::new();
        let count = self.for_each_line(context, &*alpha_calculator, &mut |_, line_index, alpha| {
            let entry_bottom = chat_bottom - line_index * entry_height;
            let entry_top = entry_bottom - entry_height;
            fills.push((
                -4,
                entry_top,
                max_width + 4 + 4,
                entry_bottom,
                ARGB::black_f32(alpha * background_opacity),
            ));
        });
        for (x0, y0, x1, y1, color) in fills {
            graphics.fill(x0, y0, x1, y1, color);
        }
        let _line_above_messages_y = chat_bottom - (count + 1) * entry_height;
        if queue_size > 0 {
            graphics.fill(
                -2,
                chat_bottom,
                max_width + 4,
                chat_bottom + message_height,
                ARGB::black_f32(background_opacity),
            );
        }

        let mut lines = Vec::new();
        self.for_each_line(
            context,
            &*alpha_calculator,
            &mut |line, line_index, alpha| {
                lines.push((line.clone(), line_index, alpha));
            },
        );
        for (line, line_index, alpha) in lines {
            let entry_bottom = chat_bottom - line_index * entry_height;
            let entry_top = entry_bottom - entry_height;
            let text_top = entry_bottom - entry_bottom_to_message_y;
            // handleMessage: the line at the opacity.
            {
                let mut text_renderer = graphics.text_renderer(effects);
                text_renderer.accept(
                    TextAlignment::LEFT,
                    0,
                    text_top,
                    &parameters.with_opacity(alpha * text_opacity),
                    &line.content,
                );
            }
            if let Some(tag) = line.tag() {
                // handleTag: the indicator.
                let indicator_color =
                    ARGB::color_alpha_f32(alpha * text_opacity, tag.indicator_color);
                graphics.fill(-4, entry_top, -2, entry_bottom, indicator_color);
                // handleTagIcon: drawn when forced (the message hovered), which isn't ported.
            }
        }

        if total > 0 && is_foreground {
            let chat_height = count * entry_height;
            let virtual_height = total * entry_height;
            let y = self.chat_scrollbar_pos * chat_height / total - chat_bottom;
            let height = chat_height * chat_height / virtual_height;
            if virtual_height != chat_height {
                let alpha = if y > 0 { 170 } else { 96 };
                let color = if self.new_message_since_scroll {
                    13382451
                } else {
                    3355562
                };
                let scroll_bar_start_x = max_width + 4;
                graphics.fill(
                    scroll_bar_start_x,
                    -y,
                    scroll_bar_start_x + 2,
                    -y - height,
                    ARGB::color_alpha(alpha, color),
                );
                graphics.fill(
                    scroll_bar_start_x + 2,
                    -y,
                    scroll_bar_start_x + 1,
                    -y - height,
                    ARGB::color_alpha(alpha, 13421772),
                );
            }
        }
    }

    pub fn clear_messages(&mut self, history: bool) {
        self.trimmed_messages.clear();
        self.all_messages.clear();
        if history {
            self.recent_chat.clear();
        }
    }

    pub fn add_client_system_message(&mut self, message: Component, context: &ChatContext) {
        self.add_message(
            message,
            GuiMessageSource::SYSTEM_CLIENT,
            Some(GuiMessageTag::system_single_player()),
            context,
        );
    }

    pub fn add_server_system_message(&mut self, message: Component, context: &ChatContext) {
        self.add_message(
            message,
            GuiMessageSource::SYSTEM_SERVER,
            Some(GuiMessageTag::system_single_player()),
            context,
        );
    }

    fn add_message(
        &mut self,
        contents: Component,
        source: GuiMessageSource,
        tag: Option<GuiMessageTag>,
        context: &ChatContext,
    ) {
        let message = Rc::new(GuiMessage {
            added_time: context.gui_ticks,
            content: contents,
            source,
            tag,
        });
        ChatComponent::log_chat_message(&message);
        self.add_message_to_display_queue(&message, context);
        self.add_message_to_queue(message);
    }

    /// `logChatMessage`: `[System] [CHAT] …`, its line breaks escaped.
    fn log_chat_message(message: &GuiMessage) {
        let message_string = message
            .content
            .get_string()
            .replace('\r', "\\r")
            .replace('\n', "\\n");
        match message.tag.as_ref().and_then(|tag| tag.log_tag.as_deref()) {
            Some(log_tag) => eprintln!("[{log_tag}] [CHAT] {message_string}"),
            None => eprintln!("[CHAT] {message_string}"),
        }
    }

    fn add_message_to_display_queue(&mut self, message: &Rc<GuiMessage>, context: &ChatContext) {
        let max_width =
            Mth::floor_f64(f64::from(self.get_width(context)) / self.get_scale(context));
        let lines = message.split_lines(context.font, max_width, context.options.chat_colors());
        let chatting = context.chat_focused;
        let count = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            if chatting && self.chat_scrollbar_pos > 0 {
                self.new_message_since_scroll = true;
                self.scroll_chat(1, context);
            }
            self.trimmed_messages.push_front(Line {
                parent: Rc::clone(message),
                content: line,
                end_of_entry: i == count - 1,
            });
        }
        while self.trimmed_messages.len() > MAX_CHAT_HISTORY {
            self.trimmed_messages.pop_back();
        }
    }

    fn add_message_to_queue(&mut self, message: Rc<GuiMessage>) {
        self.all_messages.push_front(message);
        while self.all_messages.len() > MAX_CHAT_HISTORY {
            self.all_messages.pop_back();
        }
    }

    pub fn rescale_chat(&mut self, context: &ChatContext) {
        self.reset_chat_scroll();
        self.refresh_trimmed_messages(context);
    }

    fn refresh_trimmed_messages(&mut self, context: &ChatContext) {
        self.trimmed_messages.clear();
        let messages: Vec<_> = self.all_messages.iter().rev().cloned().collect();
        for message in messages {
            self.add_message_to_display_queue(&message, context);
        }
    }

    pub fn get_recent_chat(&self) -> &VecDeque<String> {
        &self.recent_chat
    }

    /// `addRecentChat`: a message sent, unless it's the last one again. The command history file
    /// isn't ported.
    pub fn add_recent_chat(&mut self, message: &str) {
        if self.recent_chat.back().map(String::as_str) != Some(message) {
            if self.recent_chat.len() >= MAX_CHAT_HISTORY {
                self.recent_chat.pop_front();
            }
            self.recent_chat.push_back(message.to_string());
        }
    }

    pub fn reset_chat_scroll(&mut self) {
        self.chat_scrollbar_pos = 0;
        self.new_message_since_scroll = false;
    }

    pub fn scroll_chat(&mut self, dir: i32, context: &ChatContext) {
        self.chat_scrollbar_pos += dir;
        let max = self.trimmed_messages.len() as i32;
        let lines_per_page = self.get_lines_per_page(context);
        if self.chat_scrollbar_pos > max - lines_per_page {
            self.chat_scrollbar_pos = max - lines_per_page;
        }
        if self.chat_scrollbar_pos <= 0 {
            self.chat_scrollbar_pos = 0;
            self.new_message_since_scroll = false;
        }
    }

    fn get_width(&self, context: &ChatContext) -> i32 {
        ChatComponent::get_width_for(context.options.chat_width)
    }

    fn get_height(&self, context: &ChatContext) -> i32 {
        ChatComponent::get_height_for(if context.chat_focused {
            context.options.chat_height_focused
        } else {
            context.options.chat_height_unfocused
        })
    }

    fn get_scale(&self, context: &ChatContext) -> f64 {
        context.options.chat_scale
    }

    /// `getWidth(double)`.
    pub fn get_width_for(pct: f64) -> i32 {
        Mth::floor_f64(pct * 280.0 + 40.0)
    }

    /// `getHeight(double)`.
    pub fn get_height_for(pct: f64) -> i32 {
        Mth::floor_f64(pct * 160.0 + 20.0)
    }

    pub fn default_unfocused_pct() -> f64 {
        70.0 / f64::from(ChatComponent::get_height_for(1.0) - 20)
    }

    pub fn get_lines_per_page(&self, context: &ChatContext) -> i32 {
        self.get_height(context) / ChatComponent::get_line_height(context)
    }

    fn get_line_height(context: &ChatContext) -> i32 {
        (9.0 * (context.options.chat_line_spacing + 1.0)) as i32
    }

    pub fn save_as_draft(&mut self, text: &str) {
        let chat_method = if text.starts_with('/') {
            ChatMethod::COMMAND
        } else {
            ChatMethod::MESSAGE
        };
        self.latest_draft = Some(Draft {
            text: text.to_string(),
            chat_method,
        });
    }

    pub fn discard_draft(&mut self) {
        self.latest_draft = None;
    }

    /// `createScreen`'s text: the draft when it's restorable, else the method's prefix, and
    /// whether it's a draft.
    pub fn screen_text(&self, chat_method: ChatMethod) -> (String, bool) {
        match &self.latest_draft {
            Some(draft) if chat_method.is_draft_restorable(draft) => (draft.text.clone(), true),
            _ => (chat_method.prefix().to_string(), false),
        }
    }

    /// `preserveCurrentChatScreen`'s other half: the screen kept.
    pub fn preserve_screen(&mut self, screen: ScreenRef) {
        self.preserved_screen = Some(screen);
    }

    pub fn restore_chat_screen(&mut self) -> Option<ScreenRef> {
        self.preserved_screen.take()
    }
}

/// `AlphaCalculator.timeBased`: full for 9 seconds, then fading for one.
fn time_based_alpha(current_tick_time: i32, message: &Line) -> f32 {
    let tick_delta = current_tick_time - message.added_time();
    let mut t = f64::from(tick_delta) / 200.0;
    t = 1.0 - t;
    t *= 10.0;
    t = Mth::clamp_f64(t, 0.0, 1.0);
    t *= t;
    t as f32
}
