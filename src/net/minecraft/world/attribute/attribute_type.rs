//! net/minecraft/world/attribute/AttributeType.java, with AttributeTypes.java's constants
//!
//! An attribute's type: how its values decode, which modifiers it takes, and how its values blend.
//! Java's record holds a codec, a modifier library and four `LerpFunction`s; here the type is an
//! enum of the `AttributeTypes` constants the port has, which those match on. Not ported:
//! `TRI_STATE`, `INTEGER`, `ACTIVITY`, `BED_RULE`, `PARTICLE`, `AMBIENT_PARTICLES`,
//! `BACKGROUND_MUSIC`, `AMBIENT_SOUNDS` and `MOB_SPAWN_SETTINGS`, whose attributes
//! `EnvironmentAttributes` names but leaves out, and `toFloat`/`toInt`.
//!
//! An attribute's values and its modifiers' arguments are Java's type arguments `Value` and
//! `Argument`; here they're one enum, `AttributeValue`, whose kind the attribute's type and the
//! modifier decide, and `Value` turns the kinds a typed attribute hands out back into Rust types.

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::world::attribute::lerp_function::LerpFunction;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::{
    AttributeModifier, OperationId,
};
use crate::net::minecraft::world::attribute::modifier::boolean_modifier::BooleanModifier;
use crate::net::minecraft::world::attribute::modifier::color_modifier::{
    BlendToGray, ColorModifier,
};
use crate::net::minecraft::world::attribute::modifier::float_modifier::FloatModifier;
use crate::net::minecraft::world::attribute::modifier::float_with_alpha::FloatWithAlpha;
use crate::net::minecraft::world::level::moon_phase::MoonPhase;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

/// `AttributeTypes`' constants the port has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeType {
    BOOLEAN,
    FLOAT,
    ANGLE_DEGREES,
    RGB_COLOR,
    ARGB_COLOR,
    MOON_PHASE,
}

/// A value an attribute holds, or a modifier's argument.
#[derive(Clone, Copy, Debug)]
pub enum AttributeValue {
    Boolean(bool),
    Float(f32),
    Rgb(Vector3f),
    Argb(Vector4f),
    MoonPhase(MoonPhase),
    FloatWithAlpha(FloatWithAlpha),
    BlendToGray(BlendToGray),
}

impl AttributeValue {
    pub fn as_boolean(&self) -> bool {
        match self {
            AttributeValue::Boolean(value) => *value,
            other => panic!("ClassCastException: {other:?} isn't a boolean"),
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            AttributeValue::Float(value) => *value,
            other => panic!("ClassCastException: {other:?} isn't a float"),
        }
    }

    pub fn as_rgb(&self) -> Vector3f {
        match self {
            AttributeValue::Rgb(value) => *value,
            other => panic!("ClassCastException: {other:?} isn't an RGB color"),
        }
    }

    pub fn as_argb(&self) -> Vector4f {
        match self {
            AttributeValue::Argb(value) => *value,
            other => panic!("ClassCastException: {other:?} isn't an ARGB color"),
        }
    }

    pub fn as_moon_phase(&self) -> MoonPhase {
        match self {
            AttributeValue::MoonPhase(value) => *value,
            other => panic!("ClassCastException: {other:?} isn't a moon phase"),
        }
    }
}

/// The Rust type of a typed attribute's values (`EnvironmentAttribute<Value>`).
pub trait Value: Sized {
    fn from_attribute_value(value: AttributeValue) -> Self;
}

impl Value for bool {
    fn from_attribute_value(value: AttributeValue) -> bool {
        value.as_boolean()
    }
}

impl Value for f32 {
    fn from_attribute_value(value: AttributeValue) -> f32 {
        value.as_float()
    }
}

impl Value for Vector3f {
    fn from_attribute_value(value: AttributeValue) -> Vector3f {
        value.as_rgb()
    }
}

impl Value for Vector4f {
    fn from_attribute_value(value: AttributeValue) -> Vector4f {
        value.as_argb()
    }
}

impl Value for MoonPhase {
    fn from_attribute_value(value: AttributeValue) -> MoonPhase {
        value.as_moon_phase()
    }
}

impl AttributeType {
    /// `ofInterpolated` for the floats, angles and colors, `ofNotInterpolated` (steps) for the
    /// rest.
    pub fn keyframe_lerp(&self) -> LerpFunction {
        self.lerp().unwrap_or(LerpFunction::Step(1.0))
    }

    pub fn state_change_lerp(&self) -> LerpFunction {
        self.lerp().unwrap_or(LerpFunction::Step(0.0))
    }

    pub fn spatial_lerp(&self) -> LerpFunction {
        self.lerp().unwrap_or(LerpFunction::Step(0.5))
    }

    pub fn partial_tick_lerp(&self) -> LerpFunction {
        match self {
            AttributeType::ANGLE_DEGREES => LerpFunction::Degrees(90.0),
            _ => self.lerp().unwrap_or(LerpFunction::Step(0.0)),
        }
    }

    /// The lerp an interpolated type blends with everywhere but between ticks.
    fn lerp(&self) -> Option<LerpFunction> {
        match self {
            AttributeType::FLOAT | AttributeType::ANGLE_DEGREES => Some(LerpFunction::Float),
            AttributeType::RGB_COLOR => Some(LerpFunction::ColorVec3),
            AttributeType::ARGB_COLOR => Some(LerpFunction::ColorVec4),
            AttributeType::BOOLEAN | AttributeType::MOON_PHASE => None,
        }
    }

