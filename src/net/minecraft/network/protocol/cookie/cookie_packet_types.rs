//! net/minecraft/network/protocol/cookie/CookiePacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_COOKIE_REQUEST: LazyLock<PacketType> =
    LazyLock::new(|| CookiePacketTypes::create_clientbound("cookie_request"));
static SERVERBOUND_COOKIE_RESPONSE: LazyLock<PacketType> =
    LazyLock::new(|| CookiePacketTypes::create_serverbound("cookie_response"));

/// The cookie packets' types, which login, configuration and play share.
pub struct CookiePacketTypes;

impl CookiePacketTypes {
    pub const CLIENTBOUND_COOKIE_REQUEST: &'static LazyLock<PacketType> =
        &CLIENTBOUND_COOKIE_REQUEST;
    pub const SERVERBOUND_COOKIE_RESPONSE: &'static LazyLock<PacketType> =
        &SERVERBOUND_COOKIE_RESPONSE;

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
    use super::CookiePacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.cookie.CookiePacketTypes");
        golden.field(
            "CLIENTBOUND_COOKIE_REQUEST",
            (**CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST).clone(),
        );
        golden.field(
            "SERVERBOUND_COOKIE_RESPONSE",
            (**CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE).clone(),
        );
        golden.finish();
    }
}
