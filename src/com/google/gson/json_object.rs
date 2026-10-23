//! com/google/gson/JsonObject.java (Gson 2.14.0)
//!
//! A JSON object: its members in Gson's `LinkedTreeMap`, which iterates in the order names were
//! first added, a name added again keeping its place and taking the new value, and a removed name
//! leaving the others in order. Here that's a `Vec` of the members, searched by name, which is the
//! map's lookup: it orders names by `compareTo`, which only finds equal strings equal. Java's null
//! values are `JsonElement::Null`, which `add` makes of Java's `null`. The map's `equals` is
//! `AbstractMap`'s: the same names with equal values, in any order. `hashCode` isn't ported:
//! `JsonNull`'s is an identity hash.

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

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_primitive::JsonPrimitive;
use crate::java::lang::number::Number;

#[derive(Clone, Debug, Default)]
pub struct JsonObject {
    members: Vec<(String, JsonElement)>,
}

impl JsonObject {
    pub fn new() -> JsonObject {
        JsonObject::default()
    }

    /// `deepCopy()`: a copy of the members, each copied deeply.
    pub fn deep_copy(&self) -> JsonObject {
        self.clone()
    }

    /// `add(property, value)`: a member already there keeps its place and takes the new value.
    pub fn add(&mut self, property: String, value: JsonElement) {
        match self.members.iter_mut().find(|(name, _)| *name == property) {
            Some(member) => member.1 = value,
            None => self.members.push((property, value)),
        }
    }

    /// `remove(property)`: the member's value, if it was there. The others keep their order.
    pub fn remove(&mut self, property: &str) -> Option<JsonElement> {
        let index = self.members.iter().position(|(name, _)| name == property)?;
        Some(self.members.remove(index).1)
    }

    /// `addProperty(String, String)`.
    pub fn add_property_string(&mut self, property: String, value: String) {
        self.add(
            property,
            JsonElement::Primitive(JsonPrimitive::String(value)),
        );
    }

    /// `addProperty(String, Number)`.
    pub fn add_property_number(&mut self, property: String, value: Number) {
        self.add(
            property,
            JsonElement::Primitive(JsonPrimitive::Number(value)),
        );
    }

    /// `addProperty(String, Boolean)`.
    pub fn add_property_boolean(&mut self, property: String, value: bool) {
        self.add(
            property,
            JsonElement::Primitive(JsonPrimitive::Boolean(value)),
        );
    }

    /// `entrySet()`: the members in order.
    pub fn entry_set(&self) -> impl Iterator<Item = (&String, &JsonElement)> {
        self.members.iter().map(|(name, value)| (name, value))
    }

    /// `keySet()`: the names in order.
    pub fn key_set(&self) -> impl Iterator<Item = &String> {
        self.members.iter().map(|(name, _)| name)
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
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

    /// The member's value to change in place, which Java's shared references allow.
    pub fn get_mut(&mut self, member_name: &str) -> Option<&mut JsonElement> {
        self.members
            .iter_mut()
            .find(|(name, _)| name == member_name)
            .map(|(_, value)| value)
    }
}

/// `equals`: the members' map's, the same names with equal values in any order.
impl PartialEq for JsonObject {
    fn eq(&self, other: &JsonObject) -> bool {
        self.size() == other.size()
            && self
                .entry_set()
                .all(|(name, value)| other.get(name).is_some_and(|other| value == other))
    }
}
