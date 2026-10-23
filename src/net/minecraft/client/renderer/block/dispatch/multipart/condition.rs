//! net/minecraft/client/renderer/block/dispatch/multipart/Condition.java
//!
//! An interface its two records implement, so an enum of them. `CODEC` (DataFixerUpper's
//! recursive `either` of a combiner and a key-value condition) is decoded by hand from Gson's tree
//! (`Condition::parse`), the way the codec reads it.

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::client::renderer::block::dispatch::multipart::combined_condition::{
    CombinedCondition, Operation,
};
use crate::net::minecraft::client::renderer::block::dispatch::multipart::key_value_condition::KeyValueCondition;
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::Predicate;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Combined(CombinedCondition),
    KeyValue(KeyValueCondition),
}

impl Condition {
    /// The `Err` is Java's `IllegalArgumentException` or `RuntimeException` for a property the
    /// block hasn't or a value it can't take.
    pub fn instantiate(&self, definition: &StateDefinition) -> Result<Predicate, String> {
        match self {
            Condition::Combined(condition) => condition.instantiate(definition),
            Condition::KeyValue(condition) => condition.instantiate(definition),
        }
    }

    /// A map of exactly one entry, `AND` or `OR`, to a list of conditions is a combiner; any
    /// other map is a key-value condition.
    pub fn parse(value: &JsonElement) -> Result<Condition, String> {
        let object = value
            .as_json_object()
            .ok_or_else(|| format!("Not a JSON object: {}", value.to_json_string()))?;
        let combiner = (|| {
            let mut entries = object.entry_set();
            let (key, terms) = entries.next()?;
            if entries.next().is_some() {
                return None;
            }
            let operation = Operation::by_name(key)?;
            let terms = terms
                .as_json_array()?
                .iter()
                .map(Condition::parse)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            Some(CombinedCondition { operation, terms })
        })();
        match combiner {
            Some(combined) => Ok(Condition::Combined(combined)),
            None => Ok(Condition::KeyValue(KeyValueCondition::parse(object)?)),
        }
    }
}
