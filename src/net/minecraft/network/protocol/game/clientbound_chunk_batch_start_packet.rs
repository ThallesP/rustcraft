//! net/minecraft/network/protocol/game/ClientboundChunkBatchStartPacket.java
//!
//! `unit` compares the value to write with `INSTANCE`, by identity in Java. Every value of a Rust
//! unit struct is the same, so `PartialEq` is the same answer.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundChunkBatchStartPacket> =
    LazyLock::new(|| Box::new(stream_codec::unit(ClientboundChunkBatchStartPacket)));

/// The chunks that follow, until `chunk_batch_finished`, are one batch, which the client times. It
/// has no fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundChunkBatchStartPacket;

impl ClientboundChunkBatchStartPacket {
    pub const INSTANCE: ClientboundChunkBatchStartPacket = ClientboundChunkBatchStartPacket;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundChunkBatchStartPacket> =
        &STREAM_CODEC;
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundChunkBatchStartPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chunk_batch_start(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ClientboundChunkBatchStartPacket;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // No fields: [].
    impl Decode for ClientboundChunkBatchStartPacket {
        fn decode(value: &Value) -> Self {
            assert!(
                value.as_array().is_some_and(Vec::is_empty),
                "expected no fields, got {value}"
            );
            ClientboundChunkBatchStartPacket
        }
    }

    impl Encode for ClientboundChunkBatchStartPacket {
        fn encode(&self) -> Value {
            Value::Array(Vec::new())
        }
    }

    type Listener = dyn ClientGamePacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket");
        golden.field("INSTANCE", ClientboundChunkBatchStartPacket::INSTANCE);
        golden.check_codec("STREAM_CODEC", || {
            ClientboundChunkBatchStartPacket::STREAM_CODEC
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundChunkBatchStartPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut ClientboundChunkBatchStartPacket, listener: &mut Recorder| {
                this.handle(listener)
            },
        );
        golden.finish();
    }
}
