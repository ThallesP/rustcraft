//! net/minecraft/world/attribute/modifier/ColorModifier.java
//!
//! The color modifiers, of RGB (`Vector3fc`) and ARGB (`Vector4fc`) attributes. An `ArgbModifier`
//! takes an ARGB argument (`#aarrggbb`) and an `RgbModifier` an RGB one (`#rrggbb`), each blended
//! as its kind of color; `BLEND_TO_GRAY` takes a `BlendToGray`.

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::attribute_type::{
    AttributeValue, decode_argb, decode_float, decode_rgb,
};
use crate::net::minecraft::world::attribute::lerp_function::LerpFunction;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::OperationId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorModifier {
    ALPHA_BLEND_RGB,
    ALPHA_BLEND_ARGB,
    ADD_RGB,
    SUBTRACT_RGB,
    MULTIPLY_RGB,
    ADD_ARGB,
    SUBTRACT_ARGB,
    MULTIPLY_ARGB,
    BLEND_TO_GRAY_RGB,
    BLEND_TO_GRAY_ARGB,
}

impl ColorModifier {
    pub fn apply(&self, subject: AttributeValue, argument: &AttributeValue) -> AttributeValue {
        match self {
            ColorModifier::ALPHA_BLEND_RGB => AttributeValue::Rgb(ARGB::alpha_blend_vector3f(
                &subject.as_rgb(),
                &argument.as_argb(),
            )),
            ColorModifier::ALPHA_BLEND_ARGB => AttributeValue::Argb(ARGB::alpha_blend_vector4f(
                &subject.as_argb(),
                &argument.as_argb(),
            )),
            ColorModifier::ADD_RGB => AttributeValue::Rgb(ARGB::add_rgb_vector3f(
                &subject.as_rgb(),
                &argument.as_rgb(),
            )),
            ColorModifier::SUBTRACT_RGB => AttributeValue::Rgb(ARGB::subtract_rgb_vector3f(
                &subject.as_rgb(),
                &argument.as_rgb(),
            )),
            ColorModifier::MULTIPLY_RGB => AttributeValue::Rgb(ARGB::multiply_vector3f(
                &subject.as_rgb(),
                &argument.as_rgb(),
            )),
            ColorModifier::ADD_ARGB => AttributeValue::Argb(ARGB::add_rgb_vector4f(
                &subject.as_argb(),
                &argument.as_rgb(),
            )),
            ColorModifier::SUBTRACT_ARGB => AttributeValue::Argb(ARGB::subtract_rgb_vector4f(
                &subject.as_argb(),
                &argument.as_rgb(),
            )),
            ColorModifier::MULTIPLY_ARGB => AttributeValue::Argb(ARGB::multiply_vector4f(
                &subject.as_argb(),
                &argument.as_argb(),
            )),
            ColorModifier::BLEND_TO_GRAY_RGB => {
                let argument = BlendToGray::of(argument);
                let subject = subject.as_rgb();
                let multiplied_greyscale = ARGB::scale_rgb_vector3f(
                    &ARGB::greyscale_vector3f(&subject),
                    argument.brightness,
                );
                AttributeValue::Rgb(ARGB::srgb_lerp_vector3f(
                    argument.factor,
                    &subject,
                    &multiplied_greyscale,
                ))
            }
            ColorModifier::BLEND_TO_GRAY_ARGB => {
                let argument = BlendToGray::of(argument);
                let subject = subject.as_argb();
                let multiplied_greyscale = ARGB::scale_rgb_vector4f(
                    &ARGB::greyscale_vector4f(&subject),
                    argument.brightness,
                );
                AttributeValue::Argb(ARGB::srgb_lerp_vector4f(
                    argument.factor,
                    &subject,
                    &multiplied_greyscale,
                ))
            }
        }
    }

    fn is_argb_modifier(&self) -> bool {
        matches!(
            self,
            ColorModifier::ALPHA_BLEND_RGB
                | ColorModifier::ALPHA_BLEND_ARGB
                | ColorModifier::MULTIPLY_ARGB
        )
    }

