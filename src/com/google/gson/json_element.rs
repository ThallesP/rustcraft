//! com/google/gson/JsonElement.java, with JsonArray.java and JsonNull.java (Gson 2.14.0)
//!
//! Gson's tree: a JSON null (`JsonNull.INSTANCE`), a primitive (`JsonPrimitive`), an array or an
//! object (`JsonObject`), the abstract class's four subclasses as an enum. A `JsonArray` is a
//! `Vec` of its elements, which is all its `ArrayList` is: its getters ask its single element, and
//! throw `IllegalStateException` for another size. Java's getters throw `IllegalStateException`
//! or `UnsupportedOperationException` on the wrong kind of element, where these panic, with
//! Java's `toString()` of the exception.
//!
//! `equals` is `PartialEq`: an array's is its list's (equal elements in order), an object's its
//! map's (in any order), a primitive's `JsonPrimitive`'s. `toString()` is `Display`: the JSON, as a
//! lenient `JsonWriter` that writes nulls writes it, which panics like Java's for a
//! `LazilyParsedNumber` whose text isn't a JSON number. `hashCode` isn't ported: `JsonNull`'s is
//! an identity hash.

/*
 * Copyright (C) 2008 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust from Gson 2.14.0, and changed: see above.
 */

use std::fmt;

use crate::com::google::gson::internal::streams::Streams;
pub use crate::com::google::gson::json_object::JsonObject;
use crate::com::google::gson::json_primitive::JsonPrimitive;
use crate::com::google::gson::stream::json_writer::JsonWriter;
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::number::Number;
use crate::java::lang::throwable::ILLEGAL_STATE_EXCEPTION;
use crate::java::math::big_decimal::BigDecimal;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonElement {
    /// `JsonNull.INSTANCE`.
    Null,
    Primitive(JsonPrimitive),
    /// A `JsonArray`'s elements.
    Array(Vec<JsonElement>),
    Object(JsonObject),
}

impl JsonElement {
    /// `deepCopy()`: arrays and objects copied all the way down, primitives shared (a clone).
    pub fn deep_copy(&self) -> JsonElement {
        self.clone()
    }

    pub fn is_json_array(&self) -> bool {
        matches!(self, JsonElement::Array(_))
    }

    pub fn is_json_object(&self) -> bool {
        matches!(self, JsonElement::Object(_))
    }

    pub fn is_json_primitive(&self) -> bool {
        matches!(self, JsonElement::Primitive(_))
    }

    pub fn is_json_null(&self) -> bool {
        matches!(self, JsonElement::Null)
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

    pub fn as_json_object_mut(&mut self) -> Option<&mut JsonObject> {
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

    pub fn as_json_array_mut(&mut self) -> Option<&mut Vec<JsonElement>> {
        match self {
            JsonElement::Array(array) => Some(array),
            _ => None,
        }
    }

    /// `getAsJsonObject()`: Java throws `IllegalStateException` for another kind.
    pub fn get_as_json_object(&self) -> &JsonObject {
        self.as_json_object()
            .unwrap_or_else(|| panic!("{ILLEGAL_STATE_EXCEPTION}: Not a JSON Object: {self}"))
    }

    /// `getAsJsonArray()`: Java throws `IllegalStateException` for another kind.
    pub fn get_as_json_array(&self) -> &[JsonElement] {
        self.as_json_array()
            .unwrap_or_else(|| panic!("{ILLEGAL_STATE_EXCEPTION}: Not a JSON Array: {self}"))
    }

    /// `getAsJsonPrimitive()`: Java throws `IllegalStateException` for another kind.
    pub fn get_as_json_primitive(&self) -> &JsonPrimitive {
        self.as_json_primitive()
            .unwrap_or_else(|| panic!("{ILLEGAL_STATE_EXCEPTION}: Not a JSON Primitive: {self}"))
    }

    /// The primitive the getters ask: the element itself, or an array's single element. Java's
    /// `JsonNull` and `JsonObject` throw `UnsupportedOperationException` naming their class, and an
    /// array of another size `IllegalStateException`.
    fn primitive(&self) -> &JsonPrimitive {
        match self {
            JsonElement::Primitive(primitive) => primitive,
            JsonElement::Array(array) if array.len() == 1 => array[0].primitive(),
            JsonElement::Array(array) => panic!(
                "{ILLEGAL_STATE_EXCEPTION}: Array must have size 1, but has size {}",
                array.len()
            ),
            JsonElement::Null => panic!("java.lang.UnsupportedOperationException: JsonNull"),
            JsonElement::Object(_) => panic!("java.lang.UnsupportedOperationException: JsonObject"),
        }
    }

    pub fn get_as_boolean(&self) -> bool {
        self.primitive().get_as_boolean()
    }

    pub fn get_as_number(&self) -> Number {
        self.primitive().get_as_number()
    }

    pub fn get_as_string(&self) -> String {
        self.primitive().get_as_string()
    }

    pub fn get_as_double(&self) -> f64 {
        self.primitive().get_as_double()
    }

    pub fn get_as_float(&self) -> f32 {
        self.primitive().get_as_float()
    }

    pub fn get_as_long(&self) -> i64 {
        self.primitive().get_as_long()
    }

    pub fn get_as_int(&self) -> i32 {
        self.primitive().get_as_int()
    }

    pub fn get_as_byte(&self) -> i8 {
        self.primitive().get_as_byte()
    }

    pub fn get_as_big_decimal(&self) -> BigDecimal {
        self.primitive().get_as_big_decimal()
    }

    pub fn get_as_short(&self) -> i16 {
        self.primitive().get_as_short()
    }
}

/// `toString()`: the element's JSON, written by a lenient writer (so NaN and infinities are
/// written) that writes null members.
impl fmt::Display for JsonElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string_builder = String::new();
        let mut json_writer = JsonWriter::new(&mut string_builder);
        json_writer.set_strictness(Strictness::LENIENT);
        Streams::write(self, &mut json_writer).unwrap_or_else(|error| panic!("{error}"));
        f.write_str(&string_builder)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::JsonElement;
    use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;
    use crate::com::google::gson::json_object::JsonObject;
    use crate::com::google::gson::json_primitive::JsonPrimitive;
    use crate::golden::{Decode, Encode, Golden, catch};
    use crate::java::lang::byte::Byte;
    use crate::java::lang::double::Double;
    use crate::java::lang::float::Float;
    use crate::java::lang::integer::Integer;
    use crate::java::lang::long::Long;
    use crate::java::lang::number::Number;
    use crate::java::lang::short::Short;
    use crate::java::math::big_decimal::BigDecimal;

    /// The class's simple name, which the suites record a number's box by.
    pub(crate) fn simple_name(number: &Number) -> &'static str {
        let class_name = number.class_name();
        &class_name[class_name.rfind('.').expect("a package") + 1..]
    }

