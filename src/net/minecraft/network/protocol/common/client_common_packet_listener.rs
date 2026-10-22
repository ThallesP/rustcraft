//! net/minecraft/network/protocol/common/ClientCommonPacketListener.java
//!
//! Not ported yet: every handler but `handleStoreCookie`, which come with their packets in the
//! configuration protocol.

use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::cookie::client_cookie_packet_listener::ClientCookiePacketListener;

/// A client's listener in the protocols after login (configuration and play), for the packets
/// both have.
pub trait ClientCommonPacketListener: ClientCookiePacketListener {
    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket);
}

#[cfg(test)]
impl ClientCommonPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
    fn handle_store_cookie(&mut self, packet: &ClientboundStoreCookiePacket) {
        self.record("handleStoreCookie", packet);
    }
}
