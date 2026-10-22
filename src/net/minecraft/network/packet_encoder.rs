//! net/minecraft/network/PacketEncoder.java
//!
//! Not ported yet: the JFR profiler's and the debug log's record of each packet, and
//! `SkipPacketEncoderException` for a skippable packet that fails, which no serverbound packet
//! the port sends is. A Netty `MessageToByteEncoder` at the end of the connection's pipeline.
//! What `ProtocolSwapHandler.handleOutboundTerminalPacket` does after a terminal packet (stop
//! writing, until the next protocol's encoder is set up) is the connection's, which holds the
//! pipeline.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol_info::ProtocolInfo;

/// Writes each packet of a protocol, for the listener `T`, its id then its own codec's bytes.
pub struct PacketEncoder<T: ?Sized> {
    protocol_info: Box<dyn ProtocolInfo<T>>,
}

impl<T: ?Sized + 'static> PacketEncoder<T> {
    pub fn new(protocol_info: Box<dyn ProtocolInfo<T>>) -> PacketEncoder<T> {
        PacketEncoder { protocol_info }
    }

    pub fn protocol_info(&self) -> &dyn ProtocolInfo<T> {
        &*self.protocol_info
    }

    // The protocol's codec writes a boxed packet, Java's `Packet<? super T>`.
    #[allow(clippy::borrowed_box)]
    pub fn encode(&self, packet: &Box<dyn Packet<T>>, output: &mut ByteBuf) {
        self.protocol_info.codec().encode(output, packet);
    }
}
