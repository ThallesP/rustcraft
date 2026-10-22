//! net/minecraft/network/TickablePacketListener.java

use crate::net::minecraft::network::packet_listener::PacketListener;

/// A listener its connection ticks, once per client tick.
pub trait TickablePacketListener: PacketListener {
    fn tick(&mut self);
}
