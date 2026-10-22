//! net/minecraft/network/protocol/login/custom/CustomQueryPayload.java
//!
//! Java's `equals`, `hashCode` and `toString` come from `Object`, and each payload (a record)
//! overrides them, so they're the trait's `equals`, `hash_code` and `Display`, like
//! `CustomQueryAnswerPayload`'s.

use std::any::Any;
use std::fmt::{Debug, Display};

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::resources::identifier::Identifier;

/// What a server asks a client in a custom query, after the transaction id: its identifier,
/// which says what kind of query it is, then what `write` writes. Vanilla only has
/// `DiscardedQueryPayload`.
pub trait CustomQueryPayload: Any + Debug + Display + Send + Sync {
    fn id(&self) -> &Identifier;

    fn write(&self, output: &mut FriendlyByteBuf);

    /// `equals(Object)`, with another payload.
    fn equals(&self, other: &dyn CustomQueryPayload) -> bool;

    fn hash_code(&self) -> i32;
}

impl PartialEq for dyn CustomQueryPayload {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
pub mod tests {
    use std::any::Any;

    use serde_json::Value;

    use super::CustomQueryPayload;
    use crate::golden::{Decode, Encode};
    use crate::net::minecraft::network::protocol::login::custom::discarded_query_payload::DiscardedQueryPayload;

    /// A payload declared as the interface, which vanilla encodes by its own class: vanilla's
    /// only one, `DiscardedQueryPayload`.
    impl Decode for Box<dyn CustomQueryPayload> {
        fn decode(value: &Value) -> Self {
            Box::new(DiscardedQueryPayload::decode(value))
        }
    }

    impl Encode for dyn CustomQueryPayload {
        fn encode(&self) -> Value {
            let payload: &dyn Any = self;
            payload
                .downcast_ref::<DiscardedQueryPayload>()
                .unwrap_or_else(|| panic!("{self:?} isn't one of vanilla's payloads"))
                .encode()
        }
    }
}
