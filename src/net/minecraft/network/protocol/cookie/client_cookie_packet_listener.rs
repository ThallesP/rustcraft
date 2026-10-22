//! net/minecraft/network/protocol/cookie/ClientCookiePacketListener.java

use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;

/// A client's listener in a protocol where the server may ask for a cookie (login, configuration
/// and play).
pub trait ClientCookiePacketListener: ClientboundPacketListener {
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket);
}

#[cfg(test)]
impl ClientCookiePacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_request_cookie(&mut self, packet: &ClientboundCookieRequestPacket) {
        self.record("handleRequestCookie", packet);
    }
}
