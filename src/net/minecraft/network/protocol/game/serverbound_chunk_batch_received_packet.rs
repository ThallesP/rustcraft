//! net/minecraft/network/protocol/game/ServerboundChunkBatchReceivedPacket.java
//!

use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ServerboundChunkBatchReceivedPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ServerboundChunkBatchReceivedPacket::write,
            ServerboundChunkBatchReceivedPacket::from_friendly_byte_buf,
        ))
    });

/// The client's answer to a chunk batch: how many chunks a tick it could take, from how long the
/// batches took it (`ChunkBatchSizeCalculator`).
#[derive(Clone, Debug)]
pub struct ServerboundChunkBatchReceivedPacket {
    desired_chunks_per_tick: f32,
}

impl ServerboundChunkBatchReceivedPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ServerboundChunkBatchReceivedPacket,
    > = &STREAM_CODEC;

    pub fn new(desired_chunks_per_tick: f32) -> ServerboundChunkBatchReceivedPacket {
        ServerboundChunkBatchReceivedPacket {
            desired_chunks_per_tick,
        }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ServerboundChunkBatchReceivedPacket {
        ServerboundChunkBatchReceivedPacket::new(input.read_float())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_float(self.desired_chunks_per_tick);
    }

    pub fn desired_chunks_per_tick(&self) -> f32 {
        self.desired_chunks_per_tick
    }

    /// A record's, which hashes the float's bits (`Float.hashCode`).
    pub fn hash_code(&self) -> i32 {
        Float::hash_code(self.desired_chunks_per_tick)
    }
}

/// A record's equals compares a float component with `Float.compare`, so by `floatToIntBits`: NaN
/// equals NaN, and -0.0 isn't 0.0.
impl PartialEq for ServerboundChunkBatchReceivedPacket {
    fn eq(&self, other: &ServerboundChunkBatchReceivedPacket) -> bool {
        Float::float_to_int_bits(self.desired_chunks_per_tick)
            == Float::float_to_int_bits(other.desired_chunks_per_tick)
    }
}

impl<L: ServerGamePacketListener + ?Sized> Packet<L> for ServerboundChunkBatchReceivedPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::SERVERBOUND_CHUNK_BATCH_RECEIVED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chunk_batch_received(self);
    }
}

/// A record's: `ServerboundChunkBatchReceivedPacket[desiredChunksPerTick=7.0]`.
impl fmt::Display for ServerboundChunkBatchReceivedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ServerboundChunkBatchReceivedPacket[desiredChunksPerTick={}]",
            Float::to_string(self.desired_chunks_per_tick)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ServerboundChunkBatchReceivedPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ServerboundChunkBatchReceivedPacket {
        desired_chunks_per_tick: f32
    });

    type Listener = dyn ServerGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket");
        golden.check_codec("STREAM_CODEC", || {
            ServerboundChunkBatchReceivedPacket::STREAM_CODEC
        });
        golden.check("<init>(F)V", ServerboundChunkBatchReceivedPacket::new);
        golden.check_method(
            "desiredChunksPerTick()F",
            |this: &mut ServerboundChunkBatchReceivedPacket| this.desired_chunks_per_tick(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ServerboundChunkBatchReceivedPacket,
             other: ServerboundChunkBatchReceivedPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ServerboundChunkBatchReceivedPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ServerboundChunkBatchReceivedPacket| this.to_string(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ServerboundChunkBatchReceivedPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V",
            |this: &mut ServerboundChunkBatchReceivedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
