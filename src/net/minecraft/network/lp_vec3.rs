//! net/minecraft/network/LpVec3.java

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::math::Math;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// A movement in as few bytes as it takes: a zero byte for one too small, otherwise 6 bytes
/// holding each component in 15 bits as a fraction of a whole number of blocks, the scale, which
/// takes the 2 lowest bits and, past 3, a VarInt of the rest.
pub struct LpVec3;

impl LpVec3 {
    const DATA_BITS_MASK: i32 = 32767;
    const MAX_QUANTIZED_VALUE: f64 = 32766.0;
    const SCALE_BITS: i32 = 2;
    const SCALE_BITS_MASK: i32 = 3;
    const CONTINUATION_FLAG: i32 = 4;
    const X_OFFSET: i32 = 3;
    const Y_OFFSET: i32 = 18;
    const Z_OFFSET: i32 = 33;
    pub const ABS_MAX_VALUE: f64 = 1.7179869183E10;
    pub const ABS_MIN_VALUE: f64 = 3.051944088384301E-5;

    pub fn has_continuation_bit(r#in: i32) -> bool {
        (r#in & LpVec3::CONTINUATION_FLAG) == LpVec3::CONTINUATION_FLAG
    }

    /// The VarInt of a scale past 3 holds its bits from the third up, as an unsigned int.
    pub fn read(input: &mut ByteBuf) -> Vec3 {
        let lowest = i32::from(input.read_unsigned_byte());
        if lowest == 0 {
            return Vec3::ZERO;
        }

        let middle = i32::from(input.read_unsigned_byte());
        let highest = input.read_unsigned_int();
        let buffer = (highest << 16) | i64::from(middle << 8) | i64::from(lowest);
        let mut scale = i64::from(lowest & LpVec3::SCALE_BITS_MASK);
        if LpVec3::has_continuation_bit(lowest) {
            scale |= (i64::from(VarInt::read(input)) & 4294967295) << LpVec3::SCALE_BITS;
        }

        Vec3::new(
            LpVec3::unpack(buffer >> LpVec3::X_OFFSET) * scale as f64,
            LpVec3::unpack(buffer >> LpVec3::Y_OFFSET) * scale as f64,
            LpVec3::unpack(buffer >> LpVec3::Z_OFFSET) * scale as f64,
        )
    }

    /// NaN components are written as 0, and the others clamped to `ABS_MAX_VALUE`.
    pub fn write(output: &mut ByteBuf, value: &Vec3) {
        let x = LpVec3::sanitize(value.x);
        let y = LpVec3::sanitize(value.y);
        let z = LpVec3::sanitize(value.z);
        let chessboard_length = Mth::abs_max_f64(x, Mth::abs_max_f64(y, z));
        if chessboard_length < LpVec3::ABS_MIN_VALUE {
            output.write_byte(0);
        } else {
            let scale = Mth::ceil_long(chessboard_length);
            let is_partial = (scale & i64::from(LpVec3::SCALE_BITS_MASK)) != scale;
            let markers = if is_partial {
                scale & i64::from(LpVec3::SCALE_BITS_MASK) | i64::from(LpVec3::CONTINUATION_FLAG)
            } else {
                scale
            };
            let xn = LpVec3::pack(x / scale as f64) << LpVec3::X_OFFSET;
            let yn = LpVec3::pack(y / scale as f64) << LpVec3::Y_OFFSET;
            let zn = LpVec3::pack(z / scale as f64) << LpVec3::Z_OFFSET;
            let buffer = markers | xn | yn | zn;
            output.write_byte(i32::from(buffer as i8));
            output.write_byte(i32::from((buffer >> 8) as i8));
            output.write_int((buffer >> 16) as i32);
            if is_partial {
                VarInt::write(output, (scale >> LpVec3::SCALE_BITS) as i32);
            }
        }
    }

    fn sanitize(value: f64) -> f64 {
        if value.is_nan() {
            0.0
        } else {
            Math::clamp_f64(value, -LpVec3::ABS_MAX_VALUE, LpVec3::ABS_MAX_VALUE)
        }
    }

    // A fraction from -1 to 1, in 0 to 32766.
    fn pack(value: f64) -> i64 {
        Math::round_f64((value * 0.5 + 0.5) * LpVec3::MAX_QUANTIZED_VALUE)
    }

    // 32767 reads as 32766, the most `pack` writes.
    fn unpack(value: i64) -> f64 {
        Math::min_f64(
            (value & i64::from(LpVec3::DATA_BITS_MASK)) as f64,
            LpVec3::MAX_QUANTIZED_VALUE,
        ) * 2.0
            / LpVec3::MAX_QUANTIZED_VALUE
            - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::LpVec3;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.LpVec3");
        golden.field("ABS_MAX_VALUE", LpVec3::ABS_MAX_VALUE);
        golden.field("ABS_MIN_VALUE", LpVec3::ABS_MIN_VALUE);
        golden.check("hasContinuationBit(I)Z", LpVec3::has_continuation_bit);
        golden.check_mut(
            "read(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/world/phys/Vec3;",
            |input: &mut ByteBuf| LpVec3::read(input),
        );
        golden.check_mut(
            "write(Lio/netty/buffer/ByteBuf;Lnet/minecraft/world/phys/Vec3;)V",
            |output: &mut ByteBuf, value: &mut Vec3| LpVec3::write(output, value),
        );
        golden.finish();
    }
}
