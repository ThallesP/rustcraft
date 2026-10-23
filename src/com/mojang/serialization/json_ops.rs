//! com/mojang/serialization/JsonOps.java (DataFixerUpper 10.0.21)
//!
//! `DynamicOps` over Gson's JSON tree, which the login's disconnect reason and the server list's
//! status are read with. A map is a `JsonObject` (its members in order), a list a `JsonArray`, and
//! a number keeps its box (`createInt` makes an `Integer`, the parser a `LazilyParsedNumber`).
//! Only `INSTANCE` is ported: `COMPRESSED`, which writes maps as lists, comes with a caller.
//!
//! Java hands a `JsonNull` member or element out as `null` where it streams them (`getMapValues`,
//! `getStream`, `getList`, a `MapLike`'s `get`), which a codec then reads as missing or as nothing.
//! Here they stay `JsonElement::Null`, which prints and reads the same (`"null"`, not a string,
//! not a number), except where Java dereferences the `null`: `convertTo`, which panics on one,
//! where Java throws `NullPointerException`. A `JsonNull` that isn't streamed never reaches
//! `convertTo` (a map's `get` treats one as missing), so every one that does was a `null`.

use std::fmt;

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::com::google::gson::json_primitive::JsonPrimitive;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, Codec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsKind, OpsValue};
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::list_builder::ListBuilder;
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodec};
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::{
    RecordBuilder, StringBuilder, StringBuilderOps,
};
use crate::java::lang::number::Number;

pub struct JsonOps;

impl JsonOps {
    pub const INSTANCE: JsonOps = JsonOps;

    // `list != this.empty()` and its kin: Java compares with `JsonNull.INSTANCE`.
    fn is_empty(value: &JsonElement) -> bool {
        matches!(value, JsonElement::Null)
    }

    // A key `mergeToMap` takes: a string primitive.
    fn string_key(key: &JsonElement) -> Option<String> {
        match key {
            JsonElement::Primitive(primitive) if primitive.is_string() => {
                Some(primitive.get_as_string())
            }
            _ => None,
        }
    }

    // A copy of the object's members, or none for the empty value.
    fn copy_or_new(map: &JsonElement) -> JsonObject {
        let mut output = JsonObject::new();
        if let JsonElement::Object(object) = map {
            for (key, value) in object.entry_set() {
                output.add(key.clone(), value.clone());
            }
        }
        output
    }
}

/// `JSON`.
impl fmt::Display for JsonOps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON")
    }
}

impl OpsValue for JsonElement {
    /// `JsonOps.convertTo`: a number as the smallest integer box its exact value fits, else a
    /// float if it's one exactly, else a double.
    fn convert_to<U: OpsValue>(out_ops: &dyn DynamicOps<U>, input: &JsonElement) -> U {
        match input {
            JsonElement::Object(object) => out_ops.create_map(
                object
                    .entry_set()
                    .map(|(key, value)| {
                        Pair::new(
                            out_ops.create_string(key),
                            JsonElement::convert_to(out_ops, value),
                        )
                    })
                    .collect(),
            ),
            JsonElement::Array(elements) => out_ops.create_list(
                elements
                    .iter()
                    .map(|element| JsonElement::convert_to(out_ops, element))
                    .collect(),
            ),
            JsonElement::Null => panic!("NullPointerException: a JsonNull streamed as null"),
            JsonElement::Primitive(primitive) => {
                if primitive.is_string() {
                    return out_ops.create_string(&primitive.get_as_string());
                }
                if primitive.is_boolean() {
                    return out_ops.create_boolean(primitive.get_as_boolean());
                }
                let value = primitive.get_as_big_decimal();
                match value.long_value_exact() {
                    Ok(l) if l as i8 as i64 == l => out_ops.create_byte(l as i8),
                    Ok(l) if l as i16 as i64 == l => out_ops.create_short(l as i16),
                    Ok(l) if l as i32 as i64 == l => out_ops.create_int(l as i32),
                    Ok(l) => out_ops.create_long(l),
                    Err(_) => {
                        let d = value.double_value();
                        if (d as f32) as f64 == d {
                            out_ops.create_float(d as f32)
                        } else {
                            out_ops.create_double(d)
                        }
                    }
                }
            }
        }
    }

