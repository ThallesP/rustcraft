//! net/minecraft/network/protocol/game/ClientboundLightUpdatePacket.java
//!
//! Not ported yet: the constructor making the packet from a light engine, the server's.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet_data::ClientboundLightUpdatePacketData;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundLightUpdatePacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite3(
            ByteBufCodecs::VAR_INT,
            |packet: &ClientboundLightUpdatePacket| packet.x,
            ByteBufCodecs::VAR_INT,
            |packet: &ClientboundLightUpdatePacket| packet.z,
            ClientboundLightUpdatePacketData::STREAM_CODEC,
            |packet: &ClientboundLightUpdatePacket| packet.light_data.clone(),
            ClientboundLightUpdatePacket::new,
        ))
    });

/// A chunk's light, changed.
#[derive(Clone, Debug)]
pub struct ClientboundLightUpdatePacket {
    x: i32,
    z: i32,
    light_data: ClientboundLightUpdatePacketData,
}

impl ClientboundLightUpdatePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundLightUpdatePacket> =
        &STREAM_CODEC;

    pub fn new(
        x: i32,
        z: i32,
        light_data: ClientboundLightUpdatePacketData,
    ) -> ClientboundLightUpdatePacket {
        ClientboundLightUpdatePacket { x, z, light_data }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn z(&self) -> i32 {
        self.z
    }

    pub fn light_data(&self) -> &ClientboundLightUpdatePacketData {
        &self.light_data
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundLightUpdatePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_LIGHT_UPDATE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_light_update_packet(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundLightUpdatePacket;
    use crate::golden::Golden;
    use crate::io::netty::buffer::unpooled::{self, wrapped_buffer};
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::decoded;
    use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet_data::tests::encode_light;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundLightUpdatePacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                wrapped_buffer,
                unpooled::buffer,
                ClientboundLightUpdatePacket::STREAM_CODEC,
                |packet: &ClientboundLightUpdatePacket| {
                    json!([packet.x(), packet.z(), encode_light(packet.light_data())])
                },
            )
        });
        golden.finish();
    }
}
