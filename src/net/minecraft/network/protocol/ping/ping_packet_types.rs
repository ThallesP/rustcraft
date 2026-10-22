//! net/minecraft/network/protocol/ping/PingPacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_PONG_RESPONSE: LazyLock<PacketType> =
    LazyLock::new(|| PingPacketTypes::create_clientbound("pong_response"));
static SERVERBOUND_PING_REQUEST: LazyLock<PacketType> =
    LazyLock::new(|| PingPacketTypes::create_serverbound("ping_request"));

/// The types of the ping packets, which the status protocol and the game protocol share.
pub struct PingPacketTypes;

impl PingPacketTypes {
    pub const CLIENTBOUND_PONG_RESPONSE: &'static LazyLock<PacketType> = &CLIENTBOUND_PONG_RESPONSE;
    pub const SERVERBOUND_PING_REQUEST: &'static LazyLock<PacketType> = &SERVERBOUND_PING_REQUEST;

    fn create_clientbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::CLIENTBOUND,
            Identifier::with_default_namespace(id),
        )
    }

    fn create_serverbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::SERVERBOUND,
            Identifier::with_default_namespace(id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PingPacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.ping.PingPacketTypes");
        for (name, r#type) in [
            (
                "CLIENTBOUND_PONG_RESPONSE",
                PingPacketTypes::CLIENTBOUND_PONG_RESPONSE,
            ),
            (
                "SERVERBOUND_PING_REQUEST",
                PingPacketTypes::SERVERBOUND_PING_REQUEST,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
