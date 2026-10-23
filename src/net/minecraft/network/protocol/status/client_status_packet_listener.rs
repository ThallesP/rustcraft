//! net/minecraft/network/protocol/status/ClientStatusPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use crate::net::minecraft::network::protocol::status::clientbound_status_response_packet::ClientboundStatusResponsePacket;

/// The client's listener while it asks a server for its status: the server list's ping.
pub trait ClientStatusPacketListener: ClientboundPacketListener + ClientPongPacketListener {
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket);
}

#[cfg(test)]
impl ClientStatusPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_status_response(&mut self, packet: &ClientboundStatusResponsePacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleStatusResponse", r#type);
    }
}
