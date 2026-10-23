//! net/minecraft/world/attribute/modifier/AttributeModifier.java
//!
//! How an attribute map's entry or a timeline's track changes the value under it: Java's
//! interface, implemented only by its library's constants and `OverrideModifier`, is an enum of
//! them. Each constant's library (`FLOAT_LIBRARY`, `RGB_COLOR_LIBRARY`, …) is
//! `AttributeType::modifier`. Not ported: `INTEGER_LIBRARY`, `listLibrary` (`append`) and
//! `MOB_SPAWN_SETTINGS_LIBRARY` (`overlay`), whose types aren't.

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::world::attribute::attribute_type::{AttributeType, AttributeValue};
use crate::net::minecraft::world::attribute::lerp_function::LerpFunction;
use crate::net::minecraft::world::attribute::modifier::boolean_modifier::BooleanModifier;
use crate::net::minecraft::world::attribute::modifier::color_modifier::ColorModifier;
use crate::net::minecraft::world::attribute::modifier::float_modifier::FloatModifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeModifier {
    /// `OverrideModifier`: the argument, whatever was under it.
    Override,
    Boolean(BooleanModifier),
    Float(FloatModifier),
    Color(ColorModifier),
}

impl AttributeModifier {
    pub fn apply(&self, subject: AttributeValue, argument: &AttributeValue) -> AttributeValue {
        match self {
            AttributeModifier::Override => *argument,
            AttributeModifier::Boolean(modifier) => {
                AttributeValue::Boolean(modifier.apply(subject.as_boolean(), argument.as_boolean()))
            }
            AttributeModifier::Float(modifier) => modifier.apply(subject.as_float(), argument),
            AttributeModifier::Color(modifier) => modifier.apply(subject, argument),
        }
    }

    /// `argumentCodec(attribute)`, through `JsonOps`: the attribute's own values for an override.
    /// `validate_value` checks an override's value against the attribute's range.
    pub fn decode_argument(
        &self,
        r#type: AttributeType,
        json: &JsonElement,
    ) -> Result<AttributeValue, String> {
        match self {
            AttributeModifier::Override => r#type.decode_value(json),
            AttributeModifier::Boolean(_) => r#type.decode_value(json),
            AttributeModifier::Float(modifier) => modifier.decode_argument(json),
            AttributeModifier::Color(modifier) => modifier.decode_argument(json),
        }
    }

    /// `argumentKeyframeLerp(attribute)`: how a track blends this modifier's arguments.
    pub fn argument_keyframe_lerp(&self, r#type: AttributeType) -> LerpFunction {
        match self {
            AttributeModifier::Override => r#type.keyframe_lerp(),
            AttributeModifier::Boolean(_) => LerpFunction::CONSTANT,
            AttributeModifier::Float(modifier) => modifier.argument_keyframe_lerp(),
            AttributeModifier::Color(modifier) => modifier.argument_keyframe_lerp(),
        }
    }
}

/// `AttributeModifier.OperationId`: a modifier's name in data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationId {
    OVERRIDE,
    ALPHA_BLEND,
    ADD,
    SUBTRACT,
    MULTIPLY,
    BLEND_TO_GRAY,
    MINIMUM,
    MAXIMUM,
    AND,
    NAND,
    OR,
    NOR,
    XOR,
    XNOR,
    APPEND,
    OVERLAY,
}

impl OperationId {
    pub const VALUES: [OperationId; 16] = [
        OperationId::OVERRIDE,
        OperationId::ALPHA_BLEND,
        OperationId::ADD,
        OperationId::SUBTRACT,
        OperationId::MULTIPLY,
        OperationId::BLEND_TO_GRAY,
        OperationId::MINIMUM,
        OperationId::MAXIMUM,
        OperationId::AND,
        OperationId::NAND,
        OperationId::OR,
        OperationId::NOR,
        OperationId::XOR,
        OperationId::XNOR,
        OperationId::APPEND,
        OperationId::OVERLAY,
    ];

    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            OperationId::OVERRIDE => "override",
            OperationId::ALPHA_BLEND => "alpha_blend",
            OperationId::ADD => "add",
            OperationId::SUBTRACT => "subtract",
            OperationId::MULTIPLY => "multiply",
            OperationId::BLEND_TO_GRAY => "blend_to_gray",
            OperationId::MINIMUM => "minimum",
            OperationId::MAXIMUM => "maximum",
            OperationId::AND => "and",
            OperationId::NAND => "nand",
            OperationId::OR => "or",
            OperationId::NOR => "nor",
            OperationId::XOR => "xor",
            OperationId::XNOR => "xnor",
            OperationId::APPEND => "append",
            OperationId::OVERLAY => "overlay",
        }
    }

    /// `CODEC`, `StringRepresentable.fromEnum`'s name lookup.
    pub fn by_serialized_name(name: &str) -> Option<OperationId> {
        OperationId::VALUES
            .into_iter()
            .find(|operation| operation.get_serialized_name() == name)
    }

    /// `AttributeType.modifierCodec`: the modifier named in `json` that `type` takes.
    pub fn decode_modifier(
        r#type: AttributeType,
        json: &JsonElement,
    ) -> Result<AttributeModifier, String> {
        let name = json
            .as_json_primitive()
            .filter(|primitive| primitive.is_string())
            .map(|primitive| primitive.get_as_string())
            .ok_or_else(|| format!("Not a string: {}", json.to_json_string()))?;
        let operation = OperationId::by_serialized_name(&name)
            .ok_or_else(|| format!("Unknown element name:{name}"))?;
        r#type
            .modifier(operation)
            .ok_or_else(|| format!("Unknown element id: {name}"))
    }
}
