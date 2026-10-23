//! net/minecraft/network/protocol/status/StatusPacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_STATUS_RESPONSE: LazyLock<PacketType> =
    LazyLock::new(|| StatusPacketTypes::create_clientbound("status_response"));
static SERVERBOUND_STATUS_REQUEST: LazyLock<PacketType> =
    LazyLock::new(|| StatusPacketTypes::create_serverbound("status_request"));

/// The status protocol's own packet types. Its ping and pong are the ping packets' types.
pub struct StatusPacketTypes;

impl StatusPacketTypes {
    pub const CLIENTBOUND_STATUS_RESPONSE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_STATUS_RESPONSE;
    pub const SERVERBOUND_STATUS_REQUEST: &'static LazyLock<PacketType> =
        &SERVERBOUND_STATUS_REQUEST;

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
    use super::StatusPacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.status.StatusPacketTypes");
        for (name, r#type) in [
            (
                "CLIENTBOUND_STATUS_RESPONSE",
                StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE,
            ),
            (
                "SERVERBOUND_STATUS_REQUEST",
                StatusPacketTypes::SERVERBOUND_STATUS_REQUEST,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
