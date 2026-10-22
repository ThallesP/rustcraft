//! net/minecraft/network/ServerboundPacketListener.java
//!
//! Not ported yet: its `flow()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::packet_listener::PacketListener;

/// A listener on the server's side, which serverbound packets go to.
pub trait ServerboundPacketListener: PacketListener {}

#[cfg(test)]
impl ServerboundPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
}
