//! net/minecraft/network/protocol/handshake/HandshakeProtocols.java

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;
use crate::net::minecraft::network::protocol::handshake::handshake_packet_types::HandshakePacketTypes;
use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol_info::StaticProtocolInfo;

static SERVERBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ServerHandshakePacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::HANDSHAKING, |builder| {
        builder.add_packet(
            HandshakePacketTypes::CLIENT_INTENTION,
            ClientIntentionPacket::STREAM_CODEC,
        );
    })
});

static SERVERBOUND: StaticProtocolInfo<dyn ServerHandshakePacketListener> =
    LazyLock::new(|| SERVERBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

/// The handshake: one packet, from the client, which says what the connection is for.
pub struct HandshakeProtocols;

impl HandshakeProtocols {
    pub const SERVERBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ServerHandshakePacketListener,
        FriendlyByteBuf,
    > = &SERVERBOUND_TEMPLATE;
    pub const SERVERBOUND: &'static StaticProtocolInfo<dyn ServerHandshakePacketListener> =
        &SERVERBOUND;
}

#[cfg(test)]
mod tests {
    use super::HandshakeProtocols;
    use crate::golden::{Golden, encode_packets};
    use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;
    use crate::net::minecraft::network::protocol::handshake::handshake_packet_types::HandshakePacketTypes;
    use crate::net::minecraft::network::protocol::handshake::server_handshake_packet_listener::ServerHandshakePacketListener;
    use crate::net::minecraft::network::protocol_info::tests::listed;

    encode_packets!(dyn ServerHandshakePacketListener {
        HandshakePacketTypes::CLIENT_INTENTION => ClientIntentionPacket,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.handshake.HandshakeProtocols");
        golden.field(
            "SERVERBOUND_TEMPLATE",
            listed(HandshakeProtocols::SERVERBOUND_TEMPLATE.details()),
        );
        golden.field(
            "SERVERBOUND",
            (
                HandshakeProtocols::SERVERBOUND.id(),
                HandshakeProtocols::SERVERBOUND.flow(),
            ),
        );
        golden.check_codec("SERVERBOUND.codec()", || {
            HandshakeProtocols::SERVERBOUND.codec()
        });
        golden.finish();
    }
}
