//! net/minecraft/client/gui/Font.java, which shares its module with the `font` package
//!
//! Text drawn with the fonts: its width (`StringSplitter` over each codepoint's advance in its
//! style's font, bold adding one), and `prepareText`, which lays the codepoints out from a place
//! into glyph instances (with their shadows' colors), the underline and strikethrough effects, and
//! a background, for the GUI to draw (`GuiRenderer.prepareText`). Obfuscated text draws a random
//! glyph of the same width from the font's `RandomSource`. The font's provider is the font
//! manager's (`CachedFontProvider`), with or without fishy glyphs.
//!
//! Not ported yet: text in the level (`Font.DisplayMode`, the render types), the outline
//! (`prepare8xTextOutline`), the active areas a click reads (`EmptyArea`, `includeEmpty`'s), and
//! right-to-left text (`bidirectionalShaping`; the default language is left to right).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use std::cell::RefCell;
use std::rc::Rc;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::gui::font::font_manager::{FontManager, GlyphSource};
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;
use crate::net::minecraft::client::gui::font::text_renderable::TextRenderable;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::string_splitter::StringSplitter;
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::font_description::FontDescription;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::network::chat::text_color::TextColor;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::util::string_decomposer::StringDecomposer;

pub mod all_missing_glyph_provider;
pub mod codepoint_map;
pub mod font_manager;
pub mod font_option;
pub mod font_set;
pub mod font_texture;
pub mod glyph_stitcher;
pub mod glyphs;
pub mod providers;
pub mod text_renderable;

pub struct Font {
    random: RefCell<Box<dyn RandomSource>>,
    manager: Rc<FontManager>,
    non_fishy_only: bool,
    splitter: StringSplitter,
}

impl Font {
    pub const EFFECT_DEPTH: f32 = 0.01;
    pub const SHADOW_DEPTH: f32 = 0.03;
    pub const LINE_HEIGHT: i32 = 9;

    /// `new Font(provider)` with the font manager's `anyGlyphs` provider, or its `nonFishyGlyphs`
    /// one (`createFontFilterFishy`).
    pub fn new(manager: Rc<FontManager>, non_fishy_only: bool) -> Font {
        let widths = Rc::clone(&manager);
        Font {
            random: RefCell::new(random_source::create()),
            splitter: StringSplitter::new(Box::new(move |codepoint, style: &Style| {
                widths
                    .glyph_source(&style.get_font(), non_fishy_only)
                    .get_glyph(codepoint)
                    .info()
                    .get_advance_bold(style.is_bold())
            })),
            manager,
            non_fishy_only,
        }
    }

    fn get_glyph_source(&self, font_location: &FontDescription) -> GlyphSource {
        self.manager
            .glyph_source(font_location, self.non_fishy_only)
    }

    /// The glyph a codepoint is drawn with in a style: obfuscated, a random one of its width.
    fn get_glyph(&self, codepoint: i32, style: &Style) -> Rc<BakedGlyph> {
        let glyph_source = self.get_glyph_source(&style.get_font());
        let glyph = glyph_source.get_glyph(codepoint);
        if style.is_obfuscated() && codepoint != 32 {
            let target_width = Mth::ceil_f32(glyph.info().get_advance_bold(false));
            return glyph_source.get_random_glyph(&mut **self.random.borrow_mut(), target_width);
        }
        glyph
    }

    /// `provider.effect()`: the default font's white glyph.
    fn effect(
        &self,
    ) -> Rc<crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph>
    {
        self.manager
            .get_font_set_raw(&FontManager::default_font())
            .borrow()
            .white_glyph()
    }

    pub fn prepare_background(
        &self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: i32,
    ) -> TextRenderable {
        TextRenderable::Effect(
            self.effect()
                .create_effect(x0, y0, x1, y1, -0.01, color, 0, 0.0),
        )
    }

    /// `prepareText(String, float, float, int, boolean, int)`: a string with its `§` codes.
    pub fn prepare_text_str(
        &self,
        text: &str,
        x: f32,
        y: f32,
        original_color: i32,
        draw_shadow: bool,
        background_color: i32,
    ) -> PreparedText {
        assert!(!self.is_bidirectional(), "right-to-left text isn't ported");
        let mut output = PreparedText::new(
            self.effect(),
            x,
            y,
            original_color,
            background_color,
            draw_shadow,
            false,
        );
        let units: Vec<u16> = text.encode_utf16().collect();
        StringDecomposer::iterate_formatted(&units, &Style::EMPTY, &mut |position, style, c| {
            output.accept(self, position, style, c)
        });
        output
    }

