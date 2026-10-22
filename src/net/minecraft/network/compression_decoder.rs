//! net/minecraft/network/CompressionDecoder.java
//!
//! A Netty `ByteToMessageDecoder` the connection puts after the frame splitter once the server
//! sets compression up. The context it takes only allocates the output, so the port leaves it
//! out. Java's `Inflater` is zlib's, which `flate2` runs too. Netty's `DecoderException` is a
//! panic here.

use flate2::{Decompress, FlushDecompress};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::var_int::VarInt;

/// Inflates each frame: a VarInt with the packet's length, 0 for a packet sent as it is, then the
/// packet, deflated when the length isn't 0.
pub struct CompressionDecoder {
    inflater: Decompress,
    threshold: i32,
    validate_decompressed: bool,
}

impl CompressionDecoder {
    pub const MAXIMUM_COMPRESSED_LENGTH: i32 = 2097152;
    pub const MAXIMUM_UNCOMPRESSED_LENGTH: i32 = 8388608;

    pub fn new(threshold: i32, validate_decompressed: bool) -> CompressionDecoder {
        CompressionDecoder {
            inflater: Decompress::new(true),
            threshold,
            validate_decompressed,
        }
    }

    pub fn decode(&mut self, input: &mut ByteBuf, out: &mut Vec<ByteBuf>) {
        let uncompressed_length = VarInt::read(input);
        if uncompressed_length == 0 {
            let readable = input.readable_bytes();
            out.push(input.read_bytes(readable));
        } else {
            if self.validate_decompressed {
                assert!(
                    uncompressed_length >= self.threshold,
                    "Badly compressed packet - size of {uncompressed_length} is below server \
                     threshold of {}",
                    self.threshold
                );
                assert!(
                    uncompressed_length <= 8388608,
                    "Badly compressed packet - size of {uncompressed_length} is larger than \
                     protocol maximum of 8388608"
                );
            }

            let compressed = CompressionDecoder::setup_inflater_input(input);
            let output = self.inflate(&compressed, uncompressed_length);
            self.inflater.reset(true);
            out.push(output);
        }
    }

    /// The rest of the frame, which the inflater reads from.
    fn setup_inflater_input(input: &mut ByteBuf) -> Vec<u8> {
        let length = input.readable_bytes();
        let mut bytes = vec![0; length as usize];
        input.read_bytes_array(&mut bytes);
        bytes.into_iter().map(|byte| byte as u8).collect()
    }

    /// Inflates into a buffer of the declared length, which has to come out full: Java's
    /// inflater stops there too, so a packet inflating to more is cut at the declared length.
    fn inflate(&mut self, input: &[u8], uncompressed_length: i32) -> ByteBuf {
        let mut output = vec![0u8; uncompressed_length.max(0) as usize];
        let before = self.inflater.total_out();
        // A malformed stream is a `DataFormatException` in Java, which fails the frame the same.
        if let Err(error) = self
            .inflater
            .decompress(input, &mut output, FlushDecompress::None)
        {
            panic!("java.util.zip.DataFormatException: {error}");
        }
        let actual_uncompressed_length = (self.inflater.total_out() - before) as i32;
        assert!(
            actual_uncompressed_length == uncompressed_length,
            "Badly compressed packet - actual length of uncompressed payload \
             {actual_uncompressed_length} is does not match declared size {uncompressed_length}"
        );

        let bytes = output.into_iter().map(|byte| byte as i8).collect();
        ByteBuf::from_array(bytes, i32::MAX)
    }

    pub fn set_threshold(&mut self, threshold: i32, validate_decompressed: bool) {
        self.threshold = threshold;
        self.validate_decompressed = validate_decompressed;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    use super::CompressionDecoder;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::network::var_int::VarInt;

    fn frame(length: i32, body: &[u8]) -> crate::io::netty::buffer::byte_buf::ByteBuf {
        let mut buffer = unpooled::buffer();
        VarInt::write(&mut buffer, length);
        buffer.write_bytes_array(&body.iter().map(|&byte| byte as i8).collect::<Vec<_>>());
        buffer
    }

    fn deflated(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn passes_uncompressed_frames_through() {
        let mut decoder = CompressionDecoder::new(256, true);
        let mut out = Vec::new();
        decoder.decode(&mut frame(0, &[1, 2, 3]), &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].readable_bytes(), 3);
    }

    #[test]
    fn inflates_one_frame_after_another() {
        let mut decoder = CompressionDecoder::new(256, true);
        let mut out = Vec::new();
        for length in [300, 4000, 300] {
            let body: Vec<u8> = (0..length).map(|i| (i % 7) as u8).collect();
            decoder.decode(&mut frame(length, &deflated(&body)), &mut out);
        }
        assert_eq!(
            out.iter()
                .map(|frame| frame.readable_bytes())
                .collect::<Vec<_>>(),
            [300, 4000, 300]
        );
        assert_eq!(out[1].clone().read_byte(), 0);
    }

    #[test]
    #[should_panic(expected = "does not match declared size")]
    fn refuses_a_frame_shorter_than_declared() {
        let mut decoder = CompressionDecoder::new(256, true);
        decoder.decode(&mut frame(301, &deflated(&[0; 300])), &mut Vec::new());
    }

    #[test]
    #[should_panic(expected = "below server threshold")]
    fn refuses_a_compressed_frame_under_the_threshold() {
        let mut decoder = CompressionDecoder::new(256, true);
        decoder.decode(&mut frame(100, &deflated(&[0; 100])), &mut Vec::new());
    }
}
