//! net/minecraft/client/renderer/block/dispatch/VariantSelector.java

use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::property::{
    Comparable, Property,
};
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct VariantSelector;

/// The values a variant's key asks for, by property.
pub type Predicate = Box<dyn Fn(BlockState) -> bool + Send + Sync>;

impl VariantSelector {
    /// The states whose values are all `properties`' (`facing=north,half=top`, or empty for
    /// every state). Java throws `RuntimeException` for a property the block hasn't, or a value
    /// it can't take, which is the `Err`.
    pub fn predicate(
        state_definition: &StateDefinition,
        properties: &str,
    ) -> Result<Predicate, String> {
        let mut map: Vec<(&'static dyn Property, Comparable)> = Vec::new();
        for key_value in properties.split(',') {
            let mut iterator = key_value.splitn(2, '=');
            let Some(property_name) = iterator.next() else {
                continue;
            };
            let property = state_definition.get_property(property_name);
            match (property, iterator.next()) {
                (Some(property), Some(property_value)) => {
                    let value = property.get_value(property_value).ok_or_else(|| {
                        format!(
                            "Unknown value: '{property_value}' for blockstate property: '{property_name}' {:?}",
                            property.get_possible_values()
                        )
                    })?;
                    // A `HashMap`'s put: the last value for a property wins.
                    match map.iter_mut().find(|(known, _)| crate::net::minecraft::world::level::block::state::properties::property::same(*known, property)) {
                        Some(entry) => entry.1 = value,
                        None => map.push((property, value)),
                    }
                }
                _ if !property_name.is_empty() => {
                    return Err(format!("Unknown blockstate property: '{property_name}'"));
                }
                _ => {}
            }
        }

        Ok(Box::new(move |input: BlockState| {
            map.iter()
                .all(|(property, value)| input.get_comparable(*property) == *value)
        }))
    }
}
