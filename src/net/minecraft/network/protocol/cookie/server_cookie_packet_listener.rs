//! net/minecraft/network/protocol/cookie/ServerCookiePacketListener.java

use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::game::server_packet_listener::ServerPacketListener;

/// A server's listener in a protocol where it may ask for a cookie (login, configuration and
/// play).
pub trait ServerCookiePacketListener: ServerPacketListener {
    fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket);
}

#[cfg(test)]
impl ServerCookiePacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_cookie_response(&mut self, packet: &ServerboundCookieResponsePacket) {
        self.record("handleCookieResponse", packet);
    }
}
