//! net/minecraft/network/protocol/login/custom/DiscardedQueryPayload.java

use std::any::Any;
use std::fmt;

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::login::custom::custom_query_payload::CustomQueryPayload;
use crate::net::minecraft::resources::identifier::Identifier;

/// The query a client reads whatever a server asked as: the query's identifier, and it skips the
/// bytes after it.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscardedQueryPayload {
    id: Identifier,
}

impl DiscardedQueryPayload {
    pub fn new(id: Identifier) -> DiscardedQueryPayload {
        DiscardedQueryPayload { id }
    }
}

impl CustomQueryPayload for DiscardedQueryPayload {
    fn id(&self) -> &Identifier {
        &self.id
    }

    /// Writes nothing: the packet writes the identifier.
    fn write(&self, _output: &mut FriendlyByteBuf) {}

    /// A record's: another `DiscardedQueryPayload` with an equal identifier.
    fn equals(&self, other: &dyn CustomQueryPayload) -> bool {
        let other: &dyn Any = other;
        other
            .downcast_ref::<DiscardedQueryPayload>()
            .is_some_and(|other| self == other)
    }

    /// A record's of one component: its hash.
    fn hash_code(&self) -> i32 {
        self.id.hash_code()
    }
}

/// A record's: `DiscardedQueryPayload[id=minecraft:stone]`.
impl fmt::Display for DiscardedQueryPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiscardedQueryPayload[id={}]", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::DiscardedQueryPayload;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::login::custom::custom_query_payload::CustomQueryPayload;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(DiscardedQueryPayload { id: Identifier });

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.login.custom.DiscardedQueryPayload");
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;)V",
            DiscardedQueryPayload::new,
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut DiscardedQueryPayload| this.id().clone(),
        );
        golden.check_mut(
            "write(Lnet/minecraft/network/FriendlyByteBuf;)V",
            |this: &mut DiscardedQueryPayload, output: &mut FriendlyByteBuf| this.write(output),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut DiscardedQueryPayload, other: DiscardedQueryPayload| this.equals(&other),
        );
        golden.check_method("hashCode()I", |this: &mut DiscardedQueryPayload| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut DiscardedQueryPayload| this.to_string(),
        );
        golden.finish();
    }
}
