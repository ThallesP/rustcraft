//! net/minecraft/client/gui/ActiveTextCollector.java
//!
//! Where a widget's or the chat's text goes: drawn (`GuiGraphicsExtractor`'s collectors), or
//! searched for what's under the mouse. Text too wide for its space scrolls back and forth with
//! the time (`defaultScrollingHelper`), as Java's does. Not ported yet: `ClickableStyleFinder` and
//! `findElementUnderCursor`, which find the style of the text under the mouse (a click or hover
//! event's), and the overloads taking a `Component` rather than its visual order.

use std::f64::consts::PI;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::text_alignment::TextAlignment;
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;
use crate::org::joml::matrix3x2f::Matrix3x2f;

/// `ActiveTextCollector.Parameters`: the text's pose, opacity and scissor.
#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    pub pose: Matrix3x2f,
    pub opacity: f32,
    pub scissor: Option<ScreenRectangle>,
}

impl Parameters {
    pub fn new(pose: Matrix3x2f) -> Parameters {
        Parameters {
            pose,
            opacity: 1.0,
            scissor: None,
        }
    }

    pub fn with_pose(&self, pose: Matrix3x2f) -> Parameters {
        Parameters { pose, ..*self }
    }

    pub fn with_opacity(&self, opacity: f32) -> Parameters {
        Parameters { opacity, ..*self }
    }

    pub fn with_scissor(&self, scissor: ScreenRectangle) -> Parameters {
        Parameters {
            scissor: Some(scissor),
            ..*self
        }
    }

    /// `withScissor(int, int, int, int)`: the rectangle under the pose, within the scissor there is.
    pub fn with_scissor_bounds(&self, left: i32, right: i32, top: i32, bottom: i32) -> Parameters {
        let mut new_scissor = ScreenRectangle::new(left, top, right - left, bottom - top)
            .transform_axis_aligned(&self.pose);
        if let Some(scissor) = &self.scissor {
            new_scissor = scissor
                .intersection(&new_scissor)
                .unwrap_or_else(ScreenRectangle::empty);
        }
        self.with_scissor(new_scissor)
    }
}

pub trait ActiveTextCollector {
    fn font_width(&self, message: &Component) -> i32;

    fn default_parameters(&self) -> Parameters;

    fn set_default_parameters(&mut self, new_parameters: Parameters);

    /// `accept(TextAlignment, int, int, Parameters, FormattedCharSequence)`.
    fn accept(
        &mut self,
        alignment: TextAlignment,
        anchor_x: i32,
        y: i32,
        parameters: &Parameters,
        text: &FormattedCharSequence,
    );

    /// `accept(int, int, FormattedCharSequence)`.
    fn accept_left(&mut self, x: i32, y: i32, text: &FormattedCharSequence) {
        let parameters = self.default_parameters();
        self.accept(TextAlignment::LEFT, x, y, &parameters, text);
    }

    /// `accept(TextAlignment, int, int, Component)`.
    fn accept_component(
        &mut self,
        alignment: TextAlignment,
        anchor_x: i32,
        y: i32,
        text: &Component,
    ) {
        let parameters = self.default_parameters();
        let text = Language::get_instance().get_visual_order(text);
        self.accept(alignment, anchor_x, y, &parameters, &text);
    }

    /// `acceptScrolling(Component, int, int, int, int, int, Parameters)`: the collectors' own,
    /// which measure the message and call `default_scrolling_helper`.
    #[allow(clippy::too_many_arguments)]
    fn accept_scrolling_with(
        &mut self,
        message: &Component,
        center_x: i32,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
        parameters: &Parameters,
    ) {
        let line_width = self.font_width(message);
        self.default_scrolling_helper(
            message, center_x, left, right, top, bottom, line_width, 9, parameters,
        );
    }

    /// `acceptScrolling(Component, int, int, int, int, int)`.
    fn accept_scrolling(
        &mut self,
        message: &Component,
        center_x: i32,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    ) {
        let parameters = self.default_parameters();
        self.accept_scrolling_with(message, center_x, left, right, top, bottom, &parameters);
    }

    fn accept_scrolling_with_default_center(
        &mut self,
        message: &Component,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    ) {
        self.accept_scrolling(message, (left + right) / 2, left, right, top, bottom);
    }

    /// `defaultScrollingHelper`: centered when it fits, else scrolling in a scissor with the time.
    #[allow(clippy::too_many_arguments)]
    fn default_scrolling_helper(
        &mut self,
        message: &Component,
        center_x: i32,
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
        line_width: i32,
        line_height: i32,
        parameters: &Parameters,
    ) {
        let text_top = (top + bottom - line_height) / 2 + 1;
        let available_message_width = right - left;
        if line_width > available_message_width {
            let max_position = line_width - available_message_width;
            let time = Util::get_millis() as f64 / 1000.0;
            let period = (f64::from(max_position) * 0.5).max(3.0);
            let alpha = Math::sin((PI / 2.0) * Math::cos((PI * 2.0) * time / period)) / 2.0 + 0.5;
            let pos = Mth::lerp_f64(alpha, 0.0, f64::from(max_position));
            let local_parameters = parameters.with_scissor_bounds(left, right, top, bottom);
            let text = Language::get_instance().get_visual_order(message);
            self.accept(
                TextAlignment::LEFT,
                left - pos as i32,
                text_top,
                &local_parameters,
                &text,
            );
        } else {
            let text_x = Mth::clamp_i32(center_x, left + line_width / 2, right - line_width / 2);
            self.accept_component(TextAlignment::CENTER, text_x, text_top, message);
        }
    }
}

/// `isPointInRectangle`.
pub fn is_point_in_rectangle(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) -> bool {
    x >= left && x < right && y >= top && y < bottom
}
