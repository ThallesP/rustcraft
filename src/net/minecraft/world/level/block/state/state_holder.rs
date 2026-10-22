//! net/minecraft/world/level/block/state/StateHolder.java
//!
//! Not ported yet: `codec`, which waits for DataFixerUpper's codecs. `getValues` gives the
//! property and value pairs, since `Property.Value` only exists for the codecs so far.
//!
//! Java's abstract `StateHolder<O, S>` is a trait of what every state does, which `BlockState`
//! implements. A state is its definition and its index there (`StateDefinition`), so it finds its
//! values by decoding the index, and its neighbours by changing a digit of it, which gives the same
//! states as Java's neighbour table.

use std::fmt;

use crate::net::minecraft::world::level::block::state::properties::property::{
    Comparable, Property, PropertyValue, TypedProperty,
};
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

pub trait StateHolder: Copy + Sized {
    /// The definition of the states this is one of.
    fn definition(&self) -> &'static StateDefinition;

    /// Its index among the definition's states.
    fn index(&self) -> u32;

    /// The definition's state at `index`.
    fn with_index(&self, index: u32) -> Self;

    /// The next of the property's values, or the first after the last.
    fn cycle(&self, property: &'static dyn Property) -> Self {
        let next = find_next_in_collection(property, self.get_comparable(property));
        self.set_comparable(property, next)
    }

    fn get_properties(&self) -> &[&'static dyn Property] {
        self.definition().get_properties()
    }

    fn has_property(&self, property: &dyn Property) -> bool {
        self.definition().property_index(property).is_some()
    }

    /// `getValue`, with the value's type erased. Panics on a property the state doesn't have,
    /// where Java throws `IllegalArgumentException`.
    fn get_comparable(&self, property: &dyn Property) -> Comparable {
        self.get_optional_comparable(property).unwrap_or_else(|| {
            panic!(
                "IllegalArgumentException: cannot get property {property} as it does not exist in {}",
                self.definition().get_owner()
            )
        })
    }

    fn get_optional_comparable(&self, property: &dyn Property) -> Option<Comparable> {
        let definition = self.definition();
        let property_index = definition.property_index(property)?;
        Some(property.possible_value(definition.value_index(self.index(), property_index)))
    }

    /// The property's value. Panics on a property the state doesn't have.
    fn get_value<P: TypedProperty>(&self, property: &'static P) -> P::Value {
        P::Value::from_comparable(self.get_comparable(property))
    }

    fn get_optional_value<P: TypedProperty>(&self, property: &'static P) -> Option<P::Value> {
        self.get_optional_comparable(property)
            .map(P::Value::from_comparable)
    }

    fn get_value_or_else<P: TypedProperty>(
        &self,
        property: &'static P,
        default_value: P::Value,
    ) -> P::Value {
        self.get_optional_value(property).unwrap_or(default_value)
    }

    /// `setValue`, with the value's type erased. Panics on a property the state doesn't have, or
    /// a value it doesn't take, where Java throws `IllegalArgumentException`, or one whose index
    /// wraps past the values, where Java's neighbour table throws
    /// `ArrayIndexOutOfBoundsException`.
    fn set_comparable(&self, property: &dyn Property, value: Comparable) -> Self {
        let Some(property_index) = self.definition().property_index(property) else {
            panic!(
                "IllegalArgumentException: cannot set property {property} as it does not exist in {}",
                self.definition().get_owner()
            );
        };
        self.set_value_internal(property, property_index, value)
    }

    /// The state with the property's value changed. Panics like `set_comparable`.
    fn set_value<P: TypedProperty>(&self, property: &'static P, value: P::Value) -> Self {
        self.set_comparable(property, value.to_comparable())
    }

    /// `setValue`, or this state if it doesn't have the property.
    fn try_set_comparable(&self, property: &dyn Property, value: Comparable) -> Self {
        match self.definition().property_index(property) {
            Some(property_index) => self.set_value_internal(property, property_index, value),
            None => *self,
        }
    }

    fn try_set_value<P: TypedProperty>(&self, property: &'static P, value: P::Value) -> Self {
        self.try_set_comparable(property, value.to_comparable())
    }

    fn set_value_internal(
        &self,
        property: &dyn Property,
        property_index: usize,
        value: Comparable,
    ) -> Self {
        let value_index = property.get_internal_index(value);
        if value_index < 0 {
            panic!(
                "IllegalArgumentException: cannot set property {property} to {value} on {}, it is not an allowed value",
                self.definition().get_owner()
            );
        }
        let value_index = value_index as usize;
        assert!(
            value_index < property.possible_value_count(),
            "ArrayIndexOutOfBoundsException: Index {value_index} out of bounds for length {}",
            property.possible_value_count()
        );
        self.with_index(self.definition().with_value_index(
            self.index(),
            property_index,
            value_index,
        ))
    }

    fn is_singleton_state(&self) -> bool {
        self.definition().is_singleton_state()
    }

    /// `getValues()`: each property, sorted by name, with the state's value.
    fn get_values(&self) -> Vec<(&'static dyn Property, Comparable)> {
        self.get_properties()
            .iter()
            .map(|&property| (property, self.get_comparable(property)))
            .collect()
    }

    /// `toString()`: the owner, then the values when there are any,
    /// `Block{minecraft:oak_stairs}[facing=north,half=bottom,…]`.
    fn fmt_state(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.definition().get_owner())?;
        if !self.is_singleton_state() {
            let values: Vec<String> = self
                .get_values()
                .iter()
                .map(|(property, value)| {
                    format!("{}={}", property.get_name(), property.get_name_of(*value))
                })
                .collect();
            write!(f, "[{}]", values.join(","))?;
        }
        Ok(())
    }
}

/// `findNextInCollection`: the value after `current` among the property's, or the first.
fn find_next_in_collection(property: &dyn Property, current: Comparable) -> Comparable {
    let values = property.get_possible_values();
    let next = values
        .iter()
        .position(|value| *value == current)
        .map_or(0, |index| index + 1);
    if next == values.len() {
        values[0]
    } else {
        values[next]
    }
}
