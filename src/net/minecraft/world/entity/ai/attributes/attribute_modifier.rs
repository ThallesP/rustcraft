//! net/minecraft/world/entity/ai/attributes/AttributeModifier.java
//!
//! Not ported yet: `CODEC`, `MAP_CODEC` and `STREAM_CODEC`, which come with
//! `ClientboundUpdateAttributesPacket`.

use crate::net::minecraft::resources::identifier::Identifier;

/// A change to an attribute's value, under an id.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifier {
    id: Identifier,
    amount: f64,
    operation: Operation,
}

/// `AttributeModifier.Operation`: added to the base, a multiple of the base added, or the total
/// multiplied, in that order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    ADD_VALUE,
    ADD_MULTIPLIED_BASE,
    ADD_MULTIPLIED_TOTAL,
}

impl Operation {
    pub const VALUES: [Operation; 3] = [
        Operation::ADD_VALUE,
        Operation::ADD_MULTIPLIED_BASE,
        Operation::ADD_MULTIPLIED_TOTAL,
    ];

    pub fn id(&self) -> i32 {
        *self as i32
    }
}

impl AttributeModifier {
    pub fn new(id: Identifier, amount: f64, operation: Operation) -> AttributeModifier {
        AttributeModifier {
            id,
            amount,
            operation,
        }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn operation(&self) -> Operation {
        self.operation
    }

    pub fn is(&self, id: &Identifier) -> bool {
        *id == self.id
    }
}
