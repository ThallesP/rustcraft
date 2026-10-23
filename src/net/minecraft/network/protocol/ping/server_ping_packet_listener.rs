//! net/minecraft/network/protocol/ping/ServerPingPacketListener.java

use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::ping::serverbound_ping_request_packet::ServerboundPingRequestPacket;

/// A server's listener in the protocols that take a ping request: status and play.
pub trait ServerPingPacketListener: PacketListener {
    fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket);
}

#[cfg(test)]
impl ServerPingPacketListener for crate::net::minecraft::network::packet_listener::tests::Recorder {
    fn handle_ping_request(&mut self, packet: &ServerboundPingRequestPacket) {
        self.record("handlePingRequest", packet);
    }
}
