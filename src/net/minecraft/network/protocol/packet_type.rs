//! net/minecraft/network/protocol/PacketType.java
//!
//! Not ported: `hashCode`, a record's, which hashes `PacketFlow`'s identity hash and so changes
//! from run to run. A protocol's codec only looks types up, which Rust's own hash does as well.
//!
//! Java's `PacketType<T extends Packet<?>>` names the packet class it's the type of, which only
//! the compiler reads, so it's a plain `PacketType` here.

use std::fmt;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::resources::identifier::Identifier;

/// A packet's name, `minecraft:intention`, and which way it goes. A protocol's codec picks the
/// packet's id by its type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PacketType {
    flow: PacketFlow,
    id: Identifier,
}

impl PacketType {
    pub fn new(flow: PacketFlow, id: Identifier) -> PacketType {
        PacketType { flow, id }
    }

    pub fn flow(&self) -> PacketFlow {
        self.flow
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }
}

/// Like `serverbound/minecraft:intention`.
impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.flow.id(), self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::PacketType;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(PacketType {
        flow: PacketFlow,
        id: Identifier
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.PacketType");
        golden.pending(
            "hashCode()I",
            "hashes PacketFlow's identity hash, which changes from run to run",
        );
        golden.check(
            "<init>(Lnet/minecraft/network/protocol/PacketFlow;Lnet/minecraft/resources/Identifier;)V",
            PacketType::new,
        );
        golden.check_method(
            "flow()Lnet/minecraft/network/protocol/PacketFlow;",
            |this: &mut PacketType| this.flow(),
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut PacketType| this.id().clone(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut PacketType, other: PacketType| *this == other,
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut PacketType| {
            this.to_string()
        });
        golden.finish();
    }
}
