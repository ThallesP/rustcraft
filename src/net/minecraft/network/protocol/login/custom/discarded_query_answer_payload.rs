//! net/minecraft/network/protocol/login/custom/DiscardedQueryAnswerPayload.java

use std::any::Any;
use std::fmt;

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::custom::custom_query_answer_payload::CustomQueryAnswerPayload;

/// The answer a server reads whatever a client answered as: it skips the bytes. A record without
/// components, so every one equals every other.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscardedQueryAnswerPayload;

impl DiscardedQueryAnswerPayload {
    pub const INSTANCE: DiscardedQueryAnswerPayload = DiscardedQueryAnswerPayload;

    pub fn new() -> DiscardedQueryAnswerPayload {
        DiscardedQueryAnswerPayload
    }
}

impl Default for DiscardedQueryAnswerPayload {
    fn default() -> DiscardedQueryAnswerPayload {
        DiscardedQueryAnswerPayload::new()
    }
}

impl CustomQueryAnswerPayload for DiscardedQueryAnswerPayload {
    /// Writes nothing.
    fn write(&self, _output: &mut FriendlyByteBuf) {}

    /// A record's: any other `DiscardedQueryAnswerPayload`, since it has no components.
    fn equals(&self, other: &dyn CustomQueryAnswerPayload) -> bool {
        let other: &dyn Any = other;
        other.is::<DiscardedQueryAnswerPayload>()
    }

    /// A record's without components: 0.
    fn hash_code(&self) -> i32 {
        0
    }
}

/// A record's: `DiscardedQueryAnswerPayload[]`.
impl fmt::Display for DiscardedQueryAnswerPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DiscardedQueryAnswerPayload[]")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::DiscardedQueryAnswerPayload;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::login::custom::custom_query_answer_payload::CustomQueryAnswerPayload;

    // A record without components: [].
    impl Decode for DiscardedQueryAnswerPayload {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no components, got {value}"
            );
            DiscardedQueryAnswerPayload::INSTANCE
        }
    }

    impl Encode for DiscardedQueryAnswerPayload {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload");
        golden.field("INSTANCE", DiscardedQueryAnswerPayload::INSTANCE);
        golden.check("<init>()V", DiscardedQueryAnswerPayload::new);
        golden.check_mut(
            "write(Lnet/minecraft/network/FriendlyByteBuf;)V",
            |this: &mut DiscardedQueryAnswerPayload, output: &mut FriendlyByteBuf| {
                this.write(output)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut DiscardedQueryAnswerPayload, other: DiscardedQueryAnswerPayload| {
                this.equals(&other)
            },
        );
        golden.check_method("hashCode()I", |this: &mut DiscardedQueryAnswerPayload| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut DiscardedQueryAnswerPayload| this.to_string(),
        );
        golden.finish();
    }
}
