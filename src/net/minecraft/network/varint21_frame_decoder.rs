//! net/minecraft/network/Varint21FrameDecoder.java
//!
//! Not ported yet: the `BandwidthDebugMonitor` the constructor takes, which counts the bytes of
//! each frame for the debug screen's bandwidth chart, and changes nothing else.
//!
//! A Netty `ByteToMessageDecoder`: Netty calls `decode` with what has arrived so far, again while
//! it takes frames out, and passes each frame on. The context it takes goes unused, so the port
//! leaves it out, and `handlerRemoved0`, which releases the helper buffer, is Rust's drop. Java
//! throws Netty's `CorruptedFrameException`, which is a panic here.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::unpooled;
use crate::net::minecraft::network::var_int::VarInt;

/// Splits what arrives into frames, each a VarInt length of at most 3 bytes and that many bytes.
pub struct Varint21FrameDecoder {
    helper_buf: ByteBuf,
}

impl Default for Varint21FrameDecoder {
    fn default() -> Varint21FrameDecoder {
        Varint21FrameDecoder::new()
    }
}

impl Varint21FrameDecoder {
    const MAX_VARINT21_BYTES: i32 = 3;

    /// Java's helper is `Unpooled.directBuffer(3)`, which only differs from a heap buffer in where
    /// its bytes live.
    pub fn new() -> Varint21FrameDecoder {
        Varint21FrameDecoder {
            helper_buf: unpooled::buffer_initial_capacity(3),
        }
    }

    /// Copies the length's bytes, and says whether all of them have arrived.
    fn copy_varint(input: &mut ByteBuf, output: &mut ByteBuf) -> bool {
        for _ in 0..Varint21FrameDecoder::MAX_VARINT21_BYTES {
            if !input.is_readable() {
                return false;
            }

            let b = input.read_byte();
            output.write_byte(i32::from(b));
            if !VarInt::has_continuation_bit(b) {
                return true;
            }
        }

        panic!("length wider than 21-bit");
    }

    /// Takes the next frame out of `input` into `out`, or leaves `input` where it was until all of
    /// it has arrived. Marks the reader index either way.
    pub fn decode(&mut self, input: &mut ByteBuf, out: &mut Vec<ByteBuf>) {
        input.mark_reader_index();
        self.helper_buf.clear();
        if !Varint21FrameDecoder::copy_varint(input, &mut self.helper_buf) {
            input.reset_reader_index();
        } else {
            let length = VarInt::read(&mut self.helper_buf);
            assert!(length != 0, "Frame length cannot be zero");

            if input.readable_bytes() < length {
                input.reset_reader_index();
            } else {
                out.push(input.read_bytes(length));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Varint21FrameDecoder;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;

    const DECODE: &str = "decode(Lio/netty/channel/ChannelHandlerContext;Lio/netty/buffer/ByteBuf;Ljava/util/List;)V";

    // Like Netty, the cases call one decoder again while bytes are left and the last call took a
    // frame out.
    fn frames(decoder: &mut Varint21FrameDecoder, input: &mut ByteBuf, out: &mut Vec<ByteBuf>) {
        loop {
            let found = out.len();
            decoder.decode(input, out);
            if !input.is_readable() || out.len() == found {
                return;
            }
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.Varint21FrameDecoder");
        golden.pending(
            "<init>(Lnet/minecraft/network/BandwidthDebugMonitor;)V",
            "waits for BandwidthDebugMonitor",
        );
        golden.check_mut(DECODE, |input: &mut ByteBuf| {
            let mut out = Vec::new();
            frames(&mut Varint21FrameDecoder::new(), input, &mut out);
            out
        });
        // The rest of the stream arrives in chunks, each written to the buffer before the decoder
        // is asked again.
        golden.check_mut(
            &format!("{DECODE}[in chunks]"),
            |input: &mut ByteBuf, chunks: &mut Vec<Vec<i8>>| {
                let mut decoder = Varint21FrameDecoder::new();
                let mut out = Vec::new();
                frames(&mut decoder, input, &mut out);
                for chunk in chunks.iter() {
                    input.write_bytes_array(chunk);
                    frames(&mut decoder, input, &mut out);
                }
                out
            },
        );
        golden.finish();
    }
}
