//! net/minecraft/network/ProtocolInfo.java
//!
//! Not ported yet: `bundlerInfo()`, which only the game protocol has, and comes with it.
//!
//! `Details.PacketVisitor` is a functional interface, so `list_packets` takes a closure.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// A protocol bound to the buffer its packets are read from, held in a static field and made on
/// first use: Java's `static final ProtocolInfo<T>`.
pub type StaticProtocolInfo<T> = LazyLock<Box<dyn ProtocolInfo<T>>>;

/// One side of a protocol: the packets that go one way, for the listener `T` that handles them.
pub trait ProtocolInfo<T: ?Sized>: Send + Sync {
    fn id(&self) -> ConnectionProtocol;

    fn flow(&self) -> PacketFlow;

    /// Reads and writes the protocol's packets, each its id then its own codec's bytes.
    fn codec(&self) -> &dyn StreamCodec<Buffer = ByteBuf, Value = Box<dyn Packet<T>>>;
}

/// `ProtocolInfo.Details`: what a protocol is, without its codec.
pub trait Details: Send + Sync {
    fn id(&self) -> ConnectionProtocol;

    fn flow(&self) -> PacketFlow;

    /// Gives each packet's type and id, in the order of their ids.
    fn list_packets(&self, output: &mut dyn FnMut(&PacketType, i32));
}

/// `ProtocolInfo.DetailsProvider`.
pub trait DetailsProvider {
    fn details(&self) -> &dyn Details;
}

#[cfg(test)]
pub mod tests {
    use super::Details;
    use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
    use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
    use crate::net::minecraft::network::protocol::packet_type::PacketType;

    /// A template the way vanilla writes it: its details, `[id, flow, [[type, networkId], …]]`.
    pub fn listed(
        details: &dyn Details,
    ) -> (ConnectionProtocol, PacketFlow, Vec<(PacketType, i32)>) {
        let mut packets = Vec::new();
        details.list_packets(&mut |r#type, network_id| packets.push((r#type.clone(), network_id)));
        (details.id(), details.flow(), packets)
    }
}
