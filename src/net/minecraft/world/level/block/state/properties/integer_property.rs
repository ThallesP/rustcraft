//! net/minecraft/world/level/block/state/properties/IntegerProperty.java

use std::any::Any;
use std::fmt;

use crate::java::lang::integer::Integer;
use crate::net::minecraft::world::level::block::state::properties::property::{
    self, Comparable, Property, PropertyValue, TypedProperty,
};

/// A property that's an int from `min` to `max`, both included.
pub struct IntegerProperty {
    name: &'static str,
    min: i32,
    max: i32,
}

impl IntegerProperty {
    /// Panics on a negative minimum or a maximum not above it, where Java throws
    /// `IllegalArgumentException`.
    pub const fn create(name: &'static str, min: i32, max: i32) -> IntegerProperty {
        if min < 0 {
            panic!("IllegalArgumentException: an IntegerProperty's min value must be 0 or greater");
        }
        if max <= min {
            panic!(
                "IllegalArgumentException: an IntegerProperty's max value must be greater than min"
            );
        }
        IntegerProperty { name, min, max }
    }

    /// `IntStream.range(min, max + 1)`, which is empty when `max + 1` wraps.
    pub fn get_possible_values_typed(&self) -> Vec<i32> {
        (self.min..self.max.wrapping_add(1)).collect()
    }

    /// Its place among the values, counted from the minimum even below it, or -1 above the
    /// maximum.
    pub fn get_internal_index_of(&self, value: i32) -> i32 {
        if value <= self.max {
            value.wrapping_sub(self.min)
        } else {
            -1
        }
    }
}

impl Property for IntegerProperty {
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_value_class(&self) -> &'static str {
        "java.lang.Integer"
    }

    fn get_possible_values(&self) -> Vec<Comparable> {
        self.get_possible_values_typed()
            .into_iter()
            .map(Comparable::Integer)
            .collect()
    }

    fn possible_value_count(&self) -> usize {
        (self.min..self.max.wrapping_add(1)).len()
    }

    fn possible_value(&self, index: usize) -> Comparable {
        assert!(
            index < self.possible_value_count(),
            "IndexOutOfBoundsException: {index}"
        );
        Comparable::Integer(self.min + index as i32)
    }

    fn get_name_of(&self, value: Comparable) -> String {
        i32::from_comparable(value).to_string()
    }

    /// `Integer.parseInt`'s value, when it's in range.
    fn get_value(&self, name: &str) -> Option<Comparable> {
        Integer::parse_int(name)
            .ok()
            .filter(|value| (self.min..=self.max).contains(value))
            .map(Comparable::Integer)
    }

    fn get_internal_index(&self, value: Comparable) -> i32 {
        self.get_internal_index_of(i32::from_comparable(value))
    }

    fn equals(&self, other: &dyn Property) -> bool {
        property::same(self, other)
            || other
                .as_any()
                .downcast_ref::<IntegerProperty>()
                .is_some_and(|that| {
                    that.name == self.name && that.min == self.min && that.max == self.max
                })
    }

    fn value_of_constant(&self, _name: &str) -> Option<Comparable> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TypedProperty for IntegerProperty {
    type Value = i32;
}

impl fmt::Display for IntegerProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        property::to_string_helper(f, "IntegerProperty", self)
    }
}
