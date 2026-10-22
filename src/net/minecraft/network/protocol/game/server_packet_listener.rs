//! net/minecraft/network/protocol/game/ServerPacketListener.java
//!
//! Not ported yet: its `LOGGER` and `onPacketError`, which logs a packet's error instead of
//! throwing it, and comes with `PacketListener`'s.

use crate::net::minecraft::network::serverbound_packet_listener::ServerboundPacketListener;

/// A listener on the server's side of every protocol.
pub trait ServerPacketListener: ServerboundPacketListener {}

#[cfg(test)]
impl ServerPacketListener for crate::net::minecraft::network::packet_listener::tests::Recorder {}
