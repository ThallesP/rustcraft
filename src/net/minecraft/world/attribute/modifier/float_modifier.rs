//! net/minecraft/world/attribute/modifier/FloatModifier.java
//!
//! `ADD`, `SUBTRACT`, `MULTIPLY`, `MINIMUM` and `MAXIMUM` are `FloatModifier.Simple`s, which take a
//! float and blend it like a float.

use crate::com::google::gson::json_element::JsonElement;
use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::attribute_type::{AttributeValue, decode_float};
use crate::net::minecraft::world::attribute::lerp_function::LerpFunction;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::OperationId;
use crate::net::minecraft::world::attribute::modifier::float_with_alpha::FloatWithAlpha;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatModifier {
    ALPHA_BLEND,
    ADD,
    SUBTRACT,
    MULTIPLY,
    MINIMUM,
    MAXIMUM,
}

impl FloatModifier {
    pub fn apply(&self, subject: f32, argument: &AttributeValue) -> AttributeValue {
        AttributeValue::Float(match self {
            FloatModifier::ALPHA_BLEND => {
                let AttributeValue::FloatWithAlpha(argument) = argument else {
                    panic!("ClassCastException: {argument:?} isn't a FloatWithAlpha");
                };
                Mth::lerp_f32(argument.alpha, subject, argument.value)
            }
            // Float::sum.
            FloatModifier::ADD => subject + argument.as_float(),
            FloatModifier::SUBTRACT => subject - argument.as_float(),
            FloatModifier::MULTIPLY => subject * argument.as_float(),
            FloatModifier::MINIMUM => Math::min_f32(subject, argument.as_float()),
            FloatModifier::MAXIMUM => Math::max_f32(subject, argument.as_float()),
        })
    }

    pub fn decode_argument(&self, json: &JsonElement) -> Result<AttributeValue, String> {
        match self {
            FloatModifier::ALPHA_BLEND => {
                FloatWithAlpha::from_json(json).map(AttributeValue::FloatWithAlpha)
            }
            _ => decode_float(json).map(AttributeValue::Float),
        }
    }

    pub fn argument_keyframe_lerp(&self) -> LerpFunction {
        match self {
            FloatModifier::ALPHA_BLEND => LerpFunction::FloatWithAlpha,
            _ => LerpFunction::Float,
        }
    }

    /// `AttributeModifier.FLOAT_LIBRARY`.
    pub fn by_operation(operation: OperationId) -> Option<FloatModifier> {
        match operation {
            OperationId::ALPHA_BLEND => Some(FloatModifier::ALPHA_BLEND),
            OperationId::ADD => Some(FloatModifier::ADD),
            OperationId::SUBTRACT => Some(FloatModifier::SUBTRACT),
            OperationId::MULTIPLY => Some(FloatModifier::MULTIPLY),
            OperationId::MINIMUM => Some(FloatModifier::MINIMUM),
            OperationId::MAXIMUM => Some(FloatModifier::MAXIMUM),
            _ => None,
        }
    }
}
