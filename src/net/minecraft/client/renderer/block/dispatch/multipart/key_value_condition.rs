//! net/minecraft/client/renderer/block/dispatch/multipart/KeyValueCondition.java
//!
//! `CODEC` (a non-empty map of property names to terms, each a string, or an int or boolean
//! written as one) is decoded by hand (`KeyValueCondition::parse`). Java warns about a condition
//! that's always true or always false; the port doesn't.

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::Predicate;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::property::{
    Comparable, Property,
};
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

/// Properties' values the state must have, every one of them.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyValueCondition {
    pub tests: Vec<(String, Terms)>,
}

/// `KeyValueCondition.Term`: a value, or (`!`) any other.
#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub value: String,
    pub negated: bool,
}

/// `KeyValueCondition.Terms`: `|`-separated terms, any of which may match.
#[derive(Clone, Debug, PartialEq)]
pub struct Terms {
    pub entries: Vec<Term>,
}

impl KeyValueCondition {
    pub fn instantiate(&self, definition: &StateDefinition) -> Result<Predicate, String> {
        let mut predicates = Vec::with_capacity(self.tests.len());
        for (key, value_test) in &self.tests {
            let property = definition.get_property(key).ok_or_else(|| {
                format!("Unknown property '{key}' on '{}'", definition.get_owner())
            })?;
            predicates.push(value_test.instantiate(&definition.get_owner().to_string(), property)?);
        }
        Ok(Box::new(move |state: BlockState| {
            predicates.iter().all(|predicate| predicate(state))
        }))
    }

    pub fn parse(object: &JsonObject) -> Result<KeyValueCondition, String> {
        if object.is_empty() {
            return Err("Map must not be empty".to_owned());
        }
        let tests = object
            .entry_set()
            .map(|(key, value)| Ok((key.clone(), Terms::parse_element(value)?)))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(KeyValueCondition { tests })
    }
}

impl Term {
    pub fn parse(value: &str) -> Term {
        match value.strip_prefix('!') {
            Some(value) => Term {
                value: value.to_owned(),
                negated: true,
            },
            None => Term {
                value: value.to_owned(),
                negated: false,
            },
        }
    }
}

impl Terms {
    /// `CODEC`: a string, or the legacy int or boolean, then `parse`.
    fn parse_element(value: &JsonElement) -> Result<Terms, String> {
        let text = match value.as_json_primitive() {
            Some(primitive) if primitive.is_string() => primitive.get_as_string(),
            Some(primitive) if primitive.is_number() => primitive.get_as_int().to_string(),
            Some(primitive) if primitive.is_boolean() => primitive.get_as_boolean().to_string(),
            _ => return Err(format!("Not a string: {}", value.to_json_string())),
        };
        Terms::parse(&text)
    }

    pub fn parse(value: &str) -> Result<Terms, String> {
        let terms: Vec<Term> = value.split('|').map(Term::parse).collect();
        if terms.iter().any(|entry| entry.value.is_empty()) {
            return Err(format!("Empty term in value '{value}'"));
        }
        Ok(Terms { entries: terms })
    }

    /// The property's values any term allows. Java matches the fewer of the allowed and the
    /// refused values, which answers the same. The `Err` is Java's `RuntimeException` for a
    /// value the property can't take.
    pub fn instantiate(
        &self,
        owner: &str,
        property: &'static dyn Property,
    ) -> Result<Predicate, String> {
        let mut parsed = Vec::with_capacity(self.entries.len());
        for term in &self.entries {
            let value = property.get_value(&term.value).ok_or_else(|| {
                format!(
                    "Unknown value '{}' for property '{property}' on '{owner}' in '{}'",
                    term.value,
                    self.to_java_string()
                )
            })?;
            parsed.push((value, term.negated));
        }
        let allowed: Vec<Comparable> = property
            .get_possible_values()
            .into_iter()
            .filter(|value| {
                parsed
                    .iter()
                    .any(|(parsed_value, negated)| (value == parsed_value) != *negated)
            })
            .collect();
        Ok(Box::new(move |state: BlockState| {
            allowed.contains(&state.get_comparable(property))
        }))
    }

    fn to_java_string(&self) -> String {
        self.entries
            .iter()
            .map(|term| {
                if term.negated {
                    format!("!{}", term.value)
                } else {
                    term.value.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}
