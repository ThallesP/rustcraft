//! net/minecraft/client/renderer/state/gui/GuiTextRenderState.java
//!
//! A text the GUI draws: laid out by its font when first asked for (its bounds, then its glyphs),
//! under its pose and scissor.

use std::cell::OnceCell;
use std::rc::Rc;

use crate::net::minecraft::client::gui::font::{Font, PreparedText};
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::org::joml::matrix3x2f::Matrix3x2f;

pub struct GuiTextRenderState {
    font: Rc<Font>,
    text: FormattedCharSequence,
    pub pose: Matrix3x2f,
    x: i32,
    y: i32,
    color: i32,
    background_color: i32,
    drop_shadow: bool,
    include_empty: bool,
    pub scissor: Option<ScreenRectangle>,
    prepared_text: OnceCell<(PreparedText, Option<ScreenRectangle>)>,
}

impl GuiTextRenderState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        font: Rc<Font>,
        text: FormattedCharSequence,
        pose: Matrix3x2f,
        x: i32,
        y: i32,
        color: i32,
        background_color: i32,
        drop_shadow: bool,
        include_empty: bool,
        scissor: Option<ScreenRectangle>,
    ) -> GuiTextRenderState {
        GuiTextRenderState {
            font,
            text,
            pose,
            x,
            y,
            color,
            background_color,
            drop_shadow,
            include_empty,
            scissor,
            prepared_text: OnceCell::new(),
        }
    }

    /// `ensurePrepared`: the text laid out, and its bounds under the pose and in the scissor.
    pub fn ensure_prepared(&self) -> &PreparedText {
        &self
            .prepared_text
            .get_or_init(|| {
                let prepared_text = self.font.prepare_text(
                    &self.text,
                    self.x as f32,
                    self.y as f32,
                    self.color,
                    self.drop_shadow,
                    self.include_empty,
                    self.background_color,
                );
                let bounds = prepared_text.bounds().and_then(|bounds| {
                    let bounds = bounds.transform_max_bounds(&self.pose);
                    match self.scissor {
                        Some(scissor) => scissor.intersection(&bounds),
                        None => Some(bounds),
                    }
                });
                (prepared_text, bounds)
            })
            .0
    }
}

impl ScreenArea for GuiTextRenderState {
    fn bounds(&self) -> Option<ScreenRectangle> {
        self.ensure_prepared();
        self.prepared_text.get().and_then(|(_, bounds)| *bounds)
    }
}