    /// What a call panicked with, Java's `toString()` of the exception, as the suites record an
    /// exception: `[class, message]`, the message "null" when there's none.
    pub(crate) fn thrown(call: impl FnOnce()) -> (String, String) {
        let message = catch(call).expect_err("vanilla threw");
        match message.split_once(": ") {
            Some((class, message)) => (class.to_owned(), message.to_owned()),
            None => (message, "null".to_owned()),
        }
    }

    // `["null"]`, `["boolean", b]`, `["number", box, toString]`, `["string", s]`,
    // `["array", [element, …]]` or `["object", [[name, element], …]]`, as GsonSuites encodes them.
    impl Decode for JsonElement {
        fn decode(value: &Value) -> JsonElement {
            let spec = value.as_array().expect("an element");
            let text = |index: usize| String::decode(&spec[index]);
            match spec[0].as_str().expect("an element's kind") {
                "null" => JsonElement::Null,
                "boolean" => JsonElement::Primitive(JsonPrimitive::Boolean(bool::decode(&spec[1]))),
                "number" => {
                    let text = text(2);
                    let number = match spec[1].as_str().expect("a box") {
                        "Byte" => Number::Byte(Byte::parse_byte(&text).unwrap()),
                        "Short" => Number::Short(Short::parse_short(&text).unwrap()),
                        "Integer" => Number::Integer(Integer::parse_int(&text).unwrap()),
                        "Long" => Number::Long(Long::parse_long(&text).unwrap()),
                        "Float" => Number::Float(Float::parse_float(&text).unwrap()),
                        "Double" => Number::Double(Double::parse_double(&text).unwrap()),
                        "LazilyParsedNumber" => {
                            Number::LazilyParsedNumber(LazilyParsedNumber::new(text))
                        }
                        "BigDecimal" => Number::BigDecimal(BigDecimal::new(&text).unwrap()),
                        other => panic!("no such box: {other}"),
                    };
                    JsonElement::Primitive(JsonPrimitive::Number(number))
                }
                "string" => JsonElement::Primitive(JsonPrimitive::String(text(1))),
                "array" => JsonElement::Array(Vec::decode(&spec[1])),
                "object" => {
                    let mut object = JsonObject::new();
                    for member in spec[1].as_array().expect("members") {
                        let (name, value) = <(String, JsonElement)>::decode(member);
                        object.add(name, value);
                    }
                    JsonElement::Object(object)
                }
                other => panic!("no such element: {other}"),
            }
        }
    }

    impl Encode for JsonElement {
        fn encode(&self) -> Value {
            match self {
                JsonElement::Null => Value::from(vec!["null"]),
                JsonElement::Primitive(JsonPrimitive::Boolean(value)) => {
                    Value::from(vec![Value::from("boolean"), Value::from(*value)])
                }
                JsonElement::Primitive(JsonPrimitive::Number(number)) => {
                    Value::from(vec!["number", simple_name(number), &number.to_string()])
                }
                JsonElement::Primitive(JsonPrimitive::String(string)) => {
                    Value::from(vec!["string", string])
                }
                JsonElement::Array(elements) => {
                    Value::from(vec![Value::from("array"), elements.encode()])
                }
                JsonElement::Object(object) => {
                    let members: Vec<Value> = object
                        .entry_set()
                        .map(|(name, value)| {
                            Value::from(vec![Value::from(name.as_str()), value.encode()])
                        })
                        .collect();
                    Value::from(vec![Value::from("object"), Value::from(members)])
                }
            }
        }
    }

    /// `toString()` of every element GsonSuites writes, and `equals` of pairs of them.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.JsonElement");
        golden.check("toString()Ljava/lang/String;", |element: JsonElement| {
            element.to_string()
        });
        golden.check(
            "toString()Ljava/lang/String; message",
            |element: JsonElement| {
                thrown(|| {
                    element.to_string();
                })
            },
        );
        golden.check(
            "equals(Ljava/lang/Object;)Z",
            |first: JsonElement, second: JsonElement| first == second,
        );
        golden.check(
            "equals(Ljava/lang/Object;)Z message",
            |first: JsonElement, second: JsonElement| {
                thrown(|| {
                    let _ = first == second;
                })
            },
        );
        golden.finish();
    }
}
