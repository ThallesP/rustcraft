//! net/minecraft/world/entity/ai/attributes/Attribute.java
//!
//! Java's attributes are registered objects handed out as `Holder<Attribute>`s, which the game
//! compares by reference, so here an attribute is a `&'static Attribute`, compared by address
//! (`Attribute::same`). Its only subclass, `RangedAttribute`, is its range. Not ported yet:
//! `CODEC`, `STREAM_CODEC`, the sentiment and style, which come with the attribute packets and
//! tooltips.

use crate::net::minecraft::world::entity::ai::attributes::ranged_attribute::RangedAttribute;

/// A value an entity has, its default and what keeps it in range.
#[derive(Debug)]
pub struct Attribute {
    default_value: f64,
    syncable: bool,
    description_id: &'static str,
    range: Option<RangedAttribute>,
}

impl Attribute {
    /// `new RangedAttribute(descriptionId, defaultValue, minValue, maxValue).setSyncable(syncable)`.
    pub const fn ranged(
        description_id: &'static str,
        default_value: f64,
        min_value: f64,
        max_value: f64,
        syncable: bool,
    ) -> Attribute {
        Attribute {
            default_value,
            syncable,
            description_id,
            range: Some(RangedAttribute::new(default_value, min_value, max_value)),
        }
    }

    pub fn get_default_value(&self) -> f64 {
        self.default_value
    }

    pub fn is_client_syncable(&self) -> bool {
        self.syncable
    }

    pub fn get_description_id(&self) -> &'static str {
        self.description_id
    }

    /// The value kept in range: `RangedAttribute`'s clamp, or the value itself.
    pub fn sanitize_value(&self, value: f64) -> f64 {
        match &self.range {
            Some(range) => range.sanitize_value(value),
            None => value,
        }
    }

    /// Whether two holders are the same attribute, like Java's references.
    pub fn same(first: &'static Attribute, second: &'static Attribute) -> bool {
        std::ptr::eq(first, second)
    }
}
