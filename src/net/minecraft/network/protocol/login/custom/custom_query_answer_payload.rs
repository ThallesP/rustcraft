//! net/minecraft/network/protocol/login/custom/CustomQueryAnswerPayload.java
//!
//! Java's `equals`, `hashCode` and `toString` come from `Object`, and each payload (a record)
//! overrides them, so they're the trait's `equals`, `hash_code` and `Display`: `PartialEq` can't
//! be a supertrait of a trait a `dyn` holds. `dyn CustomQueryAnswerPayload` is `PartialEq`
//! through `equals`.

use std::any::Any;
use std::fmt::{Debug, Display};

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;

/// What a client answers a server's custom query with, after the transaction id. Vanilla only
/// has `DiscardedQueryAnswerPayload`.
pub trait CustomQueryAnswerPayload: Any + Debug + Display + Send + Sync {
    fn write(&self, output: &mut FriendlyByteBuf);

    /// `equals(Object)`, with another payload.
    fn equals(&self, other: &dyn CustomQueryAnswerPayload) -> bool;

    fn hash_code(&self) -> i32;
}

impl PartialEq for dyn CustomQueryAnswerPayload {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
pub mod tests {
    use std::any::Any;

    use serde_json::Value;

    use super::CustomQueryAnswerPayload;
    use crate::golden::{Decode, Encode};
    use crate::net::minecraft::network::protocol::login::custom::discarded_query_answer_payload::DiscardedQueryAnswerPayload;

    /// A payload declared as the interface, which vanilla encodes by its own class: vanilla's
    /// only one, `DiscardedQueryAnswerPayload`.
    impl Decode for Box<dyn CustomQueryAnswerPayload> {
        fn decode(value: &Value) -> Self {
            Box::new(DiscardedQueryAnswerPayload::decode(value))
        }
    }

    impl Encode for dyn CustomQueryAnswerPayload {
        fn encode(&self) -> Value {
            let payload: &dyn Any = self;
            payload
                .downcast_ref::<DiscardedQueryAnswerPayload>()
                .unwrap_or_else(|| panic!("{self:?} isn't one of vanilla's payloads"))
                .encode()
        }
    }
}
