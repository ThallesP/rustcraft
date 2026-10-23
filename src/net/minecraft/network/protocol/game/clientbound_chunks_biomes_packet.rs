//! net/minecraft/network/protocol/game/ClientboundChunksBiomesPacket.java
//!
//! Not ported yet: `forChunks` and `ChunkBiomeData`'s constructor from a `LevelChunk`, the
//! server's. `ChunkBiomeData` is a record holding an array, whose `equals`, `hashCode` and
//! `toString` take it by identity, so they aren't ported.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::io::netty::buffer::unpooled::wrapped_buffer;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

static CHUNK_BIOME_DATA_STREAM_CODEC: StaticStreamCodec<ByteBuf, ChunkBiomeData> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            ChunkPos::STREAM_CODEC,
            |data: &ChunkBiomeData| data.pos,
            ByteBufCodecs::byte_array(2097152),
            |data: &ChunkBiomeData| data.buffer.clone(),
            ChunkBiomeData::new,
        ))
    });

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundChunksBiomesPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            (&CHUNK_BIOME_DATA_STREAM_CODEC).apply(ByteBufCodecs::list()),
            |packet: &ClientboundChunksBiomesPacket| packet.chunk_biome_data.clone(),
            ClientboundChunksBiomesPacket::new,
        ))
    });

/// Chunks whose biomes changed, each with its sections' biomes.
#[derive(Clone, Debug)]
pub struct ClientboundChunksBiomesPacket {
    chunk_biome_data: Vec<ChunkBiomeData>,
}

impl ClientboundChunksBiomesPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundChunksBiomesPacket> =
        &STREAM_CODEC;

    pub fn new(chunk_biome_data: Vec<ChunkBiomeData>) -> ClientboundChunksBiomesPacket {
        ClientboundChunksBiomesPacket { chunk_biome_data }
    }

    pub fn chunk_biome_data(&self) -> &[ChunkBiomeData] {
        &self.chunk_biome_data
    }
}

/// A chunk's position and its sections' biome containers, one after another.
#[derive(Clone, Debug)]
pub struct ChunkBiomeData {
    pos: ChunkPos,
    buffer: Vec<i8>,
}

impl ChunkBiomeData {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ChunkBiomeData> =
        &CHUNK_BIOME_DATA_STREAM_CODEC;

    pub fn new(pos: ChunkPos, buffer: Vec<i8>) -> ChunkBiomeData {
        ChunkBiomeData { pos, buffer }
    }

    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    pub fn buffer(&self) -> &[i8] {
        &self.buffer
    }

    pub fn get_read_buffer(&self) -> FriendlyByteBuf {
        FriendlyByteBuf::new(wrapped_buffer(self.buffer.clone()))
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundChunksBiomesPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_CHUNKS_BIOMES
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chunks_biomes(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::ClientboundChunksBiomesPacket;
    use crate::golden::{Golden, hex};
    use crate::io::netty::buffer::unpooled::{self, wrapped_buffer};
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::decoded;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                wrapped_buffer,
                unpooled::buffer,
                ClientboundChunksBiomesPacket::STREAM_CODEC,
                |packet: &ClientboundChunksBiomesPacket| {
                    Value::from(
                        packet
                            .chunk_biome_data()
                            .iter()
                            .map(|data| json!([data.pos().x(), data.pos().z(), hex(data.buffer())]))
                            .collect::<Vec<_>>(),
                    )
                },
            )
        });
        golden.finish();
    }
}
