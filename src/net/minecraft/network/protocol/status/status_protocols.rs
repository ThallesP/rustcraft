//! net/minecraft/network/protocol/status/StatusProtocols.java

use std::sync::{Arc, LazyLock};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use crate::net::minecraft::network::protocol::ping::ping_packet_types::PingPacketTypes;
use crate::net::minecraft::network::protocol::ping::serverbound_ping_request_packet::ServerboundPingRequestPacket;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol::status::client_status_packet_listener::ClientStatusPacketListener;
use crate::net::minecraft::network::protocol::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;
use crate::net::minecraft::network::protocol::status::server_status_packet_listener::ServerStatusPacketListener;
use crate::net::minecraft::network::protocol::status::serverbound_status_request_packet::ServerboundStatusRequestPacket;
use crate::net::minecraft::network::protocol::status::status_packet_types::StatusPacketTypes;
use crate::net::minecraft::network::protocol_info::StaticProtocolInfo;

static SERVERBOUND_TEMPLATE: StaticSimpleUnboundProtocol<dyn ServerStatusPacketListener, ByteBuf> =
    LazyLock::new(|| {
        ProtocolInfoBuilder::serverbound_protocol(ConnectionProtocol::STATUS, |builder| {
            builder
                .add_packet(
                    StatusPacketTypes::SERVERBOUND_STATUS_REQUEST,
                    ServerboundStatusRequestPacket::STREAM_CODEC,
                )
                .add_packet(
                    PingPacketTypes::SERVERBOUND_PING_REQUEST,
                    ServerboundPingRequestPacket::STREAM_CODEC,
                );
        })
    });

static SERVERBOUND: StaticProtocolInfo<dyn ServerStatusPacketListener> =
    LazyLock::new(|| SERVERBOUND_TEMPLATE.bind(Arc::new(|buffer| buffer)));

static CLIENTBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ClientStatusPacketListener,
    FriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::STATUS, |builder| {
        builder
            .add_packet(
                StatusPacketTypes::CLIENTBOUND_STATUS_RESPONSE,
                ClientboundStatusResponsePacket::STREAM_CODEC,
            )
            .add_packet(
                PingPacketTypes::CLIENTBOUND_PONG_RESPONSE,
                ClientboundPongResponsePacket::STREAM_CODEC,
            );
    })
});

static CLIENTBOUND: StaticProtocolInfo<dyn ClientStatusPacketListener> =
    LazyLock::new(|| CLIENTBOUND_TEMPLATE.bind(Arc::new(FriendlyByteBuf::new)));

/// The status protocol, the server list's ping: the client asks for the server's status, then
/// pings it, and the server answers each.
pub struct StatusProtocols;

impl StatusProtocols {
    pub const SERVERBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ServerStatusPacketListener,
        ByteBuf,
    > = &SERVERBOUND_TEMPLATE;
    pub const SERVERBOUND: &'static StaticProtocolInfo<dyn ServerStatusPacketListener> =
        &SERVERBOUND;
    pub const CLIENTBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ClientStatusPacketListener,
        FriendlyByteBuf,
    > = &CLIENTBOUND_TEMPLATE;
    pub const CLIENTBOUND: &'static StaticProtocolInfo<dyn ClientStatusPacketListener> =
        &CLIENTBOUND;
}

#[cfg(test)]
mod tests {
    use super::StatusProtocols;
    use crate::golden::{Golden, encode_packets};
    use crate::net::minecraft::network::protocol::ping::ping_packet_types::PingPacketTypes;
    use crate::net::minecraft::network::protocol::ping::serverbound_ping_request_packet::ServerboundPingRequestPacket;
    use crate::net::minecraft::network::protocol::status::server_status_packet_listener::ServerStatusPacketListener;
    use crate::net::minecraft::network::protocol::status::serverbound_status_request_packet::ServerboundStatusRequestPacket;
    use crate::net::minecraft::network::protocol::status::status_packet_types::StatusPacketTypes;
    use crate::net::minecraft::network::protocol_info::tests::listed;

    encode_packets!(dyn ServerStatusPacketListener {
        StatusPacketTypes::SERVERBOUND_STATUS_REQUEST => ServerboundStatusRequestPacket,
        PingPacketTypes::SERVERBOUND_PING_REQUEST => ServerboundPingRequestPacket,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.status.StatusProtocols");
        golden.field(
            "CLIENTBOUND_TEMPLATE",
            listed(StatusProtocols::CLIENTBOUND_TEMPLATE.details()),
        );
        golden.field(
            "CLIENTBOUND",
            (
                StatusProtocols::CLIENTBOUND.id(),
                StatusProtocols::CLIENTBOUND.flow(),
            ),
        );
        golden.pending(
            "CLIENTBOUND.codec()",
            "needs ClientboundStatusResponsePacket's ServerStatus, which needs DataFixerUpper's codecs and text components",
        );
        golden.field(
            "SERVERBOUND_TEMPLATE",
            listed(StatusProtocols::SERVERBOUND_TEMPLATE.details()),
        );
        golden.field(
            "SERVERBOUND",
            (
                StatusProtocols::SERVERBOUND.id(),
                StatusProtocols::SERVERBOUND.flow(),
            ),
        );
        golden.check_codec("SERVERBOUND.codec()", || {
            StatusProtocols::SERVERBOUND.codec()
        });
        golden.finish();
    }
}
