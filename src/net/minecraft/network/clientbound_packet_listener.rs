//! net/minecraft/network/ClientboundPacketListener.java
//!
//! Not ported yet: its `flow()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::packet_listener::PacketListener;

/// A listener on the client's side, which clientbound packets go to.
pub trait ClientboundPacketListener: PacketListener {}

#[cfg(test)]
impl ClientboundPacketListener
    for crate::net::minecraft::network::packet_listener::tests::Recorder
{
}
