//! net/minecraft/network/protocol/BundlerInfo.java
//!
//! Not ported yet: `unbundlePacket`, which a server's encoder calls (`PacketBundleUnpacker`) to
//! send a bundle's packets between two delimiters; a client never sends a bundle.
//!
//! Java's `BundlerInfo` takes any `Packet<?>`; a connection's decoder hands on its protocol's
//! packets, `Box<dyn Packet<T>>` for its listener `T`, so here it's `BundlerInfo<T>`. Java tells
//! the delimiter by reference (`packet == delimiterPacket`), the one instance its codec decodes
//! to (`StreamCodec.unit`), which here is its type: no other packet has it.

use std::sync::Arc;

use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// How a protocol's bundles are told apart: the packets between two delimiters, which the
/// listener handles together.
pub trait BundlerInfo<T: ?Sized>: Send + Sync {
    /// A bundler for the packets after `packet`, if it's the delimiter that starts a bundle.
    fn start_packet_bundling(&self, packet: &dyn Packet<T>) -> Option<Box<dyn Bundler<T>>>;
}

/// The most packets a bundle may hold.
pub const BUNDLE_SIZE_LIMIT: usize = 4096;

/// `BundlerInfo.Bundler`: the packets of the bundle being read.
pub trait Bundler<T: ?Sized>: Send {
    /// Takes the bundle's next packet, and gives back the bundle once `packet` is the delimiter
    /// that ends it.
    fn add_packet(&mut self, packet: Box<dyn Packet<T>>) -> Option<Box<dyn Packet<T>>>;
}

/// Java's `Function<Iterable<Packet<? super T>>, P>`, boxing the bundle it makes.
type Constructor<T> = dyn Fn(Vec<Box<dyn Packet<T>>>) -> Box<dyn Packet<T>> + Send + Sync;

/// `createForPacket`: bundles of the packet `constructor` makes, between delimiters of
/// `delimiter_type`. The bundle's type is what `unbundlePacket` looks for, which isn't ported.
pub fn create_for_packet<T: ?Sized + 'static, P: Packet<T>>(
    _bundle_packet_type: &'static PacketType,
    constructor: impl Fn(Vec<Box<dyn Packet<T>>>) -> P + Send + Sync + 'static,
    delimiter_type: &'static PacketType,
) -> impl BundlerInfo<T> {
    ForPacket {
        constructor: Arc::new(move |packets| Box::new(constructor(packets)) as Box<dyn Packet<T>>),
        delimiter_type,
    }
}

/// The anonymous `BundlerInfo` of `createForPacket`.
struct ForPacket<T: ?Sized> {
    constructor: Arc<Constructor<T>>,
    delimiter_type: &'static PacketType,
}

impl<T: ?Sized + 'static> BundlerInfo<T> for ForPacket<T> {
    fn start_packet_bundling(&self, packet: &dyn Packet<T>) -> Option<Box<dyn Bundler<T>>> {
        (packet.r#type() == self.delimiter_type).then(|| {
            Box::new(PacketBundler {
                constructor: Arc::clone(&self.constructor),
                delimiter_type: self.delimiter_type,
                bundle_packets: Vec::new(),
            }) as Box<dyn Bundler<T>>
        })
    }
}

/// The anonymous `Bundler` of `startPacketBundling`.
struct PacketBundler<T: ?Sized> {
    constructor: Arc<Constructor<T>>,
    delimiter_type: &'static PacketType,
    bundle_packets: Vec<Box<dyn Packet<T>>>,
}

impl<T: ?Sized + 'static> Bundler<T> for PacketBundler<T> {
    fn add_packet(&mut self, packet: Box<dyn Packet<T>>) -> Option<Box<dyn Packet<T>>> {
        if packet.r#type() == self.delimiter_type {
            return Some((self.constructor)(std::mem::take(&mut self.bundle_packets)));
        }

        if self.bundle_packets.len() >= BUNDLE_SIZE_LIMIT {
            panic!("IllegalStateException: Too many packets in a bundle");
        }

        self.bundle_packets.push(packet);
        None
    }
}
