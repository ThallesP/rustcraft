//! net/minecraft/network/CompressionEncoder.java
//!
//! A Netty `MessageToByteEncoder` the connection puts in front of the length prepender once the
//! server sets compression up. Java's `Deflater` is zlib's at its default level. `flate2`'s
//! deflate at the same level can choose other bytes for the same packet, which nothing compares:
//! the server inflates them back to the same packet, and captures record packets inflated.

use flate2::{Compress, Compression, FlushCompress, Status};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::var_int::VarInt;

/// Deflates each packet at least as long as the threshold, after its length, and sends shorter
/// ones as they are after a length of 0.
pub struct CompressionEncoder {
    encode_buf: Vec<u8>,
    deflater: Compress,
    threshold: i32,
}

impl CompressionEncoder {
    pub fn new(threshold: i32) -> CompressionEncoder {
        CompressionEncoder {
            encode_buf: vec![0; 8192],
            deflater: Compress::new(Compression::default(), true),
            threshold,
        }
    }

    pub fn encode(&mut self, uncompressed: &mut ByteBuf, out: &mut ByteBuf) {
        let uncompressed_length = uncompressed.readable_bytes();
        assert!(
            uncompressed_length <= 8388608,
            "Packet too big (is {uncompressed_length}, should be less than 8388608)"
        );

        if uncompressed_length < self.threshold {
            VarInt::write(out, 0);
            out.write_bytes_buf(uncompressed);
        } else {
            let mut input = vec![0; uncompressed_length as usize];
            uncompressed.read_bytes_array(&mut input);
            VarInt::write(out, input.len() as i32);
            let input: Vec<u8> = input.into_iter().map(|byte| byte as u8).collect();
            let mut read = 0;

            loop {
                let (before_in, before_out) = (self.deflater.total_in(), self.deflater.total_out());
                let status = self
                    .deflater
                    .compress(&input[read..], &mut self.encode_buf, FlushCompress::Finish)
                    .expect("zlib's deflate doesn't fail on a buffer");
                read += (self.deflater.total_in() - before_in) as usize;
                let written = (self.deflater.total_out() - before_out) as usize;
                let bytes: Vec<i8> = self.encode_buf[..written]
                    .iter()
                    .map(|&byte| byte as i8)
                    .collect();
                out.write_bytes_array(&bytes);
                if status == Status::StreamEnd {
                    break;
                }
            }

            self.deflater.reset();
        }
    }

    pub fn get_threshold(&self) -> i32 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: i32) {
        self.threshold = threshold;
    }
}

#[cfg(test)]
mod tests {
    use super::CompressionEncoder;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::network::compression_decoder::CompressionDecoder;

    #[test]
    fn round_trips_through_the_decoder() {
        let mut encoder = CompressionEncoder::new(256);
        let mut decoder = CompressionDecoder::new(256, true);
        for length in [0, 1, 255, 256, 20000] {
            let mut packet = unpooled::buffer();
            for i in 0..length {
                packet.write_byte(i * 31 % 251);
            }
            let mut frame = unpooled::buffer();
            encoder.encode(&mut packet, &mut frame);
            assert_eq!(packet.readable_bytes(), 0);

            let mut out = Vec::new();
            decoder.decode(&mut frame, &mut out);
            let mut inflated = out.pop().unwrap();
            assert_eq!(inflated.readable_bytes(), length);
            for i in 0..length {
                assert_eq!(i32::from(inflated.read_unsigned_byte()), i * 31 % 251);
            }
        }
    }
}
