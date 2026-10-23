//! net/minecraft/world/clock/WorldClock.java
//!
//! Not ported yet: `CODEC` and `DIRECT_CODEC`, which wait for DataFixerUpper's codecs. A clock's
//! registry is synchronized, so the server sends its clocks (the overworld's and the end's) and
//! their ids are the order it sent them in.

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::WORLD_CLOCK)));

/// A clock a level's time runs on, which the timelines (the day, the moon) read. It holds
/// nothing: what it says is its ticks (`ClockNetworkState`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldClock;

impl WorldClock {
    /// A clock's holder, by its id in the registry.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;

    pub fn new() -> WorldClock {
        WorldClock
    }

    /// A record's of no components.
    pub fn hash_code(&self) -> i32 {
        0
    }
}

/// `WorldClock[]`.
impl fmt::Display for WorldClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorldClock[]")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::WorldClock;
    use crate::golden::{Decode, Encode, Golden};

    // A record of no components, [].
    impl Decode for WorldClock {
        fn decode(_value: &Value) -> Self {
            WorldClock
        }
    }

    impl Encode for WorldClock {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.clock.WorldClock");
        for name in ["CODEC", "DIRECT_CODEC"] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.check_codec("STREAM_CODEC", || WorldClock::STREAM_CODEC);
        golden.check("<init>()V", WorldClock::new);
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut WorldClock, other: WorldClock| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut WorldClock| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut WorldClock| {
            this.to_string()
        });
        golden.finish();
    }
}
