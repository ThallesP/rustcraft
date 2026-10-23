//! net/minecraft/world/clock/ClockNetworkState.java

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClockNetworkState> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        ByteBufCodecs::VAR_LONG,
        ClockNetworkState::total_ticks,
        ByteBufCodecs::FLOAT,
        ClockNetworkState::partial_tick,
        ByteBufCodecs::FLOAT,
        ClockNetworkState::rate,
        ClockNetworkState::new,
    ))
});

/// What the server says of a clock: its ticks so far, the part of a tick past them, and how many
/// of its ticks a game tick is (0 while time stands still).
#[derive(Clone, Copy, Debug)]
pub struct ClockNetworkState {
    total_ticks: i64,
    partial_tick: f32,
    rate: f32,
}

impl ClockNetworkState {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClockNetworkState> = &STREAM_CODEC;

    pub fn new(total_ticks: i64, partial_tick: f32, rate: f32) -> ClockNetworkState {
        ClockNetworkState {
            total_ticks,
            partial_tick,
            rate,
        }
    }

    pub fn total_ticks(&self) -> i64 {
        self.total_ticks
    }

    pub fn partial_tick(&self) -> f32 {
        self.partial_tick
    }

    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// A record's: each component's hash (`Long.hashCode`, the floats' bits), added to 31 times
    /// the ones before.
    pub fn hash_code(&self) -> i32 {
        let long_hash = (self.total_ticks ^ ((self.total_ticks as u64) >> 32) as i64) as i32;
        [
            Float::hash_code(self.partial_tick),
            Float::hash_code(self.rate),
        ]
        .into_iter()
        .fold(long_hash, |hash, component| {
            hash.wrapping_mul(31).wrapping_add(component)
        })
    }
}

/// A record's: the floats compare by their bits, NaNs made one (`Float.compare`).
impl PartialEq for ClockNetworkState {
    fn eq(&self, other: &ClockNetworkState) -> bool {
        self.total_ticks == other.total_ticks
            && Float::float_to_int_bits(self.partial_tick)
                == Float::float_to_int_bits(other.partial_tick)
            && Float::float_to_int_bits(self.rate) == Float::float_to_int_bits(other.rate)
    }
}

/// A record's: `ClockNetworkState[totalTicks=6000, partialTick=0.0, rate=0.0]`.
impl fmt::Display for ClockNetworkState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClockNetworkState[totalTicks={}, partialTick={}, rate={}]",
            self.total_ticks,
            Float::to_string(self.partial_tick),
            Float::to_string(self.rate)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClockNetworkState;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(ClockNetworkState {
        total_ticks: i64,
        partial_tick: f32,
        rate: f32,
    });

    type This = ClockNetworkState;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.clock.ClockNetworkState");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check_codec("STREAM_CODEC", || ClockNetworkState::STREAM_CODEC);
        golden.check("<init>(JFF)V", ClockNetworkState::new);
        golden.check_method("totalTicks()J", |this: &mut This| this.total_ticks());
        golden.check_method("partialTick()F", |this: &mut This| this.partial_tick());
        golden.check_method("rate()F", |this: &mut This| this.rate());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
        golden.finish();
    }
}
