//! net/minecraft/network/protocol/game/ClientboundChunkBatchFinishedPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundChunkBatchFinishedPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundChunkBatchFinishedPacket::write,
            ClientboundChunkBatchFinishedPacket::from_friendly_byte_buf,
        ))
    });

/// The end of a chunk batch, with how many chunks it had. The client answers with how many it
/// wants a tick.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundChunkBatchFinishedPacket {
    batch_size: i32,
}

impl ClientboundChunkBatchFinishedPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundChunkBatchFinishedPacket,
    > = &STREAM_CODEC;

    pub fn new(batch_size: i32) -> ClientboundChunkBatchFinishedPacket {
        ClientboundChunkBatchFinishedPacket { batch_size }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundChunkBatchFinishedPacket {
        ClientboundChunkBatchFinishedPacket::new(input.read_var_int())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_var_int(self.batch_size);
    }

    pub fn batch_size(&self) -> i32 {
        self.batch_size
    }

    /// A record's: its one component's hash.
    pub fn hash_code(&self) -> i32 {
        self.batch_size
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundChunkBatchFinishedPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chunk_batch_finished(self);
    }
}

/// A record's: `ClientboundChunkBatchFinishedPacket[batchSize=9]`.
impl fmt::Display for ClientboundChunkBatchFinishedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundChunkBatchFinishedPacket[batchSize={}]",
            self.batch_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundChunkBatchFinishedPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundChunkBatchFinishedPacket { batch_size: i32 });

    type Listener = dyn ClientGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundChunkBatchFinishedPacket::STREAM_CODEC
        });
        golden.check("<init>(I)V", ClientboundChunkBatchFinishedPacket::new);
        golden.check_method(
            "batchSize()I",
            |this: &mut ClientboundChunkBatchFinishedPacket| this.batch_size(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientboundChunkBatchFinishedPacket,
             other: ClientboundChunkBatchFinishedPacket| *this == other,
        );
        golden.check_method(
            "hashCode()I",
            |this: &mut ClientboundChunkBatchFinishedPacket| this.hash_code(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientboundChunkBatchFinishedPacket| this.to_string(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundChunkBatchFinishedPacket| {
                Packet::<Listener>::r#type(this).clone()
            },
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut ClientboundChunkBatchFinishedPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
