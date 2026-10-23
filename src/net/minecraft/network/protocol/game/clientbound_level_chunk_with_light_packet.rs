//! net/minecraft/network/protocol/game/ClientboundLevelChunkWithLightPacket.java
//!
//! Not ported yet: the constructor making the packet from a `LevelChunk` and a light engine,
//! the server's.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet_data::ClientboundLightUpdatePacketData;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<
    RegistryFriendlyByteBuf,
    ClientboundLevelChunkWithLightPacket,
> = LazyLock::new(|| {
    Box::new(stream_codec::composite4(
        ByteBufCodecs::INT,
        |packet: &ClientboundLevelChunkWithLightPacket| packet.x,
        ByteBufCodecs::INT,
        |packet: &ClientboundLevelChunkWithLightPacket| packet.z,
        ClientboundLevelChunkPacketData::STREAM_CODEC,
        |packet: &ClientboundLevelChunkWithLightPacket| packet.chunk_data.clone(),
        ClientboundLightUpdatePacketData::STREAM_CODEC,
        |packet: &ClientboundLevelChunkWithLightPacket| packet.light_data.clone(),
        ClientboundLevelChunkWithLightPacket::new,
    ))
});

/// A chunk the client is to keep, with its light.
#[derive(Clone, Debug)]
pub struct ClientboundLevelChunkWithLightPacket {
    x: i32,
    z: i32,
    chunk_data: ClientboundLevelChunkPacketData,
    light_data: ClientboundLightUpdatePacketData,
}

impl ClientboundLevelChunkWithLightPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundLevelChunkWithLightPacket,
    > = &STREAM_CODEC;

    pub fn new(
        x: i32,
        z: i32,
        chunk_data: ClientboundLevelChunkPacketData,
        light_data: ClientboundLightUpdatePacketData,
    ) -> ClientboundLevelChunkWithLightPacket {
        ClientboundLevelChunkWithLightPacket {
            x,
            z,
            chunk_data,
            light_data,
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn z(&self) -> i32 {
        self.z
    }

    pub fn chunk_data(&self) -> &ClientboundLevelChunkPacketData {
        &self.chunk_data
    }

    pub fn light_data(&self) -> &ClientboundLightUpdatePacketData {
        &self.light_data
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundLevelChunkWithLightPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_level_chunk_with_light(self);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;

    use serde_json::{Value, json};

    use super::ClientboundLevelChunkWithLightPacket;
    use crate::golden::{Encode, Golden, catch, hex, readable, unhex};
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::io::netty::buffer::unpooled::{self, wrapped_buffer};
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet_data::tests::encode_light;
    use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::LevelChunkSection;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::tests::{encode_section, registries};
    use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;

    /// Decodes the bytes with the codec: `[fields(packet), unread, written]`, the packet written
    /// back (or `{"throws": …}`), like `LevelChunkSuites.decodeCase` in oracle/golden/.
    pub(crate) fn decoded<B: AsMut<ByteBuf>, C: StreamCodec<Buffer = B>>(
        hex_bytes: &str,
        reader: impl Fn(Vec<i8>) -> B,
        writer: impl Fn() -> B,
        codec: C,
        fields: impl Fn(&C::Value) -> Value,
    ) -> Value {
        let mut input = reader(unhex(hex_bytes));
        let packet = codec.decode(&mut input);
        let unread = input.as_mut().readable_bytes();
        let decoded = fields(&packet);
        let written = catch(|| {
            let mut output = writer();
            codec.encode(&mut output, &packet);
            Value::from(hex(&readable(output.as_mut())))
        })
        .unwrap_or_else(|_| json!({ "throws": "panic" }));
        json!([decoded, unread, written])
    }

    pub(crate) fn friendly(bytes: Vec<i8>) -> FriendlyByteBuf {
        FriendlyByteBuf::new(wrapped_buffer(bytes))
    }

    pub(crate) fn friendly_writer() -> FriendlyByteBuf {
        FriendlyByteBuf::new(unpooled::buffer())
    }

    /// The registries the chunk packets read with: the block entity types are built in, so none.
    pub(crate) fn registry_reader(bytes: Vec<i8>) -> RegistryFriendlyByteBuf {
        RegistryFriendlyByteBuf::new(wrapped_buffer(bytes), Arc::new(RegistryAccess::default()))
    }

    pub(crate) fn registry_writer() -> RegistryFriendlyByteBuf {
        RegistryFriendlyByteBuf::new(unpooled::buffer(), Arc::new(RegistryAccess::default()))
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&golden.input("BIOMES"));
        let section_count = golden.input("SECTION_COUNT").as_i64().expect("an int") as i32;
        let factory = PalettedContainerFactory::create(&registries);
        let fields = |packet: &ClientboundLevelChunkWithLightPacket| {
            fields(packet, &factory, &registries, section_count)
        };
        for method in ["captured", "generated"] {
            golden.check(method, |bytes: String| {
                decoded(
                    &bytes,
                    registry_reader,
                    registry_writer,
                    ClientboundLevelChunkWithLightPacket::STREAM_CODEC,
                    fields,
                )
            });
        }
        golden.finish();
    }

    // Mirrors ChunkPacketSuite.fields in oracle/golden/.
    fn fields(
        packet: &ClientboundLevelChunkWithLightPacket,
        factory: &PalettedContainerFactory,
        registries: &RegistryAccess,
        section_count: i32,
    ) -> Value {
        let biomes = registries.lookup_or_throw(Registries::BIOME);
        let data = packet.chunk_data();
        let heightmaps: Vec<Value> = data
            .get_heightmaps()
            .iter()
            .map(|(r#type, longs)| json!([r#type.get_serialization_key(), longs]))
            .collect();
        let mut block_entities = Vec::new();
        data.for_each_block_entity_tag(packet.x(), packet.z(), &mut |pos, r#type, tag| {
            block_entities.push(json!([
                pos.get_x(),
                pos.get_y(),
                pos.get_z(),
                r#type.key(),
                tag.map(|tag| tag.encode())
            ]));
        });
        let mut buffer = data.get_read_buffer();
        let mut read = Vec::new();
        let mut sections = Vec::new();
        let unread_sections = catch(|| {
            for _ in 0..section_count {
                let mut section = LevelChunkSection::new(factory);
                section.read(&mut buffer);
                let mut state = encode_section(&section, biomes);
                section.recalc_block_counts();
                state
                    .as_array_mut()
                    .expect("a section is an array")
                    .push(section.counts().encode());
                sections.push(state);
                read.push(section);
            }
            Value::from(buffer.readable_bytes())
        })
        .unwrap_or_else(|_| json!({ "throws": "panic" }));
        let mut written = friendly_writer();
        for section in &read {
            section.write(&mut written);
        }
        json!([
            packet.x(),
            packet.z(),
            heightmaps,
            block_entities,
            encode_light(packet.light_data()),
            sections,
            unread_sections,
            hex(&readable(&written))
        ])
    }
}
