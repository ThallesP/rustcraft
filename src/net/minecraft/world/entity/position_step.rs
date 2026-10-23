//! net/minecraft/world/entity/PositionStep.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::world::phys::vec3::Vec3;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, PositionStep> = LazyLock::new(|| {
    Box::new(stream_codec::composite2(
        Vec3::STREAM_CODEC,
        PositionStep::position,
        ByteBufCodecs::VAR_INT,
        PositionStep::tick_offset,
        PositionStep::new,
    ))
});

/// A step of an entity's path: where it gets to, and how many ticks after the step before.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionStep {
    position: Vec3,
    tick_offset: i32,
}

impl PositionStep {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, PositionStep> = &STREAM_CODEC;

    pub fn new(position: Vec3, tick_offset: i32) -> PositionStep {
        PositionStep {
            position,
            tick_offset,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn tick_offset(&self) -> i32 {
        self.tick_offset
    }

    pub fn add_delta(&self, delta: &Vec3) -> PositionStep {
        PositionStep::new(self.position.add(delta), self.tick_offset)
    }

    /// A record's: the position's hash, then the offset, added to 31 times the one before.
    pub fn hash_code(&self) -> i32 {
        self.position
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.tick_offset)
    }
}

/// A record's: `PositionStep[position=(0.5, -60.0, 0.5), tickOffset=3]`.
impl fmt::Display for PositionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PositionStep[position={}, tickOffset={}]",
            self.position, self.tick_offset
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PositionStep;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::world::phys::vec3::Vec3;

    encode_fields!(PositionStep {
        position: Vec3,
        tick_offset: i32,
    });

    type This = PositionStep;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.PositionStep");
        golden.check_codec("STREAM_CODEC", || PositionStep::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;I)V",
            PositionStep::new,
        );
        golden.check_method(
            "addDelta(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/entity/PositionStep;",
            |this: &mut This, delta: Vec3| this.add_delta(&delta),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
        golden.check_method(
            "position()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This| this.position(),
        );
        golden.check_method("tickOffset()I", |this: &mut This| this.tick_offset());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.finish();
    }
}
