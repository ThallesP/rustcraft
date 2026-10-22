//! net/minecraft/world/entity/HumanoidArm.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs, and `caption()`, which waits
//! for text components. `StringRepresentable`'s `getSerializedName` is its own, like
//! `Rotation`'s.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};

/// A player's main hand, which the client sends in its `ClientInformation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HumanoidArm {
    LEFT,
    RIGHT,
}

// By id, and the left arm for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> HumanoidArm + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        HumanoidArm::id,
        &HumanoidArm::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, HumanoidArm> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), HumanoidArm::id)));

impl HumanoidArm {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, HumanoidArm> = &STREAM_CODEC;

    pub const VALUES: [HumanoidArm; 2] = [HumanoidArm::LEFT, HumanoidArm::RIGHT];

    pub fn values() -> [HumanoidArm; 2] {
        HumanoidArm::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> HumanoidArm {
        match name {
            "LEFT" => HumanoidArm::LEFT,
            "RIGHT" => HumanoidArm::RIGHT,
            _ => panic!("IllegalArgumentException: no HumanoidArm constant {name}"),
        }
    }

    pub fn get_opposite(&self) -> HumanoidArm {
        match self {
            HumanoidArm::LEFT => HumanoidArm::RIGHT,
            HumanoidArm::RIGHT => HumanoidArm::LEFT,
        }
    }

    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            HumanoidArm::LEFT => "left",
            HumanoidArm::RIGHT => "right",
        }
    }

    // The id each constant is made with.
    fn id(&self) -> i32 {
        match self {
            HumanoidArm::LEFT => 0,
            HumanoidArm::RIGHT => 1,
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for HumanoidArm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::HumanoidArm;
    use crate::golden::{Golden, encode_names};

    encode_names!(HumanoidArm { LEFT, RIGHT });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.HumanoidArm");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending(
            "caption()Lnet/minecraft/network/chat/Component;",
            "waits for text components",
        );
        golden.check_codec("STREAM_CODEC", || HumanoidArm::STREAM_CODEC);
        for arm in HumanoidArm::VALUES {
            golden.field(&format!("{arm:?}"), arm);
        }
        golden.check(
            "values()[Lnet/minecraft/world/entity/HumanoidArm;",
            HumanoidArm::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/entity/HumanoidArm;",
            |name: String| HumanoidArm::value_of(&name),
        );
        golden.check_method(
            "getOpposite()Lnet/minecraft/world/entity/HumanoidArm;",
            |this: &mut HumanoidArm| this.get_opposite(),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut HumanoidArm| this.get_serialized_name().to_owned(),
        );
        golden.finish();
    }
}
