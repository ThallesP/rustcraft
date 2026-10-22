//! net/minecraft/network/protocol/login/ClientLoginPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s, and
//! `handleLoginFinished`, `handleDisconnect` and `handleCustomQuery`, which come with
//! `ClientboundLoginFinishedPacket`, `ClientboundLoginDisconnectPacket` and
//! `ClientboundCustomQueryPacket`.

use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;
use crate::net::minecraft::network::protocol::login::clientbound_hello_packet::ClientboundHelloPacket;
use crate::net::minecraft::network::protocol::login::clientbound_login_compression_packet::ClientboundLoginCompressionPacket;

/// The client's listener while it logs in.
pub trait ClientLoginPacketListener: ClientCookiePacketListener {
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket);

    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket);
}

#[cfg(test)]
impl ClientLoginPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_hello(&mut self, packet: &ClientboundHelloPacket) {
        self.record("handleHello", packet);
    }

    fn handle_compression(&mut self, packet: &ClientboundLoginCompressionPacket) {
        self.record("handleCompression", packet);
    }
}
