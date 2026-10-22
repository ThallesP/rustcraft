//! net/minecraft/network/PacketDecoder.java
//!
//! Not ported yet: the JFR profiler's and the debug log's record of each packet. A Netty
//! `ByteToMessageDecoder` the connection puts after the frame splitter, and decompression when
//! there's any. What `ProtocolSwapHandler.handleInboundTerminalPacket` does after a terminal
//! packet (stop reading, until the next protocol's decoder is set up) is the connection's, which
//! holds the pipeline. Java's `IOException` is a panic here.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol_info::ProtocolInfo;

/// Reads each frame as one packet of a protocol, for the listener `T`.
pub struct PacketDecoder<T: ?Sized> {
    protocol_info: Box<dyn ProtocolInfo<T>>,
}

impl<T: ?Sized + 'static> PacketDecoder<T> {
    pub fn new(protocol_info: Box<dyn ProtocolInfo<T>>) -> PacketDecoder<T> {
        PacketDecoder { protocol_info }
    }

    pub fn protocol_info(&self) -> &dyn ProtocolInfo<T> {
        &*self.protocol_info
    }

    /// The frame's packet, which has to take all of its bytes.
    pub fn decode(&self, input: &mut ByteBuf) -> Box<dyn Packet<T>> {
        let packet = self.protocol_info.codec().decode(input);
        let packet_id = packet.r#type();
        if input.readable_bytes() > 0 {
            panic!(
                "Packet {}/{packet_id} ({packet:?}) was larger than I expected, found {} bytes \
                 extra whilst reading packet {packet_id}",
                self.protocol_info.id().id(),
                input.readable_bytes()
            );
        }

        packet
    }
}