    /// `prepareText(FormattedCharSequence, float, float, int, boolean, boolean, int)`.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_text(
        &self,
        text: &FormattedCharSequence,
        x: f32,
        y: f32,
        original_color: i32,
        draw_shadow: bool,
        include_empty: bool,
        background_color: i32,
    ) -> PreparedText {
        let mut builder = PreparedText::new(
            self.effect(),
            x,
            y,
            original_color,
            background_color,
            draw_shadow,
            include_empty,
        );
        text.accept(&mut |position, style, c| builder.accept(self, position, style, c));
        builder
    }

    pub fn width(&self, str: &str) -> i32 {
        Mth::ceil_f32(self.splitter.string_width(Some(str)))
    }

    pub fn width_text(&self, text: &impl FormattedText) -> i32 {
        Mth::ceil_f32(self.splitter.string_width_text(text))
    }

    pub fn width_sequence(&self, text: &FormattedCharSequence) -> i32 {
        Mth::ceil_f32(self.splitter.string_width_sequence(text))
    }

    pub fn plain_substr_by_width_reverse(&self, str: &str, width: i32, reverse: bool) -> String {
        if reverse {
            self.splitter.plain_tail_by_width(str, width, &Style::EMPTY)
        } else {
            self.splitter.plain_head_by_width(str, width, &Style::EMPTY)
        }
    }

    pub fn plain_substr_by_width(&self, str: &str, width: i32) -> String {
        self.splitter.plain_head_by_width(str, width, &Style::EMPTY)
    }

    /// `split(FormattedText, int)`: the text's lines at the width, each in its visual order.
    pub fn split(&self, text: &impl FormattedText, width: i32) -> Vec<FormattedCharSequence> {
        let language = Language::get_instance();
        self.splitter
            .split_lines_text(text, width, &Style::EMPTY)
            .iter()
            .map(|line| language.get_visual_order(line))
            .collect()
    }

    pub fn is_bidirectional(&self) -> bool {
        Language::get_instance().is_default_right_to_left()
    }

    pub fn get_splitter(&self) -> &StringSplitter {
        &self.splitter
    }
}

/// `Font.PreparedText` as `PreparedTextBuilder` makes it: the glyphs, the effects and the
/// background a text was laid out into, and its bounds.
pub struct PreparedText {
    /// `Font.this.provider.effect()`, which the background is drawn with.
    effect:
        Rc<crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph>,
    draw_shadow: bool,
    color: i32,
    background_color: i32,
    include_empty: bool,
    x: f32,
    y: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    background_left: f32,
    background_top: f32,
    background_right: f32,
    background_bottom: f32,
    glyphs: Vec<TextRenderable>,
    effects: Option<Vec<TextRenderable>>,
}

impl PreparedText {
    fn new(
        effect: Rc<
            crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph,
        >,
        x: f32,
        y: f32,
        color: i32,
        background_color: i32,
        draw_shadow: bool,
        include_empty: bool,
    ) -> PreparedText {
        let mut result = PreparedText {
            effect,
            draw_shadow,
            color,
            background_color,
            include_empty,
            x,
            y,
            left: f32::MAX,
            top: f32::MAX,
            right: -f32::MAX,
            bottom: -f32::MAX,
            background_left: f32::MAX,
            background_top: f32::MAX,
            background_right: -f32::MAX,
            background_bottom: -f32::MAX,
            glyphs: Vec::new(),
            effects: None,
        };
        result.mark_background(x, y, 0.0);
        result
    }

    fn mark_size(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.left = Math::min_f32(self.left, left);
        self.top = Math::min_f32(self.top, top);
        self.right = Math::max_f32(self.right, right);
        self.bottom = Math::max_f32(self.bottom, bottom);
    }

    fn mark_background(&mut self, x: f32, y: f32, advance: f32) {
        if ARGB::alpha(self.background_color) != 0 {
            self.background_left = Math::min_f32(self.background_left, x - 1.0);
            self.background_top = Math::min_f32(self.background_top, y - 1.0);
            self.background_right = Math::max_f32(self.background_right, x + advance);
            self.background_bottom = Math::max_f32(self.background_bottom, y + 9.0);
            self.mark_size(
                self.background_left,
                self.background_top,
                self.background_right,
                self.background_bottom,
            );
        }
    }

