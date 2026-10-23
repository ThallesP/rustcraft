//! net/minecraft/core/Rotations.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs, and the record's `toString`,
//! which prints its floats with Java's `Float.toString`.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};

static STREAM_CODEC: StaticStreamCodec<ByteBuf, Rotations> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &Rotations| {
            output.write_float(value.x);
            output.write_float(value.y);
            output.write_float(value.z);
        },
        |input: &mut ByteBuf| {
            Rotations::new(input.read_float(), input.read_float(), input.read_float())
        },
    ))
});

/// Angles in degrees about each axis, an armor stand's parts'.
#[derive(Clone, Copy, Debug)]
pub struct Rotations {
    x: f32,
    y: f32,
    z: f32,
}

impl Rotations {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Rotations> = &STREAM_CODEC;

    /// Each angle modulo 360, and 0 for one that's infinite or NaN.
    pub fn new(x: f32, y: f32, z: f32) -> Rotations {
        let angle = |value: f32| {
            if value.is_finite() {
                value % 360.0
            } else {
                0.0
            }
        };
        Rotations {
            x: angle(x),
            y: angle(y),
            z: angle(z),
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }

    /// A record's: each angle's bits, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        [self.y, self.z]
            .into_iter()
            .fold(Float::hash_code(self.x), |hash, angle| {
                hash.wrapping_mul(31).wrapping_add(Float::hash_code(angle))
            })
    }
}

/// A record's: the angles compare by their bits, NaNs made one (`Float.compare`).
impl PartialEq for Rotations {
    fn eq(&self, other: &Rotations) -> bool {
        Float::float_to_int_bits(self.x) == Float::float_to_int_bits(other.x)
            && Float::float_to_int_bits(self.y) == Float::float_to_int_bits(other.y)
            && Float::float_to_int_bits(self.z) == Float::float_to_int_bits(other.z)
    }
}

#[cfg(test)]
mod tests {
    use super::Rotations;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(Rotations {
        x: f32,
        y: f32,
        z: f32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.Rotations");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending(
            "toString()Ljava/lang/String;",
            "waits for Java's Float.toString",
        );
        golden.check_codec("STREAM_CODEC", || Rotations::STREAM_CODEC);
        golden.check("<init>(FFF)V", Rotations::new);
        golden.check_method("x()F", |this: &mut Rotations| this.x());
        golden.check_method("y()F", |this: &mut Rotations| this.y());
        golden.check_method("z()F", |this: &mut Rotations| this.z());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Rotations, other: Rotations| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut Rotations| this.hash_code());
        golden.finish();
    }
}
