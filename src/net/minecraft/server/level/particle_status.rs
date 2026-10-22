//! net/minecraft/server/level/ParticleStatus.java
//!
//! Not ported yet: `LEGACY_CODEC`, which waits for DataFixerUpper's codecs, and `caption()`,
//! which waits for text components.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};

/// How many particles a client shows: all, fewer, or the fewest. The client sends it in its
/// `ClientInformation`, and the oracle plays with `MINIMAL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleStatus {
    ALL,
    DECREASED,
    MINIMAL,
}

// By id, wrapping around.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> ParticleStatus + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        ParticleStatus::id,
        &ParticleStatus::VALUES,
        OutOfBoundsStrategy::WRAP,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ParticleStatus> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), ParticleStatus::id)));

impl ParticleStatus {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ParticleStatus> = &STREAM_CODEC;

    pub const VALUES: [ParticleStatus; 3] = [
        ParticleStatus::ALL,
        ParticleStatus::DECREASED,
        ParticleStatus::MINIMAL,
    ];

    pub fn values() -> [ParticleStatus; 3] {
        ParticleStatus::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> ParticleStatus {
        match name {
            "ALL" => ParticleStatus::ALL,
            "DECREASED" => ParticleStatus::DECREASED,
            "MINIMAL" => ParticleStatus::MINIMAL,
            _ => panic!("IllegalArgumentException: no ParticleStatus constant {name}"),
        }
    }

    // The id each constant is made with.
    fn id(&self) -> i32 {
        match self {
            ParticleStatus::ALL => 0,
            ParticleStatus::DECREASED => 1,
            ParticleStatus::MINIMAL => 2,
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for ParticleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::ParticleStatus;
    use crate::golden::{Golden, encode_names};

    encode_names!(ParticleStatus {
        ALL,
        DECREASED,
        MINIMAL
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.server.level.ParticleStatus");
        golden.pending("LEGACY_CODEC", "waits for DataFixerUpper's codecs");
        golden.pending(
            "caption()Lnet/minecraft/network/chat/Component;",
            "waits for text components",
        );
        golden.check_codec("STREAM_CODEC", || ParticleStatus::STREAM_CODEC);
        for status in ParticleStatus::VALUES {
            golden.field(&format!("{status:?}"), status);
        }
        golden.check(
            "values()[Lnet/minecraft/server/level/ParticleStatus;",
            ParticleStatus::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/server/level/ParticleStatus;",
            |name: String| ParticleStatus::value_of(&name),
        );
        golden.finish();
    }
}
