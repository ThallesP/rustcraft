//! net/minecraft/network/protocol/login/LoginProtocols.java
//!
//! Not ported yet: `ClientboundLoginDisconnectPacket`, which reads its reason with `Component`'s
//! codec: the clientbound protocol has the `Unported` stand-in for it, which the client answers
//! nothing to anyway (the server closes the connection after it).

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
use crate::net::minecraft::network::protocol::login::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
use crate::net::minecraft::network::protocol::login::clientbound_hello_packet::ClientboundHelloPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;
use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
use crate::net::minecraft::network::protocol::login::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
use crate::net::minecraft::network::protocol::login::serverbound_hello_packet::ServerboundHelloPacket;
use crate::net::minecraft::network::protocol::login::serverbound_key_packet::ServerboundKeyPacket;
use crate::net::minecraft::network::protocol::login::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol_info::StaticProtocolInfo;

static SERVERBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ServerLoginPacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::LOGIN, |builder| {
        builder
            .add_packet(
                LoginPacketTypes::SERVERBOUND_HELLO,
                ServerboundHelloPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::SERVERBOUND_KEY,
                ServerboundKeyPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER,
                ServerboundCustomQueryAnswerPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED,
                ServerboundLoginAcknowledgedPacket::STREAM_CODEC,
            )
            .add_packet(
                CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE,
                ServerboundCookieResponsePacket::STREAM_CODEC,
            );
    })
});

static SERVERBOUND: StaticProtocolInfo<dyn ServerLoginPacketListener> =
    LazyLock::new(|| SERVERBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

static CLIENTBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ClientLoginPacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::LOGIN, |builder| {
        builder
            .add_packet(
                LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT,
                ClientboundLoginDisconnectPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::CLIENTBOUND_HELLO,
                ClientboundHelloPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED,
                ClientboundLoginFinishedPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION,
                ClientboundLoginCompressionPacket::STREAM_CODEC,
            )
            .add_packet(
                LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY,
                ClientboundCustomQueryPacket::STREAM_CODEC,
            )
            .add_packet(
                CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST,
                ClientboundCookieRequestPacket::STREAM_CODEC,
            );
    })
});

static CLIENTBOUND: StaticProtocolInfo<dyn ClientLoginPacketListener> =
    LazyLock::new(|| CLIENTBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

/// Login: the client says who it is, the two sides may set up encryption, compression and custom
/// queries, and the server says the login is finished, which the client acknowledges.
pub struct LoginProtocols;

impl LoginProtocols {
    pub const CLIENTBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ClientLoginPacketListener,
        FriendlyByteBuf,
    > = &CLIENTBOUND_TEMPLATE;
    pub const CLIENTBOUND: &'static StaticProtocolInfo<dyn ClientLoginPacketListener> =
        &CLIENTBOUND;
    pub const SERVERBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ServerLoginPacketListener,
        FriendlyByteBuf,
    > = &SERVERBOUND_TEMPLATE;
    pub const SERVERBOUND: &'static StaticProtocolInfo<dyn ServerLoginPacketListener> =
        &SERVERBOUND;
}

#[cfg(test)]
mod tests {
    use super::LoginProtocols;
    use crate::golden::{Golden, encode_packets};
    use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
    use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
    use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
    use crate::net::minecraft::network::protocol::login::client_login_packet_listener::ClientLoginPacketListener;
    use crate::net::minecraft::network::protocol::login::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
    use crate::net::minecraft::network::protocol::login::clientbound_hello_packet::ClientboundHelloPacket;
    use crate::net::minecraft::network::protocol::login::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
    use crate::net::minecraft::network::protocol::login::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket;
    use crate::net::minecraft::network::protocol::login::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;
    use crate::net::minecraft::network::protocol::login::login_packet_types::LoginPacketTypes;
    use crate::net::minecraft::network::protocol::login::server_login_packet_listener::ServerLoginPacketListener;
    use crate::net::minecraft::network::protocol::login::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
    use crate::net::minecraft::network::protocol::login::serverbound_hello_packet::ServerboundHelloPacket;
    use crate::net::minecraft::network::protocol::login::serverbound_key_packet::ServerboundKeyPacket;
    use crate::net::minecraft::network::protocol::login::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;
    use crate::net::minecraft::network::protocol_info::tests::listed;

    encode_packets!(dyn ClientLoginPacketListener {
        LoginPacketTypes::CLIENTBOUND_LOGIN_DISCONNECT => ClientboundLoginDisconnectPacket,
        LoginPacketTypes::CLIENTBOUND_HELLO => ClientboundHelloPacket,
        LoginPacketTypes::CLIENTBOUND_LOGIN_FINISHED => ClientboundLoginFinishedPacket,
        LoginPacketTypes::CLIENTBOUND_LOGIN_COMPRESSION => ClientboundLoginCompressionPacket,
        LoginPacketTypes::CLIENTBOUND_CUSTOM_QUERY => ClientboundCustomQueryPacket,
        CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST => ClientboundCookieRequestPacket,
    });

    encode_packets!(dyn ServerLoginPacketListener {
        LoginPacketTypes::SERVERBOUND_HELLO => ServerboundHelloPacket,
        LoginPacketTypes::SERVERBOUND_KEY => ServerboundKeyPacket,
        LoginPacketTypes::SERVERBOUND_CUSTOM_QUERY_ANSWER => ServerboundCustomQueryAnswerPacket,
        LoginPacketTypes::SERVERBOUND_LOGIN_ACKNOWLEDGED => ServerboundLoginAcknowledgedPacket,
        CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE => ServerboundCookieResponsePacket,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.login.LoginProtocols");
        golden.field(
            "CLIENTBOUND_TEMPLATE",
            listed(LoginProtocols::CLIENTBOUND_TEMPLATE.details()),
        );
        golden.field(
            "CLIENTBOUND",
            (
                LoginProtocols::CLIENTBOUND.id(),
                LoginProtocols::CLIENTBOUND.flow(),
            ),
        );
        golden.check_codec("CLIENTBOUND.codec()", || {
            LoginProtocols::CLIENTBOUND.codec()
        });
        golden.field(
            "SERVERBOUND_TEMPLATE",
            listed(LoginProtocols::SERVERBOUND_TEMPLATE.details()),
        );
        golden.field(
            "SERVERBOUND",
            (
                LoginProtocols::SERVERBOUND.id(),
                LoginProtocols::SERVERBOUND.flow(),
            ),
        );
        golden.check_codec("SERVERBOUND.codec()", || {
            LoginProtocols::SERVERBOUND.codec()
        });
        golden.finish();
    }
}