    fn boxed_codec<A>(codec: &BoxedCodec<A>) -> &(dyn Codec<JsonElement, Value = A> + Send + Sync) {
        codec.json()
    }

    fn boxed_map_codec<A>(
        codec: &BoxedMapCodec<A>,
    ) -> &(dyn MapCodec<JsonElement, Value = A> + Send + Sync) {
        codec.json()
    }
}

impl DynamicOps<JsonElement> for JsonOps {
    fn kind(&self) -> OpsKind {
        OpsKind::Json
    }

    fn empty(&self) -> JsonElement {
        JsonElement::Null
    }

    fn empty_map(&self) -> JsonElement {
        JsonElement::Object(JsonObject::new())
    }

    fn empty_list(&self) -> JsonElement {
        JsonElement::Array(Vec::new())
    }

    fn get_number_value(&self, input: &JsonElement) -> DataResult<Number> {
        match input {
            JsonElement::Primitive(primitive) if primitive.is_number() => {
                DataResult::success(primitive.get_as_number())
            }
            _ => DataResult::error(|| format!("Not a number: {input}")),
        }
    }

    fn create_numeric(&self, i: Number) -> JsonElement {
        JsonElement::Primitive(JsonPrimitive::Number(i))
    }

    fn get_boolean_value(&self, input: &JsonElement) -> DataResult<bool> {
        match input {
            JsonElement::Primitive(primitive) if primitive.is_boolean() => {
                DataResult::success(primitive.get_as_boolean())
            }
            _ => DataResult::error(|| format!("Not a boolean: {input}")),
        }
    }

    fn create_boolean(&self, value: bool) -> JsonElement {
        JsonElement::Primitive(JsonPrimitive::Boolean(value))
    }

    fn get_string_value(&self, input: &JsonElement) -> DataResult<String> {
        match input {
            JsonElement::Primitive(primitive) if primitive.is_string() => {
                DataResult::success(primitive.get_as_string())
            }
            _ => DataResult::error(|| format!("Not a string: {input}")),
        }
    }

    fn create_string(&self, value: &str) -> JsonElement {
        JsonElement::Primitive(JsonPrimitive::String(value.to_owned()))
    }

    fn merge_to_list(&self, list: &JsonElement, value: JsonElement) -> DataResult<JsonElement> {
        let mut result = match list {
            JsonElement::Array(elements) => elements.clone(),
            JsonElement::Null => Vec::new(),
            _ => {
                return DataResult::error_partial(
                    || format!("mergeToList called with not a list: {list}"),
                    list.clone(),
                );
            }
        };
        result.push(value);
        DataResult::success(JsonElement::Array(result))
    }

    fn merge_to_list_values(
        &self,
        list: &JsonElement,
        values: Vec<JsonElement>,
    ) -> DataResult<JsonElement> {
        let mut result = match list {
            JsonElement::Array(elements) => elements.clone(),
            JsonElement::Null => Vec::new(),
            _ => {
                return DataResult::error_partial(
                    || format!("mergeToList called with not a list: {list}"),
                    list.clone(),
                );
            }
        };
        if values.is_empty() {
            return DataResult::success(if JsonOps::is_empty(list) {
                self.empty_list()
            } else {
                list.clone()
            });
        }
        result.extend(values);
        DataResult::success(JsonElement::Array(result))
    }

