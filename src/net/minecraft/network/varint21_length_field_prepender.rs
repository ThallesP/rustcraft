//! net/minecraft/network/Varint21LengthFieldPrepender.java
//!
//! A Netty `MessageToByteEncoder`: Netty calls `encode` with each frame and the buffer to send. The
//! context it takes goes unused, so the port leaves it out. Java throws Netty's
//! `EncoderException`, which is a panic here.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::var_int::VarInt;

/// Puts each frame's length in front of it, in a VarInt of at most 3 bytes.
pub struct Varint21LengthFieldPrepender;

impl Varint21LengthFieldPrepender {
    pub const MAX_VARINT21_BYTES: i32 = 3;

    /// Writes the message's readable bytes after their length, without reading them: Netty
    /// releases the message after.
    pub fn encode(&self, msg: &ByteBuf, out: &mut ByteBuf) {
        let body_length = msg.readable_bytes();
        let header_length = VarInt::get_byte_size(body_length);
        // Java's message says 8, whatever the size.
        assert!(
            header_length <= 3,
            "Packet too large: size {body_length} is over 8"
        );

        out.ensure_writable(header_length + body_length);
        VarInt::write(out, body_length);
        out.write_bytes_buf_range(msg, msg.reader_index(), body_length);
    }
}

#[cfg(test)]
mod tests {
    use super::Varint21LengthFieldPrepender;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.Varint21LengthFieldPrepender");
        golden.field(
            "MAX_VARINT21_BYTES",
            Varint21LengthFieldPrepender::MAX_VARINT21_BYTES,
        );
        golden.check_mut(
            "encode(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Lio/netty/buffer/ByteBuf;)V",
            |msg: &mut ByteBuf, out: &mut ByteBuf| Varint21LengthFieldPrepender.encode(msg, out),
        );
        golden.finish();
    }
}
