//! net/minecraft/network/Utf8String.java
//!
//! Java throws Netty's `DecoderException` and `EncoderException`, which are panics here.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::byte_buf_util::ByteBufUtil;
use crate::io::netty::buffer::unpooled;
use crate::net::minecraft::network::var_int::VarInt;

/// A string as its UTF-8 size in a VarInt, then its UTF-8. Limits are in UTF-16 units, like Java's
/// lengths, and the size may be up to three bytes a unit. That triple wraps for limits over a third
/// of the int range, as in Java, and then every string is too big.
pub struct Utf8String;

impl Utf8String {
    pub fn read(input: &mut ByteBuf, max_length: i32) -> String {
        let max_encoded_length = ByteBufUtil::utf8_max_bytes(max_length);
        let buffer_length = VarInt::read(input);
        assert!(
            buffer_length <= max_encoded_length,
            "The received encoded string buffer length is longer than maximum allowed ({buffer_length} > {max_encoded_length})"
        );
        assert!(
            buffer_length >= 0,
            "The received encoded string buffer length is less than zero! Weird string!"
        );
        let available_bytes = input.readable_bytes();
        assert!(
            buffer_length <= available_bytes,
            "Not enough bytes in buffer, expected {buffer_length}, but got {available_bytes}"
        );
        let result = input.to_string_utf8(input.reader_index(), buffer_length);
        input.set_reader_index(input.reader_index() + buffer_length);
        let length = result.encode_utf16().count() as i32;
        assert!(
            length <= max_length,
            "The received string length is longer than maximum allowed ({length} > {max_length})"
        );
        result
    }

    /// Checks the length in units first, then the size against `utf8MaxBytes(maxLength)`.
    pub fn write(output: &mut ByteBuf, value: &str, max_length: i32) {
        let length = value.encode_utf16().count() as i32;
        assert!(
            length <= max_length,
            "String too big (was {length} characters, max {max_length})"
        );

        let max_encoded_value_length = ByteBufUtil::utf8_max_bytes_seq(value);
        // output.alloc().buffer(maxEncodedValueLength)
        let mut tmp = unpooled::buffer_initial_capacity(max_encoded_value_length);

        let bytes_written = ByteBufUtil::write_utf8(&mut tmp, value);
        let max_allowed_encoded_length = ByteBufUtil::utf8_max_bytes(max_length);
        assert!(
            bytes_written <= max_allowed_encoded_length,
            "String too big (was {bytes_written} bytes encoded, max {max_allowed_encoded_length})"
        );

        VarInt::write(output, bytes_written);
        output.write_bytes_buf(&mut tmp);
    }
}

#[cfg(test)]
mod tests {
    use super::Utf8String;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.Utf8String");
        golden.check_mut(
            "read(Lio/netty/buffer/ByteBuf;I)Ljava/lang/String;",
            |input: &mut ByteBuf, max_length: &mut i32| Utf8String::read(input, *max_length),
        );
        golden.check_mut(
            "write(Lio/netty/buffer/ByteBuf;Ljava/lang/CharSequence;I)V",
            |output: &mut ByteBuf, value: &mut String, max_length: &mut i32| {
                Utf8String::write(output, value, *max_length)
            },
        );
        golden.finish();
    }
}
