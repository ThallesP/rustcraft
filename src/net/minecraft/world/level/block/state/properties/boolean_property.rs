//! net/minecraft/world/level/block/state/properties/BooleanProperty.java

use std::any::Any;
use std::fmt;

use crate::net::minecraft::world::level::block::state::properties::property::{
    self, Comparable, Property, PropertyValue, TypedProperty,
};

/// A property that's `true` or `false`, in that order.
pub struct BooleanProperty {
    name: &'static str,
}

const VALUES: [bool; 2] = [true, false];

impl BooleanProperty {
    pub const fn create(name: &'static str) -> BooleanProperty {
        BooleanProperty { name }
    }

    pub fn get_possible_values_typed(&self) -> Vec<bool> {
        VALUES.to_vec()
    }

    pub fn get_internal_index_of(&self, value: bool) -> i32 {
        if value { 0 } else { 1 }
    }
}

impl Property for BooleanProperty {
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_value_class(&self) -> &'static str {
        "java.lang.Boolean"
    }

    fn get_possible_values(&self) -> Vec<Comparable> {
        VALUES.map(Comparable::Boolean).to_vec()
    }

    fn possible_value_count(&self) -> usize {
        VALUES.len()
    }

    fn possible_value(&self, index: usize) -> Comparable {
        Comparable::Boolean(VALUES[index])
    }

    fn get_name_of(&self, value: Comparable) -> String {
        bool::from_comparable(value).to_string()
    }

    fn get_value(&self, name: &str) -> Option<Comparable> {
        match name {
            "true" => Some(Comparable::Boolean(true)),
            "false" => Some(Comparable::Boolean(false)),
            _ => None,
        }
    }

    fn get_internal_index(&self, value: Comparable) -> i32 {
        self.get_internal_index_of(bool::from_comparable(value))
    }

    /// Property's: another boolean property of the same name.
    fn equals(&self, other: &dyn Property) -> bool {
        property::same(self, other)
            || (other.get_value_class() == self.get_value_class() && other.get_name() == self.name)
    }

    fn value_of_constant(&self, _name: &str) -> Option<Comparable> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TypedProperty for BooleanProperty {
    type Value = bool;
}

impl fmt::Display for BooleanProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        property::to_string_helper(f, "BooleanProperty", self)
    }
}
