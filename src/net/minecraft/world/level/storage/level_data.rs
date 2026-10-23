//! net/minecraft/world/level/storage/LevelData.java
//!
//! Not ported yet: the interface itself (the level's spawn, time, difficulty and whether it's
//! hardcore), which the world track ports with `ClientLevel`'s level data, and `RespawnData`'s
//! `MAP_CODEC` and `CODEC`, which wait for DataFixerUpper's codecs. Its record's `hashCode` hashes the dimension's
//! identity hash, which changes from run to run.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::float::Float;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::global_pos::GlobalPos;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::util::mth::Mth;
use std::fmt;

static DEFAULT: LazyLock<RespawnData> = LazyLock::new(|| {
    RespawnData::new(
        GlobalPos::of(
            ResourceKey::create(
                Registries::DIMENSION,
                Identifier::with_default_namespace("overworld"),
            ),
            BlockPos::ZERO,
        ),
        0.0,
        0.0,
    )
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, RespawnData> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        GlobalPos::STREAM_CODEC,
        RespawnData::global_pos,
        ByteBufCodecs::FLOAT,
        RespawnData::yaw,
        ByteBufCodecs::FLOAT,
        RespawnData::pitch,
        RespawnData::new,
    ))
});

/// `LevelData.RespawnData`: where players spawn, and which way they face.
#[derive(Clone, Debug)]
pub struct RespawnData {
    global_pos: GlobalPos,
    yaw: f32,
    pitch: f32,
}

impl RespawnData {
    /// The overworld's origin, facing south.
    pub const DEFAULT: &'static LazyLock<RespawnData> = &DEFAULT;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, RespawnData> = &STREAM_CODEC;

    pub fn new(global_pos: GlobalPos, yaw: f32, pitch: f32) -> RespawnData {
        RespawnData {
            global_pos,
            yaw,
            pitch,
        }
    }

    /// The yaw wrapped to -180 to 180 degrees, and the pitch clamped to -90 to 90.
    pub fn of(dimension: ResourceKey, pos: BlockPos, yaw: f32, pitch: f32) -> RespawnData {
        RespawnData::new(
            GlobalPos::of(dimension, pos.immutable()),
            Mth::wrap_degrees_f32(yaw),
            Mth::clamp_f32(pitch, -90.0, 90.0),
        )
    }

    pub fn global_pos(&self) -> GlobalPos {
        self.global_pos.clone()
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn dimension(&self) -> ResourceKey {
        self.global_pos.dimension()
    }

    pub fn pos(&self) -> BlockPos {
        self.global_pos.pos()
    }
}

/// A record's: the angles compare by their bits, NaNs made one (`Float.compare`).
impl PartialEq for RespawnData {
    fn eq(&self, other: &RespawnData) -> bool {
        self.global_pos == other.global_pos
            && Float::float_to_int_bits(self.yaw) == Float::float_to_int_bits(other.yaw)
            && Float::float_to_int_bits(self.pitch) == Float::float_to_int_bits(other.pitch)
    }
}

/// A record's: `RespawnData[globalPos=ResourceKey[minecraft:dimension / minecraft:overworld] BlockPos{x=0, y=0, z=0}, yaw=0.0, pitch=0.0]`.
impl fmt::Display for RespawnData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RespawnData[globalPos={}, yaw={}, pitch={}]",
            self.global_pos,
            Float::to_string(self.yaw),
            Float::to_string(self.pitch)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::RespawnData;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::global_pos::GlobalPos;

    encode_fields!(RespawnData {
        global_pos: GlobalPos,
        yaw: f32,
        pitch: f32,
    });

    type This = RespawnData;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.storage.LevelData$RespawnData");
        for name in ["CODEC", "MAP_CODEC"] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.pending("hashCode()I", "hashes the dimension's identity hash");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.field("DEFAULT", RespawnData::clone(RespawnData::DEFAULT));
        golden.check_codec("STREAM_CODEC", || RespawnData::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/core/GlobalPos;FF)V",
            RespawnData::new,
        );
        golden.check(
            "of(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/core/BlockPos;FF)Lnet/minecraft/world/level/storage/LevelData$RespawnData;",
            RespawnData::of,
        );
        golden.check_method(
            "globalPos()Lnet/minecraft/core/GlobalPos;",
            |this: &mut This| this.global_pos(),
        );
        golden.check_method("yaw()F", |this: &mut This| this.yaw());
        golden.check_method("pitch()F", |this: &mut This| this.pitch());
        golden.check_method(
            "dimension()Lnet/minecraft/resources/ResourceKey;",
            |this: &mut This| this.dimension(),
        );
        golden.check_method("pos()Lnet/minecraft/core/BlockPos;", |this: &mut This| {
            this.pos()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.finish();
    }
}
