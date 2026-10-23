//! com/google/gson/JsonElement.java, JsonObject.java, JsonArray.java, JsonPrimitive.java and
//! JsonNull.java (Gson 2.14.0)
//!
//! Gson's tree, as the game reads models and block states from it. A number keeps its text
//! (`LazilyParsedNumber`), which each getter parses into its type, and an object keeps its members
//! in the order they came (Gson's `LinkedTreeMap`), a later member of the same name replacing the
//! earlier one's value in its place. Written from Gson's behaviour: its sources aren't among the
//! game's. Java's getters throw `IllegalStateException` or `UnsupportedOperationException` on
//! the wrong kind of element, where these panic.

use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonElement {
    Null,
    Primitive(JsonPrimitive),
    Array(Vec<JsonElement>),
    Object(JsonObject),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonPrimitive {
    Boolean(bool),
    Number(LazilyParsedNumber),
    String(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonObject {
    members: Vec<(String, JsonElement)>,
}

impl JsonElement {
    pub fn is_json_null(&self) -> bool {
        matches!(self, JsonElement::Null)
    }

    pub fn is_json_primitive(&self) -> bool {
        matches!(self, JsonElement::Primitive(_))
    }

    pub fn is_json_array(&self) -> bool {
        matches!(self, JsonElement::Array(_))
    }

    pub fn is_json_object(&self) -> bool {
        matches!(self, JsonElement::Object(_))
    }

    pub fn as_json_primitive(&self) -> Option<&JsonPrimitive> {
        match self {
            JsonElement::Primitive(primitive) => Some(primitive),
            _ => None,
        }
    }

    pub fn as_json_object(&self) -> Option<&JsonObject> {
        match self {
            JsonElement::Object(object) => Some(object),
            _ => None,
        }
    }

    pub fn as_json_array(&self) -> Option<&[JsonElement]> {
        match self {
            JsonElement::Array(array) => Some(array),
            _ => None,
        }
    }

    /// `getAsJsonObject()`: Java throws `IllegalStateException` for another kind.
    pub fn get_as_json_object(&self) -> &JsonObject {
        self.as_json_object()
            .unwrap_or_else(|| panic!("IllegalStateException: Not a JSON Object: {self:?}"))
    }

    fn primitive(&self) -> &JsonPrimitive {
        match self {
            JsonElement::Primitive(primitive) => primitive,
            // A JSON array of one element answers for it, like Gson's JsonArray.
            JsonElement::Array(array) if array.len() == 1 => array[0].primitive(),
            _ => panic!("UnsupportedOperationException: {self:?}"),
        }
    }

    pub fn get_as_string(&self) -> String {
        self.primitive().get_as_string()
    }

    pub fn get_as_float(&self) -> f32 {
        self.primitive().get_as_float()
    }

    pub fn get_as_int(&self) -> i32 {
        self.primitive().get_as_int()
    }

    pub fn get_as_boolean(&self) -> bool {
        self.primitive().get_as_boolean()
    }

    /// Java's `toString()`, for messages: compact JSON.
    pub fn to_json_string(&self) -> String {
        match self {
            JsonElement::Null => "null".to_owned(),
            JsonElement::Primitive(JsonPrimitive::Boolean(value)) => value.to_string(),
            JsonElement::Primitive(JsonPrimitive::Number(number)) => number.to_string(),
            JsonElement::Primitive(JsonPrimitive::String(string)) => {
                serde_json::Value::from(string.as_str()).to_string()
            }
            JsonElement::Array(elements) => format!(
                "[{}]",
                elements
                    .iter()
                    .map(JsonElement::to_json_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            JsonElement::Object(object) => format!(
                "{{{}}}",
                object
                    .entry_set()
                    .map(|(name, value)| format!(
                        "{}:{}",
                        serde_json::Value::from(name.as_str()),
                        value.to_json_string()
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

impl JsonPrimitive {
    pub fn is_boolean(&self) -> bool {
        matches!(self, JsonPrimitive::Boolean(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, JsonPrimitive::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, JsonPrimitive::String(_))
    }

    /// A number's text as it was read, or a boolean's name.
    pub fn get_as_string(&self) -> String {
        match self {
            JsonPrimitive::Boolean(value) => value.to_string(),
            JsonPrimitive::Number(number) => number.to_string(),
            JsonPrimitive::String(string) => string.clone(),
        }
    }

    /// A number's `floatValue()`, or `Float.parseFloat` of the text.
    pub fn get_as_float(&self) -> f32 {
        match self {
            JsonPrimitive::Number(number) => number.float_value(),
            _ => LazilyParsedNumber::new(self.get_as_string()).float_value(),
        }
    }

    /// A number's `intValue()`, or `Integer.parseInt` of the text.
    pub fn get_as_int(&self) -> i32 {
        match self {
            JsonPrimitive::Number(number) => number.int_value(),
            _ => {
                let text = self.get_as_string();
                text.parse().unwrap_or_else(|_| {
                    panic!("NumberFormatException: For input string: \"{text}\"")
                })
            }
        }
    }

    /// A boolean, or `Boolean.parseBoolean` of the text: whether it's "true" in any case.
    pub fn get_as_boolean(&self) -> bool {
        match self {
            JsonPrimitive::Boolean(value) => *value,
            _ => self.get_as_string().eq_ignore_ascii_case("true"),
        }
    }
}

impl JsonObject {
    pub fn new() -> JsonObject {
        JsonObject::default()
    }

    /// `add`: a member already there keeps its place and takes the new value.
    pub fn add(&mut self, property: String, value: JsonElement) {
        match self.members.iter_mut().find(|(name, _)| *name == property) {
            Some(member) => member.1 = value,
            None => self.members.push((property, value)),
        }
    }

    pub fn has(&self, member_name: &str) -> bool {
        self.get(member_name).is_some()
    }

    pub fn get(&self, member_name: &str) -> Option<&JsonElement> {
        self.members
            .iter()
            .find(|(name, _)| name == member_name)
            .map(|(_, value)| value)
    }

    /// The members in the order they came.
    pub fn entry_set(&self) -> impl Iterator<Item = (&String, &JsonElement)> {
        self.members.iter().map(|(name, value)| (name, value))
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}
