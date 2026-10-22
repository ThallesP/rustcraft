//! net/minecraft/network/protocol/login/LoginPacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_CUSTOM_QUERY: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_clientbound("custom_query"));
static CLIENTBOUND_LOGIN_FINISHED: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_clientbound("login_finished"));
static CLIENTBOUND_HELLO: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_clientbound("hello"));
static CLIENTBOUND_LOGIN_COMPRESSION: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_clientbound("login_compression"));
static CLIENTBOUND_LOGIN_DISCONNECT: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_clientbound("login_disconnect"));
static SERVERBOUND_CUSTOM_QUERY_ANSWER: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_serverbound("custom_query_answer"));
static SERVERBOUND_HELLO: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_serverbound("hello"));
static SERVERBOUND_KEY: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_serverbound("key"));
static SERVERBOUND_LOGIN_ACKNOWLEDGED: LazyLock<PacketType> =
    LazyLock::new(|| LoginPacketTypes::create_serverbound("login_acknowledged"));

/// The login's packet types.
pub struct LoginPacketTypes;

impl LoginPacketTypes {
    pub const CLIENTBOUND_CUSTOM_QUERY: &'static LazyLock<PacketType> = &CLIENTBOUND_CUSTOM_QUERY;
    pub const CLIENTBOUND_LOGIN_FINISHED: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LOGIN_FINISHED;
    pub const CLIENTBOUND_HELLO: &'static LazyLock<PacketType> = &CLIENTBOUND_HELLO;
    pub const CLIENTBOUND_LOGIN_COMPRESSION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LOGIN_COMPRESSION;
    pub const CLIENTBOUND_LOGIN_DISCONNECT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LOGIN_DISCONNECT;
    pub const SERVERBOUND_CUSTOM_QUERY_ANSWER: &'static LazyLock<PacketType> =
        &SERVERBOUND_CUSTOM_QUERY_ANSWER;
    pub const SERVERBOUND_HELLO: &'static LazyLock<PacketType> = &SERVERBOUND_HELLO;
    pub const SERVERBOUND_KEY: &'static LazyLock<PacketType> = &SERVERBOUND_KEY;
    pub const SERVERBOUND_LOGIN_ACKNOWLEDGED: &'static LazyLock<PacketType> =
        &SERVERBOUND_LOGIN_ACKNOWLEDGED;

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
    use super::LoginPacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.login.LoginPacketTypes");
        for (name, r#type) in [
            (
                "CLIENTBOUND_CUSTOM_QUERY",
                LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY,
            ),
            (
                "CLIENTBOUND_LOGIN_FINISHED",
                LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED,
            ),
            ("CLIENTBOUND_HELLO", LoginPacketTypes::CLIENTBOUND_HELLO),
            (
                "CLIENTBOUND_LOGIN_COMPRESSION",
                LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION,
            ),
            (
                "CLIENTBOUND_LOGIN_DISCONNECT",
                LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT,
            ),
            (
                "SERVERBOUND_CUSTOM_QUERY_ANSWER",
                LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER,
            ),
            ("SERVERBOUND_HELLO", LoginPacketTypes::SERVERBOUND_HELLO),
            ("SERVERBOUND_KEY", LoginPacketTypes::SERVERBOUND_KEY),
            (
                "SERVERBOUND_LOGIN_ACKNOWLEDGED",
                LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
