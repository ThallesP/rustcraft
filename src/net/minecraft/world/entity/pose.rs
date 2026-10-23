//! net/minecraft/world/entity/Pose.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::string_representable::StringRepresentable;

/// How an entity holds itself, which decides its size.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pose {
    STANDING,
    FALL_FLYING,
    SLEEPING,
    SWIMMING,
    SPIN_ATTACK,
    CROUCHING,
    LONG_JUMPING,
    DYING,
    CROAKING,
    USING_TONGUE,
    SITTING,
    ROARING,
    SNIFFING,
    EMERGING,
    DIGGING,
    SLIDING,
    SHOOTING,
    INHALING,
}

// Each constant's id and serialized name, in their order.
const DATA: [(i32, &str); 18] = [
    (0, "standing"),
    (1, "fall_flying"),
    (2, "sleeping"),
    (3, "swimming"),
    (4, "spin_attack"),
    (5, "crouching"),
    (6, "long_jumping"),
    (7, "dying"),
    (8, "croaking"),
    (9, "using_tongue"),
    (10, "sitting"),
    (11, "roaring"),
    (12, "sniffing"),
    (13, "emerging"),
    (14, "digging"),
    (15, "sliding"),
    (16, "shooting"),
    (17, "inhaling"),
];

// By id, the first for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> Pose + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        Pose::id,
        &Pose::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, Pose> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), Pose::id)));

impl Pose {
    pub const VALUES: [Pose; 18] = [
        Pose::STANDING,
        Pose::FALL_FLYING,
        Pose::SLEEPING,
        Pose::SWIMMING,
        Pose::SPIN_ATTACK,
        Pose::CROUCHING,
        Pose::LONG_JUMPING,
        Pose::DYING,
        Pose::CROAKING,
        Pose::USING_TONGUE,
        Pose::SITTING,
        Pose::ROARING,
        Pose::SNIFFING,
        Pose::EMERGING,
        Pose::DIGGING,
        Pose::SLIDING,
        Pose::SHOOTING,
        Pose::INHALING,
    ];
    pub const BY_ID: &'static LazyLock<Box<dyn Fn(i32) -> Pose + Send + Sync>> = &BY_ID;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Pose> = &STREAM_CODEC;

    pub fn values() -> [Pose; 18] {
        Pose::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Pose {
        Pose::VALUES
            .into_iter()
            .find(|pose| pose.name() == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Pose constant {name}"))
    }

    pub fn id(&self) -> i32 {
        DATA[*self as usize].0
    }

    /// The constant's name, `name()`.
    pub fn name(&self) -> &'static str {
        match self {
            Pose::STANDING => "STANDING",
            Pose::FALL_FLYING => "FALL_FLYING",
            Pose::SLEEPING => "SLEEPING",
            Pose::SWIMMING => "SWIMMING",
            Pose::SPIN_ATTACK => "SPIN_ATTACK",
            Pose::CROUCHING => "CROUCHING",
            Pose::LONG_JUMPING => "LONG_JUMPING",
            Pose::DYING => "DYING",
            Pose::CROAKING => "CROAKING",
            Pose::USING_TONGUE => "USING_TONGUE",
            Pose::SITTING => "SITTING",
            Pose::ROARING => "ROARING",
            Pose::SNIFFING => "SNIFFING",
            Pose::EMERGING => "EMERGING",
            Pose::DIGGING => "DIGGING",
            Pose::SLIDING => "SLIDING",
            Pose::SHOOTING => "SHOOTING",
            Pose::INHALING => "INHALING",
        }
    }
}

impl StringRepresentable for Pose {
    fn get_serialized_name(&self) -> &'static str {
        DATA[*self as usize].1
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Pose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::Pose;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::util::string_representable::StringRepresentable;

    encode_names!(Pose {
        STANDING,
        FALL_FLYING,
        SLEEPING,
        SWIMMING,
        SPIN_ATTACK,
        CROUCHING,
        LONG_JUMPING,
        DYING,
        CROAKING,
        USING_TONGUE,
        SITTING,
        ROARING,
        SNIFFING,
        EMERGING,
        DIGGING,
        SLIDING,
        SHOOTING,
        INHALING
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.Pose");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.check_function("BY_ID", || Pose::BY_ID.as_ref());
        golden.check_codec("STREAM_CODEC", || Pose::STREAM_CODEC);
        for pose in Pose::VALUES {
            golden.field(&pose.to_string(), pose);
        }
        golden.check("values()[Lnet/minecraft/world/entity/Pose;", Pose::values);
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/entity/Pose;",
            |name: String| Pose::value_of(&name),
        );
        golden.check_method("id()I", |this: &mut Pose| this.id());
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Pose| this.get_serialized_name().to_string(),
        );
        golden.finish();
    }
}
