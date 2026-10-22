//! net/minecraft/network/VarInt.java

use crate::io::netty::buffer::byte_buf::ByteBuf;

/// An int in 1 to 5 bytes, 7 bits at a time from the lowest, each byte but the last with its top
/// bit set. Negative ints take all 5.
pub struct VarInt;

impl VarInt {
    pub const MAX_VARINT_SIZE: i32 = 5;
    const DATA_BITS_MASK: i32 = 127;
    const CONTINUATION_BIT_MASK: i32 = 128;
    const DATA_BITS_PER_BYTE: i32 = 7;

    pub fn get_byte_size(value: i32) -> i32 {
        for i in 1..5 {
            if (value & (-1 << (i * VarInt::DATA_BITS_PER_BYTE))) == 0 {
                return i;
            }
        }

        5
    }

    pub fn has_continuation_bit(r#in: i8) -> bool {
        (i32::from(r#in) & VarInt::CONTINUATION_BIT_MASK) == VarInt::CONTINUATION_BIT_MASK
    }

    /// Reads until a byte without the continuation bit. A fifth byte's high bits fall off the
    /// int, and a sixth byte throws after it's read.
    pub fn read(input: &mut ByteBuf) -> i32 {
        let mut out = 0;
        let mut bytes = 0;

        loop {
            let r#in = input.read_byte();
            // Java masks the shift to 5 bits: a sixth byte's 35 is 3, before it throws.
            out |= (i32::from(r#in) & VarInt::DATA_BITS_MASK)
                .wrapping_shl((bytes * VarInt::DATA_BITS_PER_BYTE) as u32);
            bytes += 1;
            assert!(bytes <= 5, "VarInt too big");
            if !VarInt::has_continuation_bit(r#in) {
                break;
            }
        }

        out
    }

    pub fn write(output: &mut ByteBuf, mut value: i32) -> &mut ByteBuf {
        while (value & -128) != 0 {
            output.write_byte(value & VarInt::DATA_BITS_MASK | VarInt::CONTINUATION_BIT_MASK);
            value = ((value as u32) >> 7) as i32;
        }

        output.write_byte(value);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.VarInt");
        golden.field("MAX_VARINT_SIZE", VarInt::MAX_VARINT_SIZE);
        golden.check("getByteSize(I)I", VarInt::get_byte_size);
        golden.check("hasContinuationBit(B)Z", VarInt::has_continuation_bit);
        golden.check_mut("read(Lio/netty/buffer/ByteBuf;)I", |input: &mut ByteBuf| {
            VarInt::read(input)
        });
        golden.check_mut(
            "write(Lio/netty/buffer/ByteBuf;I)Lio/netty/buffer/ByteBuf;",
            |output: &mut ByteBuf, value: &mut i32| VarInt::write(output, *value).clone(),
        );
        golden.finish();
    }
}
