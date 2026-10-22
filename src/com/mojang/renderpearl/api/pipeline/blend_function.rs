//! com/mojang/renderpearl/api/pipeline/BlendFunction.java

use crate::com::mojang::renderpearl::api::pipeline::blend_equation::BlendEquation;
use crate::com::mojang::renderpearl::api::pipeline::blend_factor::BlendFactor::{self, *};
use crate::com::mojang::renderpearl::api::pipeline::blend_op::BlendOp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlendFunction {
    pub color: BlendEquation,
    pub alpha: BlendEquation,
}

impl BlendFunction {
    pub const LIGHTNING: BlendFunction = BlendFunction::of(SRC_ALPHA, ONE);
    pub const GLINT: BlendFunction = BlendFunction::separate(SRC_COLOR, ONE, ZERO, ONE);
    pub const OVERLAY: BlendFunction = BlendFunction::separate(SRC_ALPHA, ONE, ONE, ZERO);
    pub const TRANSLUCENT: BlendFunction =
        BlendFunction::separate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA);
    pub const TRANSLUCENT_PREMULTIPLIED_ALPHA: BlendFunction =
        BlendFunction::separate(ONE, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA);
    pub const ADDITIVE: BlendFunction = BlendFunction::of(ONE, ONE);
    pub const ENTITY_OUTLINE_BLIT: BlendFunction =
        BlendFunction::separate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ZERO, ONE);
    pub const INVERT: BlendFunction =
        BlendFunction::separate(ONE_MINUS_DST_COLOR, ONE_MINUS_SRC_COLOR, ONE, ZERO);
    pub const MAX: BlendFunction = BlendFunction::of_op(ONE, ONE, BlendOp::MAX);

    pub const fn new(color: BlendEquation, alpha: BlendEquation) -> BlendFunction {
        BlendFunction { color, alpha }
    }

    /// `BlendFunction(srcColor, dstColor, colorOp, srcAlpha, dstAlpha, alphaOp)`.
    pub const fn separate_ops(
        src_color_factor: BlendFactor,
        dst_color_factor: BlendFactor,
        color_op: BlendOp,
        src_alpha_factor: BlendFactor,
        dst_alpha_factor: BlendFactor,
        alpha_op: BlendOp,
    ) -> BlendFunction {
        BlendFunction::new(
            BlendEquation::new(src_color_factor, dst_color_factor, color_op),
            BlendEquation::new(src_alpha_factor, dst_alpha_factor, alpha_op),
        )
    }

    /// `BlendFunction(srcColor, dstColor, srcAlpha, dstAlpha)`, adding both.
    pub const fn separate(
        src_color_factor: BlendFactor,
        dst_color_factor: BlendFactor,
        src_alpha_factor: BlendFactor,
        dst_alpha_factor: BlendFactor,
    ) -> BlendFunction {
        BlendFunction::separate_ops(
            src_color_factor,
            dst_color_factor,
            BlendOp::ADD,
            src_alpha_factor,
            dst_alpha_factor,
            BlendOp::ADD,
        )
    }

    /// `BlendFunction(BlendEquation)`, the same equation for color and alpha.
    pub const fn from_equation(equation: BlendEquation) -> BlendFunction {
        BlendFunction::new(equation, equation)
    }

    /// `BlendFunction(srcFactor, dstFactor, op)`.
    pub const fn of_op(
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
        op: BlendOp,
    ) -> BlendFunction {
        BlendFunction::from_equation(BlendEquation::new(src_factor, dst_factor, op))
    }

    /// `BlendFunction(srcFactor, dstFactor)`, adding.
    pub const fn of(src_factor: BlendFactor, dst_factor: BlendFactor) -> BlendFunction {
        BlendFunction::of_op(src_factor, dst_factor, BlendOp::ADD)
    }
}
