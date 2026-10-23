//! net/minecraft/network/protocol/ping/ClientPongPacketListener.java

use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;

/// A client's listener in the protocols that answer a ping request: status and play.
pub trait ClientPongPacketListener: PacketListener {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket);
}

#[cfg(test)]
impl ClientPongPacketListener for crate::net::minecraft::network::packet_listener::tests::Recorder {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
        self.record("handlePongResponse", packet);
    }
}
