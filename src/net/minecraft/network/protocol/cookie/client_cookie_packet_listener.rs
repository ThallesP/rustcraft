//! net/minecraft/network/protocol/cookie/ClientCookiePacketListener.java
//!
//! Not ported yet: `handleRequestCookie`, which comes with `ClientboundCookieRequestPacket`.

use crate::net::minecraft::network::clientbound_packet_listener::ClientboundPacketListener;

/// A client's listener in a protocol where the server may ask for a cookie (login, configuration
/// and play).
pub trait ClientCookiePacketListener: ClientboundPacketListener {}

#[cfg(test)]
impl ClientCookiePacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
}
