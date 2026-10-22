//! net/minecraft/network/protocol/handshake/ServerHandshakePacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::game::server_packet_listener::ServerPacketListener;
use crate::net::minecraft::network::protocol::handshake::client_intention_packet::ClientIntentionPacket;

/// The server's listener while a connection shakes hands, which only takes the handshake.
pub trait ServerHandshakePacketListener: ServerPacketListener {
    fn handle_intention(&mut self, packet: &ClientIntentionPacket);
}

#[cfg(test)]
impl ServerHandshakePacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_intention(&mut self, packet: &ClientIntentionPacket) {
        self.record("handleIntention", packet);
    }
}
