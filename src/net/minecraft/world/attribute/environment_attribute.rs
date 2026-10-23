//! net/minecraft/world/attribute/EnvironmentAttribute.java
//!
//! An attribute of the environment, like the sky's color or the fog's distance: its type, its
//! default value and range, and how layers may vary it. Java's objects, which the game compares
//! by reference, are their place in `BuiltInRegistries.ENVIRONMENT_ATTRIBUTE` among the ported
//! ones (`EnvironmentAttributes`), whose table holds what Java's object does. An
//! `EnvironmentAttribute<V>` is that place with the Rust type of its values, which its getters
//! hand out; the system holds attributes by place. `valueCodec` is the type's decoder and the
//! range's `validate`.

use std::fmt;
use std::marker::PhantomData;

use crate::net::minecraft::world::attribute::attribute_range::AttributeRange;
use crate::net::minecraft::world::attribute::attribute_type::{AttributeType, AttributeValue};
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;

/// An attribute whose values are `V`s.
pub struct EnvironmentAttribute<V> {
    id: usize,
    value: PhantomData<fn() -> V>,
}

impl<V> Clone for EnvironmentAttribute<V> {
    fn clone(&self) -> EnvironmentAttribute<V> {
        *self
    }
}

impl<V> Copy for EnvironmentAttribute<V> {}

impl<V> PartialEq for EnvironmentAttribute<V> {
    fn eq(&self, other: &EnvironmentAttribute<V>) -> bool {
        self.id == other.id
    }
}

impl<V> EnvironmentAttribute<V> {
    pub(crate) const fn new(id: usize) -> EnvironmentAttribute<V> {
        EnvironmentAttribute {
            id,
            value: PhantomData,
        }
    }

    /// Its place among the ported attributes, by which systems and maps hold it.
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn info(&self) -> &'static AttributeInfo {
        EnvironmentAttributes::info(self.id)
    }
}

impl<V> fmt::Debug for EnvironmentAttribute<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

/// What Java's attribute object holds (its builder's fields), and its registered name.
#[derive(Debug)]
pub struct AttributeInfo {
    pub name: &'static str,
    pub r#type: AttributeType,
    pub default_value: AttributeValue,
    pub value_range: AttributeRange,
    pub is_syncable: bool,
    pub is_positional: bool,
    pub is_spatially_interpolated: bool,
    pub full_resolution_biomes: bool,
}

impl AttributeInfo {
    /// `builder(type).defaultValue(defaultValue)`: any value, not synced, positional, not
    /// spatially interpolated.
    pub(crate) fn builder(
        name: &'static str,
        r#type: AttributeType,
        default_value: AttributeValue,
    ) -> AttributeInfo {
        AttributeInfo {
            name,
            r#type,
            default_value,
            value_range: AttributeRange::Any,
            is_syncable: false,
            is_positional: true,
            is_spatially_interpolated: false,
            full_resolution_biomes: false,
        }
    }

    pub(crate) fn value_range(mut self, value_range: AttributeRange) -> AttributeInfo {
        self.value_range = value_range;
        self
    }

    pub(crate) fn syncable(mut self) -> AttributeInfo {
        self.is_syncable = true;
        self
    }

    pub(crate) fn not_positional(mut self) -> AttributeInfo {
        self.is_positional = false;
        self
    }

    pub(crate) fn spatially_interpolated(mut self) -> AttributeInfo {
        self.is_spatially_interpolated = true;
        self
    }

    pub fn sanitize_value(&self, value: AttributeValue) -> AttributeValue {
        self.value_range.sanitize(value)
    }

    /// `valueCodec()`: the type's values, in the range.
    pub fn decode_value(
        &self,
        json: &crate::com::google::gson::json_element::JsonElement,
    ) -> Result<AttributeValue, String> {
        let value = self.r#type.decode_value(json)?;
        self.value_range.validate(&value)?;
        Ok(value)
    }
}
