//! net/minecraft/network/protocol/Packet.java
//!
//! Java's `Packet<T extends PacketListener>` is `Packet<T>`, for the listener `T` it's handled
//! by. A protocol's codec reads and writes `Box<dyn Packet<L>>` for its listener `L`, which takes
//! packets of `L`'s parents too (Java's `Packet<? super L>`): a packet any listener of a kind
//! handles implements `Packet<L>` for every such `L`, like the login packets for every
//! `ClientLoginPacketListener`. `type()` returns the shared constant (`&'static PacketType`), and
//! is `r#type`.

use std::any::Any;
use std::fmt::Debug;

use crate::net::minecraft::network::codec::stream_codec::{self, StreamCodec};
use crate::net::minecraft::network::codec::stream_decoder::StreamDecoder;
use crate::net::minecraft::network::codec::stream_member_encoder::StreamMemberEncoder;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

/// A message one side sends the other. `Any` lets a protocol's codec cast it back to its own
/// class to write it.
pub trait Packet<T: ?Sized>: Any + Debug + Send + Sync {
    fn r#type(&self) -> &'static PacketType;

    /// Calls the listener's handler for this packet.
    fn handle(&self, listener: &mut T);

    fn is_skippable(&self) -> bool {
        false
    }

    /// Whether the packet ends its protocol, so the connection moves on to the next one.
    fn is_terminal(&self) -> bool {
        false
    }
}

/// `codec(StreamMemberEncoder, StreamDecoder)`: a packet's codec, from its `write` method and
/// its constructor reading a buffer.
pub fn codec<B, T>(
    writer: impl StreamMemberEncoder<B, T>,
    reader: impl StreamDecoder<B, T>,
) -> impl StreamCodec<Buffer = B, Value = T> {
    stream_codec::of_member(writer, reader)
}
