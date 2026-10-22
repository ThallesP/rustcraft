//! net/minecraft/network/protocol/cookie/ServerCookiePacketListener.java
//!
//! Not ported yet: `handleCookieResponse`, which comes with `ServerboundCookieResponsePacket`.

use crate::net::minecraft::network::protocol::game::server_packet_listener::ServerPacketListener;

/// A server's listener in a protocol where it may ask for a cookie (login, configuration and
/// play).
pub trait ServerCookiePacketListener: ServerPacketListener {}

#[cfg(test)]
impl ServerCookiePacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
}
