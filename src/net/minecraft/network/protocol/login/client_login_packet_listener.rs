//! net/minecraft/network/protocol/login/ClientLoginPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::net::minecraft::network::protocol::login::clientbound_custom_query_packet::ClientboundCustomQueryPacket;
use crate::net::minecraft::network::protocol::login::clientbound_hello_packet::ClientboundHelloPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_finished_packet::ClientboundLoginFinishedPacket;

/// The client's listener while it logs in.
pub trait ClientLoginPacketListener: ClientCookiePacketListener {
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket);

    fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket);

    fn handle_disconnect(&mut self, packet: &ClientboundLoginDisconnectPacket);

    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket);

    fn handle_custom_query(&mut self, packet: &ClientboundCustomQueryPacket);
}

#[cfg(test)]
impl ClientLoginPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket) {
        self.record("handleHello", packet);
    }

    fn handle_login_finished(&mut self, packet: &ClientboundLoginFinishedPacket) {
        self.record("handleLoginFinished", packet);
    }

    fn handle_disconnect(&mut self, packet: &ClientboundLoginDisconnectPacket) {
        self.record("handleDisconnect", packet);
    }

    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket) {
        self.record("handleCompression", packet);
    }

    fn handle_custom_query(&mut self, packet: &ClientboundCustomQueryPacket) {
        self.record("handleCustomQuery", packet);
    }
}