    fn add_glyph(&mut self, instance: TextRenderable) {
        self.mark_size(
            instance.left(),
            instance.top(),
            instance.right(),
            instance.bottom(),
        );
        self.glyphs.push(instance);
    }

    fn add_effect(&mut self, effect: TextRenderable) {
        self.mark_size(effect.left(), effect.top(), effect.right(), effect.bottom());
        self.effects.get_or_insert_with(Vec::new).push(effect);
    }

    /// `accept(int, Style, int)`: the codepoint's glyph at the cursor, which moves on by its
    /// advance.
    fn accept(&mut self, font: &Font, position: i32, style: &Style, c: i32) -> bool {
        let glyph = font.get_glyph(c, style);
        self.accept_glyph(font, position, style, &glyph)
    }

    fn accept_glyph(
        &mut self,
        font: &Font,
        position: i32,
        style: &Style,
        glyph: &BakedGlyph,
    ) -> bool {
        let glyph_info = glyph.info();
        let bold = style.is_bold();
        let style_color = style.get_color();
        let text_color = self.get_text_color(style_color);
        let shadow_color = self.get_shadow_color(style, text_color);
        let advance = glyph_info.get_advance_bold(bold);
        let effect_x0 = if position == 0 { self.x - 1.0 } else { self.x };
        let shadow_offset = glyph_info.get_shadow_offset();
        let bold_offset = if bold {
            glyph_info.get_bold_offset()
        } else {
            0.0
        };
        match glyph.create_glyph(
            self.x,
            self.y,
            text_color,
            shadow_color,
            style,
            bold_offset,
            shadow_offset,
        ) {
            Some(instance) => self.add_glyph(instance),
            None if self.include_empty => {
                // EmptyArea comes with the active areas.
            }
            None => {}
        }

        self.mark_background(self.x, self.y, advance);
        if style.is_strikethrough() {
            let effect = font.effect().create_effect(
                effect_x0,
                self.y + 4.5 - 1.0,
                self.x + advance,
                self.y + 4.5,
                0.01,
                text_color,
                shadow_color,
                shadow_offset,
            );
            self.add_effect(TextRenderable::Effect(effect));
        }

        if style.is_underlined() {
            let effect = font.effect().create_effect(
                effect_x0,
                self.y + 9.0 - 1.0,
                self.x + advance,
                self.y + 9.0,
                0.01,
                text_color,
                shadow_color,
                shadow_offset,
            );
            self.add_effect(TextRenderable::Effect(effect));
        }

        self.x += advance;
        true
    }

    /// `visit`: the background, the glyphs, then the effects.
    pub fn visit(&self, visitor: &mut dyn FnMut(&TextRenderable)) {
        if ARGB::alpha(self.background_color) != 0 {
            let background = self.effect.create_effect(
                self.background_left,
                self.background_top,
                self.background_right,
                self.background_bottom,
                -0.01,
                self.background_color,
                0,
                0.0,
            );
            visitor(&TextRenderable::Effect(background));
        }
        for glyph in &self.glyphs {
            visitor(glyph);
        }
        if let Some(effects) = &self.effects {
            for effect in effects {
                visitor(effect);
            }
        }
    }

    pub fn discard_effects(&mut self) {
        self.effects = None;
    }

    fn get_text_color(&self, text_color: Option<TextColor>) -> i32 {
        match text_color {
            Some(text_color) => {
                let alpha = ARGB::alpha(self.color);
                let rgb = text_color.get_value();
                ARGB::color_alpha(alpha, rgb)
            }
            None => self.color,
        }
    }

    fn get_shadow_color(&self, style: &Style, text_color: i32) -> i32 {
        match style.get_shadow_color() {
            Some(shadow) => {
                let text_alpha = ARGB::alpha_float(text_color);
                let shadow_alpha = ARGB::alpha_float(shadow);
                if text_alpha != 1.0 {
                    ARGB::color_alpha(ARGB::as8_bit_channel(text_alpha * shadow_alpha), shadow)
                } else {
                    shadow
                }
            }
            None => {
                if self.draw_shadow {
                    ARGB::scale_rgb(text_color, 0.25)
                } else {
                    0
                }
            }
        }
    }

    pub fn bounds(&self) -> Option<ScreenRectangle> {
        if !(self.left >= self.right) && !(self.top >= self.bottom) {
            let left = Mth::floor_f32(self.left);
            let top = Mth::floor_f32(self.top);
            let right = Mth::ceil_f32(self.right);
            let bottom = Mth::ceil_f32(self.bottom);
            Some(ScreenRectangle::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }
}
