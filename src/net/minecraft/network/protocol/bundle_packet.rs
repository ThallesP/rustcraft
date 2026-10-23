//! net/minecraft/network/protocol/BundlePacket.java
//!
//! Java's sub-packets are an `Iterable` the bundle shares by reference, so they're an `Arc`: a
//! copy of a bundle (the one a handler queues for the game thread) shares its packets.

use std::fmt;
use std::sync::Arc;

use crate::net::minecraft::network::protocol::packet::Packet;

/// The packets between two of a protocol's delimiters, which a listener handles together. Each
/// protocol's bundle class holds one and derefs to it.
pub struct BundlePacket<T: ?Sized> {
    packets: Arc<Vec<Box<dyn Packet<T>>>>,
}

impl<T: ?Sized> BundlePacket<T> {
    pub fn new(packets: Vec<Box<dyn Packet<T>>>) -> BundlePacket<T> {
        BundlePacket {
            packets: Arc::new(packets),
        }
    }

    pub fn sub_packets(&self) -> &[Box<dyn Packet<T>>] {
        &self.packets
    }
}

impl<T: ?Sized> Clone for BundlePacket<T> {
    fn clone(&self) -> BundlePacket<T> {
        BundlePacket {
            packets: Arc::clone(&self.packets),
        }
    }
}

impl<T: ?Sized> fmt::Debug for BundlePacket<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BundlePacket")
            .field("packets", &self.packets)
            .finish()
    }
}