    /// The modifier of the operation, from the type's library (`modifierCodec`): override for
    /// every type, or none the type takes.
    pub fn modifier(&self, operation: OperationId) -> Option<AttributeModifier> {
        if operation == OperationId::OVERRIDE {
            return Some(AttributeModifier::Override);
        }
        match self {
            AttributeType::BOOLEAN => {
                BooleanModifier::by_operation(operation).map(AttributeModifier::Boolean)
            }
            AttributeType::FLOAT | AttributeType::ANGLE_DEGREES => {
                FloatModifier::by_operation(operation).map(AttributeModifier::Float)
            }
            AttributeType::RGB_COLOR => {
                ColorModifier::rgb_by_operation(operation).map(AttributeModifier::Color)
            }
            AttributeType::ARGB_COLOR => {
                ColorModifier::argb_by_operation(operation).map(AttributeModifier::Color)
            }
            AttributeType::MOON_PHASE => None,
        }
    }

    /// `valueCodec`, through `JsonOps`.
    pub fn decode_value(&self, json: &JsonElement) -> Result<AttributeValue, String> {
        Ok(match self {
            AttributeType::BOOLEAN => AttributeValue::Boolean(decode_bool(json)?),
            AttributeType::FLOAT | AttributeType::ANGLE_DEGREES => {
                AttributeValue::Float(decode_float(json)?)
            }
            AttributeType::RGB_COLOR => AttributeValue::Rgb(decode_rgb(json)?),
            AttributeType::ARGB_COLOR => AttributeValue::Argb(decode_argb(json)?),
            AttributeType::MOON_PHASE => {
                let name = decode_string(json)?;
                AttributeValue::MoonPhase(
                    MoonPhase::by_serialized_name(&name)
                        .ok_or_else(|| format!("Unknown element name:{name}"))?,
                )
            }
        })
    }
}

fn decode_string(json: &JsonElement) -> Result<String, String> {
    match json.as_json_primitive() {
        Some(primitive) if primitive.is_string() => Ok(primitive.get_as_string()),
        _ => Err(format!("Not a string: {}", json.to_json_string())),
    }
}

/// `Codec.FLOAT`: a number's `floatValue()`, `Float.parseFloat` of its text.
pub(crate) fn decode_float(json: &JsonElement) -> Result<f32, String> {
    match json.as_json_primitive() {
        Some(primitive) if primitive.is_number() => Ok(primitive.get_as_float()),
        _ => Err(format!("Not a number: {}", json.to_json_string())),
    }
}

/// `Codec.INT`: a number's `intValue()`.
pub(crate) fn decode_int(json: &JsonElement) -> Result<i32, String> {
    match json.as_json_primitive() {
        Some(primitive) if primitive.is_number() => Ok(primitive.get_as_int()),
        _ => Err(format!("Not a number: {}", json.to_json_string())),
    }
}

/// `Codec.BOOL`: a boolean, or a number's byte that isn't 0.
pub(crate) fn decode_bool(json: &JsonElement) -> Result<bool, String> {
    match json.as_json_primitive() {
        Some(primitive) if primitive.is_boolean() => Ok(primitive.get_as_boolean()),
        Some(primitive) if primitive.is_number() => Ok(primitive.get_as_int() as i8 != 0),
        _ => Err(format!("Not a boolean: {}", json.to_json_string())),
    }
}

/// `ExtraCodecs.hexColor(digits)`: `#` and exactly that many hex digits.
fn decode_hex_color(json: &JsonElement, digits: usize) -> Result<i32, String> {
    let text = decode_string(json)?;
    let hex = text
        .strip_prefix('#')
        .ok_or("Hex color must begin with #")?;
    if hex.chars().count() != digits {
        return Err(format!(
            "Hex color is wrong size, expected {digits} digits but got {}",
            hex.chars().count()
        ));
    }
    u32::from_str_radix(hex, 16)
        .map(|value| value as i32)
        .map_err(|_| format!("Invalid color value: {text}"))
}

/// `Util.fixedSize` of `Codec.FLOAT.listOf()`.
fn decode_floats<const N: usize>(json: &JsonElement) -> Result<[f32; N], String> {
    let array = json
        .as_json_array()
        .ok_or_else(|| format!("Not a list: {}", json.to_json_string()))?;
    if array.len() != N {
        return Err(format!("Input is not a list of {N} elements"));
    }
    let mut floats = [0.0; N];
    for (float, element) in floats.iter_mut().zip(array) {
        *float = decode_float(element)?;
    }
    Ok(floats)
}

/// `ExtraCodecs.STRING_RGB_VEC3_COLOR`: `#rrggbb`, or `RGB_COLOR_VEC3_CODEC`'s three floats or
/// int.
pub(crate) fn decode_rgb(json: &JsonElement) -> Result<Vector3f, String> {
    if let Ok(color) = decode_hex_color(json, 6) {
        return Ok(ARGB::vector3f_from_rgb24(ARGB::opaque(color)));
    }
    if let Ok([x, y, z]) = decode_floats::<3>(json) {
        return Ok(Vector3f::from_xyz(x, y, z));
    }
    Ok(ARGB::vector3f_from_rgb24(decode_int(json)?))
}

/// `ExtraCodecs.STRING_ARGB_VEC4_COLOR`: `#aarrggbb`, or `ARGB_COLOR_VEC4_CODEC`'s four floats or
/// int.
pub(crate) fn decode_argb(json: &JsonElement) -> Result<Vector4f, String> {
    if let Ok(color) = decode_hex_color(json, 8) {
        return Ok(ARGB::vector4f_from_argb32(color));
    }
    if let Ok([x, y, z, w]) = decode_floats::<4>(json) {
        return Ok(Vector4f::from_xyzw(x, y, z, w));
    }
    Ok(ARGB::vector4f_from_argb32(decode_int(json)?))
}
