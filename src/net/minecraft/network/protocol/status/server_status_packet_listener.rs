//! net/minecraft/network/protocol/status/ServerStatusPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::game::server_packet_listener::ServerPacketListener;
use crate::net::minecraft::network::protocol::ping::server_ping_packet_listener::ServerPingPacketListener;
use crate::net::minecraft::network::protocol::status::serverbound_status_request_packet::ServerboundStatusRequestPacket;

/// The server's listener while a client asks for its status.
pub trait ServerStatusPacketListener: ServerPacketListener + ServerPingPacketListener {
    fn handle_status_request(&mut self, packet: &ServerboundStatusRequestPacket);
}

#[cfg(test)]
impl ServerStatusPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_status_request(&mut self, packet: &ServerboundStatusRequestPacket) {
        self.record("handleStatusRequest", packet);
    }
}
