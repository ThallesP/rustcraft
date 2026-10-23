//! net/minecraft/util/GsonHelper.java
//!
//! Only the getters models are read with. Java throws `JsonSyntaxException` for a member that's
//! missing or of the wrong kind, which is the `Err` here, with Java's message.

use crate::com::google::gson::json_element::{JsonElement, JsonObject};

pub struct GsonHelper;

impl GsonHelper {
    pub fn is_string_value(node: &JsonElement) -> bool {
        node.as_json_primitive()
            .is_some_and(|primitive| primitive.is_string())
    }

    pub fn is_number_value(node: &JsonObject, name: &str) -> bool {
        node.get(name)
            .and_then(JsonElement::as_json_primitive)
            .is_some_and(|primitive| primitive.is_number())
    }

    pub fn convert_to_string(element: &JsonElement, name: &str) -> Result<String, String> {
        if element.is_json_primitive() {
            Ok(element.get_as_string())
        } else {
            Err(format!(
                "Expected {name} to be a string, was {}",
                GsonHelper::get_type(element)
            ))
        }
    }

    pub fn get_as_string(object: &JsonObject, name: &str) -> Result<String, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_string(element, name),
            None => Err(format!("Missing {name}, expected to find a string")),
        }
    }

    pub fn get_as_string_or(object: &JsonObject, name: &str, def: &str) -> Result<String, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_string(element, name),
            None => Ok(def.to_owned()),
        }
    }

    pub fn convert_to_boolean(element: &JsonElement, name: &str) -> Result<bool, String> {
        if element.is_json_primitive() {
            Ok(element.get_as_boolean())
        } else {
            Err(format!(
                "Expected {name} to be a Boolean, was {}",
                GsonHelper::get_type(element)
            ))
        }
    }

    pub fn get_as_boolean(object: &JsonObject, name: &str) -> Result<bool, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_boolean(element, name),
            None => Err(format!("Missing {name}, expected to find a Boolean")),
        }
    }

    pub fn get_as_boolean_or(object: &JsonObject, name: &str, def: bool) -> Result<bool, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_boolean(element, name),
            None => Ok(def),
        }
    }

    fn is_number(element: &JsonElement) -> bool {
        element
            .as_json_primitive()
            .is_some_and(|primitive| primitive.is_number())
    }

    pub fn convert_to_float(element: &JsonElement, name: &str) -> Result<f32, String> {
        if GsonHelper::is_number(element) {
            Ok(element.get_as_float())
        } else {
            Err(format!(
                "Expected {name} to be a Float, was {}",
                GsonHelper::get_type(element)
            ))
        }
    }

    pub fn get_as_float(object: &JsonObject, name: &str) -> Result<f32, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_float(element, name),
            None => Err(format!("Missing {name}, expected to find a Float")),
        }
    }

    pub fn get_as_float_or(object: &JsonObject, name: &str, def: f32) -> Result<f32, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_float(element, name),
            None => Ok(def),
        }
    }

    pub fn convert_to_int(element: &JsonElement, name: &str) -> Result<i32, String> {
        if GsonHelper::is_number(element) {
            Ok(element.get_as_int())
        } else {
            Err(format!(
                "Expected {name} to be a Int, was {}",
                GsonHelper::get_type(element)
            ))
        }
    }

    pub fn get_as_int(object: &JsonObject, name: &str) -> Result<i32, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_int(element, name),
            None => Err(format!("Missing {name}, expected to find a Int")),
        }
    }

    pub fn get_as_int_or(object: &JsonObject, name: &str, def: i32) -> Result<i32, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_int(element, name),
            None => Ok(def),
        }
    }

    pub fn convert_to_json_object<'a>(
        element: &'a JsonElement,
        name: &str,
    ) -> Result<&'a JsonObject, String> {
        element.as_json_object().ok_or_else(|| {
            format!(
                "Expected {name} to be a JsonObject, was {}",
                GsonHelper::get_type(element)
            )
        })
    }

    pub fn get_as_json_object<'a>(
        object: &'a JsonObject,
        name: &str,
    ) -> Result<&'a JsonObject, String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_json_object(element, name),
            None => Err(format!("Missing {name}, expected to find a JsonObject")),
        }
    }

    pub fn convert_to_json_array<'a>(
        element: &'a JsonElement,
        name: &str,
    ) -> Result<&'a [JsonElement], String> {
        element.as_json_array().ok_or_else(|| {
            format!(
                "Expected {name} to be a JsonArray, was {}",
                GsonHelper::get_type(element)
            )
        })
    }

    pub fn get_as_json_array<'a>(
        object: &'a JsonObject,
        name: &str,
    ) -> Result<&'a [JsonElement], String> {
        match object.get(name) {
            Some(element) => GsonHelper::convert_to_json_array(element, name),
            None => Err(format!("Missing {name}, expected to find a JsonArray")),
        }
    }

    /// What an element is, for messages. Java abbreviates the element's text to 10 characters.
    pub fn get_type(element: &JsonElement) -> String {
        let text = element.to_json_string();
        let value = if text.chars().count() > 10 {
            let chars: Vec<char> = text.chars().collect();
            let start: String = chars[..4].iter().collect();
            let end: String = chars[chars.len() - 3..].iter().collect();
            format!("{start}...{end}")
        } else {
            text
        };
        match element {
            JsonElement::Null => "null (json)".to_owned(),
            JsonElement::Array(_) => format!("an array ({value})"),
            JsonElement::Object(_) => format!("an object ({value})"),
            JsonElement::Primitive(primitive) if primitive.is_number() => {
                format!("a number ({value})")
            }
            JsonElement::Primitive(primitive) if primitive.is_boolean() => {
                format!("a boolean ({value})")
            }
            JsonElement::Primitive(_) => value,
        }
    }
}
