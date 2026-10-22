//! net/minecraft/network/protocol/handshake/HandshakePacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENT_INTENTION: LazyLock<PacketType> =
    LazyLock::new(|| HandshakePacketTypes::create_serverbound("intention"));

/// The handshake's packet types.
pub struct HandshakePacketTypes;

impl HandshakePacketTypes {
    pub const CLIENT_INTENTION: &'static LazyLock<PacketType> = &CLIENT_INTENTION;

    fn create_serverbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::SERVERBOUND,
            Identifier::with_default_namespace(id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::HandshakePacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.handshake.HandshakePacketTypes");
        golden.field(
            "CLIENT_INTENTION",
            (*HandshakePacketTypes::CLIENT_INTENTION).clone(),
        );
        golden.finish();
    }
}
