//! net/minecraft/world/level/block/state/properties/EnumProperty.java
//!
//! `create(String, Class, Predicate)`, which filters the class's constants, is a list of the
//! constants it keeps, in their order, written out where the property is (`BlockStateProperties`).

use std::any::Any;
use std::fmt;

use crate::net::minecraft::world::level::block::state::properties::property::{
    self, Comparable, Property, PropertyEnum, PropertyValue, TypedProperty,
};

/// A property whose values are some of an enum's constants, in the order it lists them.
pub struct EnumProperty<T: 'static> {
    name: &'static str,
    values: &'static [T],
}

impl<T: PropertyEnum> EnumProperty<T> {
    /// `create(name, clazz)`: every constant of the enum.
    pub const fn create_all(name: &'static str) -> EnumProperty<T> {
        EnumProperty::create(name, T::ENUM_CONSTANTS)
    }

    /// `create(name, clazz, values)`. Panics with no values, where Java throws
    /// `IllegalArgumentException`.
    pub const fn create(name: &'static str, values: &'static [T]) -> EnumProperty<T> {
        if values.is_empty() {
            panic!("IllegalArgumentException: trying to make an empty EnumProperty");
        }
        EnumProperty { name, values }
    }

    pub fn get_possible_values_typed(&self) -> &'static [T] {
        self.values
    }

    /// Its place among the values, or -1 for a constant they leave out.
    pub fn get_internal_index_of(&self, value: T) -> i32 {
        self.values
            .iter()
            .position(|candidate| *candidate == value)
            .map_or(-1, |index| index as i32)
    }
}

impl<T: PropertyEnum> Property for EnumProperty<T> {
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_value_class(&self) -> &'static str {
        T::CLASS
    }

    fn get_possible_values(&self) -> Vec<Comparable> {
        self.values
            .iter()
            .map(|value| value.to_comparable())
            .collect()
    }

    fn possible_value_count(&self) -> usize {
        self.values.len()
    }

    fn possible_value(&self, index: usize) -> Comparable {
        self.values[index].to_comparable()
    }

    fn get_name_of(&self, value: Comparable) -> String {
        T::from_comparable(value).get_serialized_name().to_owned()
    }

    /// The value whose serialized name it is.
    fn get_value(&self, name: &str) -> Option<Comparable> {
        self.values
            .iter()
            .find(|value| value.get_serialized_name() == name)
            .map(|value| value.to_comparable())
    }

    fn get_internal_index(&self, value: Comparable) -> i32 {
        self.get_internal_index_of(T::from_comparable(value))
    }

    fn equals(&self, other: &dyn Property) -> bool {
        property::same(self, other)
            || other
                .as_any()
                .downcast_ref::<EnumProperty<T>>()
                .is_some_and(|that| {
                    that.name == self.name
                        && that.values.len() == self.values.len()
                        && that.values.iter().zip(self.values).all(|(a, b)| a == b)
                })
    }

    fn value_of_constant(&self, name: &str) -> Option<Comparable> {
        T::ENUM_CONSTANTS
            .iter()
            .find(|constant| format!("{constant:?}") == name)
            .map(|constant| constant.to_comparable())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T: PropertyEnum> TypedProperty for EnumProperty<T> {
    type Value = T;
}

impl<T: PropertyEnum> fmt::Display for EnumProperty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        property::to_string_helper(f, "EnumProperty", self)
    }
}