    fn merge_to_map(
        &self,
        map: &JsonElement,
        key: JsonElement,
        value: JsonElement,
    ) -> DataResult<JsonElement> {
        if !matches!(map, JsonElement::Object(_) | JsonElement::Null) {
            return DataResult::error_partial(
                || format!("mergeToMap called with not a map: {map}"),
                map.clone(),
            );
        }
        match JsonOps::string_key(&key) {
            Some(key) => {
                let mut output = JsonOps::copy_or_new(map);
                output.add(key, value);
                DataResult::success(JsonElement::Object(output))
            }
            None => {
                DataResult::error_partial(|| format!("key is not a string: {key}"), map.clone())
            }
        }
    }

    fn merge_to_map_like(
        &self,
        map: &JsonElement,
        values: &dyn MapLike<JsonElement>,
    ) -> DataResult<JsonElement> {
        if !matches!(map, JsonElement::Object(_) | JsonElement::Null) {
            return DataResult::error_partial(
                || format!("mergeToMap called with not a map: {map}"),
                map.clone(),
            );
        }
        let entries = values.entries();
        if entries.is_empty() {
            return DataResult::success(if JsonOps::is_empty(map) {
                self.empty_map()
            } else {
                map.clone()
            });
        }
        let mut output = JsonOps::copy_or_new(map);
        let mut missed = Vec::new();
        for entry in entries {
            let (key, value) = entry.into_parts();
            match JsonOps::string_key(&key) {
                Some(key) => output.add(key, value),
                None => missed.push(key.to_string()),
            }
        }
        if !missed.is_empty() {
            let missed = missed.join(", ");
            DataResult::error_partial(
                || format!("some keys are not strings: [{missed}]"),
                JsonElement::Object(output),
            )
        } else {
            DataResult::success(JsonElement::Object(output))
        }
    }

    fn get_map_values(
        &self,
        input: &JsonElement,
    ) -> DataResult<Vec<Pair<JsonElement, JsonElement>>> {
        match input {
            JsonElement::Object(object) => DataResult::success(
                object
                    .entry_set()
                    .map(|(key, value)| Pair::new(self.create_string(key), value.clone()))
                    .collect(),
            ),
            _ => DataResult::error(|| format!("Not a JSON object: {input}")),
        }
    }

    fn create_map(&self, map: Vec<Pair<JsonElement, JsonElement>>) -> JsonElement {
        let mut result = JsonObject::new();
        for entry in map {
            let (key, value) = entry.into_parts();
            result.add(key.get_as_string(), value);
        }
        JsonElement::Object(result)
    }

    fn get_map<'a>(
        &self,
        input: &'a JsonElement,
    ) -> DataResult<Box<dyn MapLike<JsonElement> + 'a>> {
        match input {
            JsonElement::Object(object) => DataResult::success(Box::new(ObjectMapLike { object })),
            _ => DataResult::error(|| format!("Not a JSON object: {input}")),
        }
    }

    fn get_stream(&self, input: &JsonElement) -> DataResult<Vec<JsonElement>> {
        match input {
            JsonElement::Array(elements) => DataResult::success(elements.clone()),
            _ => DataResult::error(|| format!("Not a json array: {input}")),
        }
    }

    fn create_list(&self, input: Vec<JsonElement>) -> JsonElement {
        JsonElement::Array(input)
    }

    fn remove(&self, input: &JsonElement, key: &str) -> JsonElement {
        match input {
            JsonElement::Object(object) => {
                let mut result = JsonObject::new();
                for (name, value) in object.entry_set() {
                    if name != key {
                        result.add(name.clone(), value.clone());
                    }
                }
                JsonElement::Object(result)
            }
            _ => input.clone(),
        }
    }

    fn list_builder(&self) -> Box<dyn ListBuilder<JsonElement> + '_> {
        Box::new(ArrayBuilder {
            builder: DataResult::success_lifecycle(Vec::new(), Lifecycle::stable()),
        })
    }

    fn map_builder(&self) -> Box<dyn RecordBuilder<JsonElement> + '_> {
        Box::new(StringBuilder::new(self, JsonRecordBuilder))
    }
}

