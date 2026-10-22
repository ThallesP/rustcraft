//! net/minecraft/network/VarLong.java

use crate::io::netty::buffer::byte_buf::ByteBuf;

/// A long in 1 to 10 bytes, 7 bits at a time from the lowest, like `VarInt`.
pub struct VarLong;

impl VarLong {
    const MAX_VARLONG_SIZE: i32 = 10;
    const DATA_BITS_MASK: i32 = 127;
    const CONTINUATION_BIT_MASK: i32 = 128;
    const DATA_BITS_PER_BYTE: i32 = 7;

    pub fn get_byte_size(value: i64) -> i32 {
        for i in 1..VarLong::MAX_VARLONG_SIZE {
            if (value & (-1i64 << (i * VarLong::DATA_BITS_PER_BYTE))) == 0 {
                return i;
            }
        }

        VarLong::MAX_VARLONG_SIZE
    }

    pub fn has_continuation_bit(r#in: i8) -> bool {
        (i32::from(r#in) & VarLong::CONTINUATION_BIT_MASK) == VarLong::CONTINUATION_BIT_MASK
    }

    /// Reads until a byte without the continuation bit. A tenth byte's high bits fall off the
    /// long, and an eleventh byte throws after it's read.
    pub fn read(input: &mut ByteBuf) -> i64 {
        let mut out = 0i64;
        let mut bytes = 0;

        loop {
            let r#in = input.read_byte();
            // Java masks the shift to 6 bits: an eleventh byte's 70 is 6, before it throws.
            out |= i64::from(i32::from(r#in) & VarLong::DATA_BITS_MASK)
                .wrapping_shl((bytes * VarLong::DATA_BITS_PER_BYTE) as u32);
            bytes += 1;
            assert!(bytes <= VarLong::MAX_VARLONG_SIZE, "VarLong too big");
            if !VarLong::has_continuation_bit(r#in) {
                break;
            }
        }

        out
    }

    pub fn write(output: &mut ByteBuf, mut value: i64) -> &mut ByteBuf {
        while (value & -128) != 0 {
            output.write_byte(
                (value & i64::from(VarLong::DATA_BITS_MASK)) as i32
                    | VarLong::CONTINUATION_BIT_MASK,
            );
            value = ((value as u64) >> 7) as i64;
        }

        output.write_byte(value as i32);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::VarLong;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.VarLong");
        golden.check("getByteSize(J)I", VarLong::get_byte_size);
        golden.check("hasContinuationBit(B)Z", VarLong::has_continuation_bit);
        golden.check_mut("read(Lio/netty/buffer/ByteBuf;)J", |input: &mut ByteBuf| {
            VarLong::read(input)
        });
        golden.check_mut(
            "write(Lio/netty/buffer/ByteBuf;J)Lio/netty/buffer/ByteBuf;",
            |output: &mut ByteBuf, value: &mut i64| VarLong::write(output, *value).clone(),
        );
        golden.finish();
    }
}
