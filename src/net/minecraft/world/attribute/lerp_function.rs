//! net/minecraft/world/attribute/LerpFunction.java
//!
//! How two values blend: Java's lambdas, as an enum of the ones the ported types and modifiers
//! use. `ofInteger`, `ofColor` (of ints) and `ofListCrossFade` come with their types.

use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::modifier::color_modifier::BlendToGray;
use crate::net::minecraft::world::attribute::modifier::float_with_alpha::FloatWithAlpha;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LerpFunction {
    /// `ofFloat`: `Mth.lerp`.
    Float,
    /// `ofDegrees(maxDelta)`: along the shorter way round, jumping when it's `maxDelta` or more.
    Degrees(f32),
    /// `ofStep(threshold)`: `from` until `alpha` reaches the threshold. `ofConstant` is
    /// `Step(1.0)`.
    Step(f32),
    /// `ofColorVec3`: `ARGB.srgbLerp`.
    ColorVec3,
    /// `ofColorVec4`.
    ColorVec4,
    /// `FloatModifier.ALPHA_BLEND`'s argument lerp: each of the value and the alpha.
    FloatWithAlpha,
    /// `ColorModifier.BlendToGray::lerp`.
    BlendToGray,
}

impl LerpFunction {
    pub const CONSTANT: LerpFunction = LerpFunction::Step(1.0);

    pub fn apply(&self, alpha: f32, from: AttributeValue, to: AttributeValue) -> AttributeValue {
        match self {
            LerpFunction::Float => {
                AttributeValue::Float(Mth::lerp_f32(alpha, from.as_float(), to.as_float()))
            }
            LerpFunction::Degrees(max_delta) => {
                let (from, to) = (from.as_float(), to.as_float());
                let delta = Mth::wrap_degrees_f32(to - from);
                AttributeValue::Float(if delta.abs() >= *max_delta {
                    to
                } else {
                    from + alpha * delta
                })
            }
            LerpFunction::Step(threshold) => {
                if alpha >= *threshold {
                    to
                } else {
                    from
                }
            }
            LerpFunction::ColorVec3 => AttributeValue::Rgb(ARGB::srgb_lerp_vector3f(
                alpha,
                &from.as_rgb(),
                &to.as_rgb(),
            )),
            LerpFunction::ColorVec4 => AttributeValue::Argb(ARGB::srgb_lerp_vector4f(
                alpha,
                &from.as_argb(),
                &to.as_argb(),
            )),
            LerpFunction::FloatWithAlpha => {
                let (AttributeValue::FloatWithAlpha(from), AttributeValue::FloatWithAlpha(to)) =
                    (from, to)
                else {
                    panic!("ClassCastException: {from:?} and {to:?} aren't FloatWithAlphas");
                };
                AttributeValue::FloatWithAlpha(FloatWithAlpha::new(
                    Mth::lerp_f32(alpha, from.value, to.value),
                    Mth::lerp_f32(alpha, from.alpha, to.alpha),
                ))
            }
            LerpFunction::BlendToGray => {
                let (AttributeValue::BlendToGray(from), AttributeValue::BlendToGray(to)) =
                    (from, to)
                else {
                    panic!("ClassCastException: {from:?} and {to:?} aren't BlendToGrays");
                };
                AttributeValue::BlendToGray(BlendToGray::lerp(alpha, &from, &to))
            }
        }
    }
}