/// `getMap`'s `MapLike`: a `JsonNull` member is missing.
struct ObjectMapLike<'a> {
    object: &'a JsonObject,
}

impl MapLike<JsonElement> for ObjectMapLike<'_> {
    fn get(&self, key: &JsonElement) -> Option<&JsonElement> {
        self.get_str(&key.get_as_string())
    }

    fn get_str(&self, key: &str) -> Option<&JsonElement> {
        self.object
            .get(key)
            .filter(|element| !matches!(element, JsonElement::Null))
    }

    /// Every member, a `JsonNull` one too.
    fn entries(&self) -> Vec<Pair<JsonElement, JsonElement>> {
        self.object
            .entry_set()
            .map(|(key, value)| {
                Pair::new(
                    JsonElement::Primitive(JsonPrimitive::String(key.clone())),
                    value.clone(),
                )
            })
            .collect()
    }
}

/// `MapLike[{…}]`, the object's JSON.
impl fmt::Display for ObjectMapLike<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MapLike[{}]", JsonElement::Object(self.object.clone()))
    }
}

/// `ArrayBuilder`: the elements in a `JsonArray`, appended to a list prefix's.
struct ArrayBuilder {
    builder: DataResult<Vec<JsonElement>>,
}

impl ArrayBuilder {
    fn take(&mut self) -> DataResult<Vec<JsonElement>> {
        std::mem::replace(
            &mut self.builder,
            DataResult::success_lifecycle(Vec::new(), Lifecycle::stable()),
        )
    }
}

impl ListBuilder<JsonElement> for ArrayBuilder {
    fn ops(&self) -> &dyn DynamicOps<JsonElement> {
        &JsonOps::INSTANCE
    }

    fn add(&mut self, value: JsonElement) {
        self.builder = self.take().map(|mut b| {
            b.push(value);
            b
        });
    }

    fn add_result(&mut self, value: DataResult<JsonElement>) {
        self.builder = self.take().apply2stable(
            |mut b, element| {
                b.push(element);
                b
            },
            value,
        );
    }

    fn with_errors_from(&mut self, result: DataResult<()>) {
        self.builder = self.take().flat_map(|r| result.map(|_| r));
    }

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String) {
        self.builder = self.take().map_error(on_error);
    }

    fn build(&mut self, prefix: JsonElement) -> DataResult<JsonElement> {
        self.take().flat_map(|b| {
            let mut array = match &prefix {
                JsonElement::Array(elements) => elements.clone(),
                JsonElement::Null => Vec::new(),
                _ => {
                    let message = format!("Cannot append a list to not a list: {prefix}");
                    return DataResult::error_partial(|| message, prefix);
                }
            };
            array.extend(b);
            DataResult::success_lifecycle(JsonElement::Array(array), Lifecycle::stable())
        })
    }
}

/// `JsonRecordBuilder`: fields into an object, after an object prefix's members.
struct JsonRecordBuilder;

impl StringBuilderOps<JsonElement> for JsonRecordBuilder {
    type Builder = JsonObject;

    fn init_builder(&self) -> JsonObject {
        JsonObject::new()
    }

    fn append(&self, key: String, value: JsonElement, mut builder: JsonObject) -> JsonObject {
        builder.add(key, value);
        builder
    }

    fn build(&self, builder: JsonObject, prefix: JsonElement) -> DataResult<JsonElement> {
        match prefix {
            JsonElement::Null => DataResult::success(JsonElement::Object(builder)),
            JsonElement::Object(object) => {
                let mut result = JsonObject::new();
                for (key, value) in object.entry_set() {
                    result.add(key.clone(), value.clone());
                }
                for (key, value) in builder.entry_set() {
                    result.add(key.clone(), value.clone());
                }
                DataResult::success(JsonElement::Object(result))
            }
            prefix => {
                let message = format!("mergeToMap called with not a map: {prefix}");
                DataResult::error_partial(|| message, prefix)
            }
        }
    }
}
