//! net/minecraft/network/protocol/login/ServerLoginPacketListener.java
//!
//! Not ported yet: its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::cookie::server_cookie_packet_listener::ServerCookiePacketListener;
use crate::net::minecraft::network::protocol::login::serverbound_custom_query_answer_packet::ServerboundCustomQueryAnswerPacket;
use crate::net::minecraft::network::protocol::login::serverbound_hello_packet::ServerboundHelloPacket;
use crate::net::minecraft::network::protocol::login::serverbound_key_packet::ServerboundKeyPacket;
use crate::net::minecraft::network::protocol::login::serverbound_login_acknowledged_packet::ServerboundLoginAcknowledgedPacket;

/// The server's listener while a client logs in.
pub trait ServerLoginPacketListener: ServerCookiePacketListener {
    fn handle_hello(&mut self, packet: &ServerboundHelloPacket);

    fn handle_key(&mut self, packet: &ServerboundKeyPacket);

    fn handle_custom_query_packet(&mut self, packet: &ServerboundCustomQueryAnswerPacket);

    fn handle_login_acknowledgement(&mut self, packet: &ServerboundLoginAcknowledgedPacket);
}

#[cfg(test)]
impl ServerLoginPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_hello(&mut self, packet: &ServerboundHelloPacket) {
        self.record("handleHello", packet);
    }

    fn handle_key(&mut self, packet: &ServerboundKeyPacket) {
        self.record("handleKey", packet);
    }

    fn handle_custom_query_packet(&mut self, packet: &ServerboundCustomQueryAnswerPacket) {
        self.record("handleCustomQueryPacket", packet);
    }

    fn handle_login_acknowledgement(&mut self, packet: &ServerboundLoginAcknowledgedPacket) {
        self.record("handleLoginAcknowledgement", packet);
    }
}
