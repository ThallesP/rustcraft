//! net/minecraft/network/protocol/game/ClientboundForgetLevelChunkPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundForgetLevelChunkPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundForgetLevelChunkPacket::write,
            ClientboundForgetLevelChunkPacket::from_friendly_byte_buf,
        ))
    });

/// A chunk the client is to drop.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundForgetLevelChunkPacket {
    pos: ChunkPos,
}

impl ClientboundForgetLevelChunkPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundForgetLevelChunkPacket,
    > = &STREAM_CODEC;

    pub fn new(pos: ChunkPos) -> ClientboundForgetLevelChunkPacket {
        ClientboundForgetLevelChunkPacket { pos }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundForgetLevelChunkPacket {
        ClientboundForgetLevelChunkPacket::new(input.read_chunk_pos())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_chunk_pos(&self.pos);
    }

    pub fn pos(&self) -> ChunkPos {
        self.pos
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundForgetLevelChunkPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_FORGET_LEVEL_CHUNK
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_forget_level_chunk(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundForgetLevelChunkPacket;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::{
        decoded, friendly, friendly_writer,
    };

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                friendly,
                friendly_writer,
                ClientboundForgetLevelChunkPacket::STREAM_CODEC,
                |packet: &ClientboundForgetLevelChunkPacket| {
                    json!([packet.pos().x(), packet.pos().z()])
                },
            )
        });
        golden.finish();
    }
}
