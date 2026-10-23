//! net/minecraft/client/model/geom/builders/UVPair.java
//!
//! Only the packing a baked quad keeps its texture coordinates in; the record itself comes with
//! entity models.

pub struct UVPair;

impl UVPair {
    /// `u`'s bits high and `v`'s low, each through `Float.floatToIntBits`, so every NaN packs
    /// as the canonical one.
    pub fn pack(u: f32, v: f32) -> i64 {
        let high = i64::from(float_to_int_bits(u)) & 0xFFFF_FFFF;
        let low = i64::from(float_to_int_bits(v)) & 0xFFFF_FFFF;
        high << 32 | low
    }

    pub fn unpack_u(packed_uv: i64) -> f32 {
        f32::from_bits((packed_uv >> 32) as u32)
    }

    pub fn unpack_v(packed_uv: i64) -> f32 {
        f32::from_bits(packed_uv as u32)
    }
}

fn float_to_int_bits(value: f32) -> i32 {
    crate::java::lang::float::Float::float_to_int_bits(value)
}