    pub fn decode_argument(&self, json: &JsonElement) -> Result<AttributeValue, String> {
        match self {
            ColorModifier::BLEND_TO_GRAY_RGB | ColorModifier::BLEND_TO_GRAY_ARGB => {
                BlendToGray::from_json(json).map(AttributeValue::BlendToGray)
            }
            _ if self.is_argb_modifier() => decode_argb(json).map(AttributeValue::Argb),
            _ => decode_rgb(json).map(AttributeValue::Rgb),
        }
    }

    pub fn argument_keyframe_lerp(&self) -> LerpFunction {
        match self {
            ColorModifier::BLEND_TO_GRAY_RGB | ColorModifier::BLEND_TO_GRAY_ARGB => {
                LerpFunction::BlendToGray
            }
            _ if self.is_argb_modifier() => LerpFunction::ColorVec4,
            _ => LerpFunction::ColorVec3,
        }
    }

    /// `AttributeModifier.RGB_COLOR_LIBRARY`.
    pub fn rgb_by_operation(operation: OperationId) -> Option<ColorModifier> {
        match operation {
            OperationId::ALPHA_BLEND => Some(ColorModifier::ALPHA_BLEND_RGB),
            OperationId::ADD => Some(ColorModifier::ADD_RGB),
            OperationId::SUBTRACT => Some(ColorModifier::SUBTRACT_RGB),
            OperationId::MULTIPLY => Some(ColorModifier::MULTIPLY_RGB),
            OperationId::BLEND_TO_GRAY => Some(ColorModifier::BLEND_TO_GRAY_RGB),
            _ => None,
        }
    }

    /// `AttributeModifier.ARGB_COLOR_LIBRARY`.
    pub fn argb_by_operation(operation: OperationId) -> Option<ColorModifier> {
        match operation {
            OperationId::ALPHA_BLEND => Some(ColorModifier::ALPHA_BLEND_ARGB),
            OperationId::ADD => Some(ColorModifier::ADD_ARGB),
            OperationId::SUBTRACT => Some(ColorModifier::SUBTRACT_ARGB),
            OperationId::MULTIPLY => Some(ColorModifier::MULTIPLY_ARGB),
            OperationId::BLEND_TO_GRAY => Some(ColorModifier::BLEND_TO_GRAY_ARGB),
            _ => None,
        }
    }
}

/// `ColorModifier.BlendToGray`: how bright the gray is, and how far toward it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendToGray {
    pub brightness: f32,
    pub factor: f32,
}

impl BlendToGray {
    pub fn new(brightness: f32, factor: f32) -> BlendToGray {
        BlendToGray { brightness, factor }
    }

    fn of(argument: &AttributeValue) -> BlendToGray {
        match argument {
            AttributeValue::BlendToGray(argument) => *argument,
            other => panic!("ClassCastException: {other:?} isn't a BlendToGray"),
        }
    }

    pub fn lerp(alpha: f32, from: &BlendToGray, to: &BlendToGray) -> BlendToGray {
        BlendToGray::new(
            Mth::lerp_f32(alpha, from.brightness, to.brightness),
            Mth::lerp_f32(alpha, from.factor, to.factor),
        )
    }

    /// `CODEC`: `brightness` and `factor`, each in 0..1.
    pub fn from_json(json: &JsonElement) -> Result<BlendToGray, String> {
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("Not an object: {}", json))?;
        let unit = |name: &str| -> Result<f32, String> {
            let value = decode_float(
                object
                    .get(name)
                    .ok_or(format!("No key {name} in MapLike"))?,
            )?;
            if !(0.0..=1.0).contains(&value) {
                return Err(format!("Value {value} outside of range [0.0:1.0]"));
            }
            Ok(value)
        };
        Ok(BlendToGray::new(unit("brightness")?, unit("factor")?))
    }
}
