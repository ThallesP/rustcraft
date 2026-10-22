//! net/minecraft/world/level/block/Rotation.java
//!
//! Not ported yet: `CODEC` and `LEGACY_CODEC`, which wait for DataFixerUpper's codecs, and
//! `rotation()`, which waits for `OctahedralGroup`.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::util::util::Util;

/// A quarter turn of a block (or a structure) around the vertical axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    NONE,
    CLOCKWISE_90,
    CLOCKWISE_180,
    COUNTERCLOCKWISE_90,
}

// By index, wrapping around.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> Rotation + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        Rotation::get_index,
        &Rotation::VALUES,
        OutOfBoundsStrategy::WRAP,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, Rotation> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::id_mapper(
        Rotation::BY_ID.as_ref(),
        Rotation::get_index,
    ))
});

impl Rotation {
    /// Java's `IntFunction<Rotation>`.
    pub const BY_ID: &'static LazyLock<Box<dyn Fn(i32) -> Rotation + Send + Sync>> = &BY_ID;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Rotation> = &STREAM_CODEC;

    /// `values()`: the constants in declaration order, which is also their index and ordinal.
    pub const VALUES: [Rotation; 4] = [
        Rotation::NONE,
        Rotation::CLOCKWISE_90,
        Rotation::CLOCKWISE_180,
        Rotation::COUNTERCLOCKWISE_90,
    ];

    pub fn values() -> [Rotation; 4] {
        Rotation::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Rotation {
        match name {
            "NONE" => Rotation::NONE,
            "CLOCKWISE_90" => Rotation::CLOCKWISE_90,
            "CLOCKWISE_180" => Rotation::CLOCKWISE_180,
            "COUNTERCLOCKWISE_90" => Rotation::COUNTERCLOCKWISE_90,
            _ => panic!("IllegalArgumentException: no Rotation constant {name}"),
        }
    }

    /// This rotation followed by `rot`.
    pub fn get_rotated(&self, rot: Rotation) -> Rotation {
        match rot {
            Rotation::CLOCKWISE_90 => match self {
                Rotation::NONE => Rotation::CLOCKWISE_90,
                Rotation::CLOCKWISE_90 => Rotation::CLOCKWISE_180,
                Rotation::CLOCKWISE_180 => Rotation::COUNTERCLOCKWISE_90,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::NONE,
            },
            Rotation::CLOCKWISE_180 => match self {
                Rotation::NONE => Rotation::CLOCKWISE_180,
                Rotation::CLOCKWISE_90 => Rotation::COUNTERCLOCKWISE_90,
                Rotation::CLOCKWISE_180 => Rotation::NONE,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::CLOCKWISE_90,
            },
            Rotation::COUNTERCLOCKWISE_90 => match self {
                Rotation::NONE => Rotation::COUNTERCLOCKWISE_90,
                Rotation::CLOCKWISE_90 => Rotation::NONE,
                Rotation::CLOCKWISE_180 => Rotation::CLOCKWISE_90,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::CLOCKWISE_180,
            },
            Rotation::NONE => *self,
        }
    }

    pub fn rotate(&self, direction: Direction) -> Direction {
        if direction.get_axis() == Axis::Y {
            return direction;
        }

        match self {
            Rotation::CLOCKWISE_90 => direction.get_clock_wise(),
            Rotation::CLOCKWISE_180 => direction.get_opposite(),
            Rotation::COUNTERCLOCKWISE_90 => direction.get_counter_clock_wise(),
            Rotation::NONE => direction,
        }
    }

    /// Turns one of `steps` evenly spaced rotations (a sign's 16) by this rotation. Panics when
    /// `steps` is 0, where Java throws `ArithmeticException`, but `NONE` never divides.
    pub fn rotate_i32(&self, rotation: i32, steps: i32) -> i32 {
        match self {
            Rotation::CLOCKWISE_90 => rotation.wrapping_add(steps / 4).wrapping_rem(steps),
            Rotation::CLOCKWISE_180 => rotation.wrapping_add(steps / 2).wrapping_rem(steps),
            Rotation::COUNTERCLOCKWISE_90 => rotation
                .wrapping_add(steps.wrapping_mul(3) / 4)
                .wrapping_rem(steps),
            Rotation::NONE => rotation,
        }
    }

    pub fn get_random(random: &mut dyn RandomSource) -> Rotation {
        *Util::get_random(&Rotation::VALUES, random)
    }

    pub fn get_shuffled(random: &mut dyn RandomSource) -> Vec<Rotation> {
        Util::shuffled_copy(&Rotation::values(), random)
    }

    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Rotation::NONE => "none",
            Rotation::CLOCKWISE_90 => "clockwise_90",
            Rotation::CLOCKWISE_180 => "180",
            Rotation::COUNTERCLOCKWISE_90 => "counterclockwise_90",
        }
    }

    // The index each constant is made with, which is its ordinal.
    fn get_index(&self) -> i32 {
        *self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::Rotation;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::util::random_source::RandomSource;

    encode_names!(Rotation {
        NONE,
        CLOCKWISE_90,
        CLOCKWISE_180,
        COUNTERCLOCKWISE_90
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.Rotation");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending("LEGACY_CODEC", "waits for DataFixerUpper's codecs");
        golden.check_codec("STREAM_CODEC", || Rotation::STREAM_CODEC);
        golden.check_function("BY_ID", || Rotation::BY_ID.as_ref());
        golden.pending(
            "rotation()Lcom/mojang/math/OctahedralGroup;",
            "waits for OctahedralGroup",
        );
        for rotation in Rotation::VALUES {
            golden.field(&format!("{rotation:?}"), rotation);
        }
        golden.check(
            "values()[Lnet/minecraft/world/level/block/Rotation;",
            Rotation::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/level/block/Rotation;",
            |name: String| Rotation::value_of(&name),
        );
        golden.check_method(
            "getRotated(Lnet/minecraft/world/level/block/Rotation;)Lnet/minecraft/world/level/block/Rotation;",
            |this: &mut Rotation, rot| this.get_rotated(rot),
        );
        golden.check_method(
            "rotate(Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;",
            |this: &mut Rotation, direction: Direction| this.rotate(direction),
        );
        golden.check_method("rotate(II)I", |this: &mut Rotation, rotation, steps| {
            this.rotate_i32(rotation, steps)
        });
        golden.check_random(
            "getRandom(Lnet/minecraft/util/RandomSource;)Lnet/minecraft/world/level/block/Rotation;",
            |random: &mut dyn RandomSource| Rotation::get_random(random),
        );
        golden.check_random(
            "getShuffled(Lnet/minecraft/util/RandomSource;)Ljava/util/List;",
            |random: &mut dyn RandomSource| Rotation::get_shuffled(random),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Rotation| this.get_serialized_name().to_owned(),
        );
        golden.finish();
    }
}
