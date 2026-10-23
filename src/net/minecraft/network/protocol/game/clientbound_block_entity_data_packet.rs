//! net/minecraft/network/protocol/game/ClientboundBlockEntityDataPacket.java
//!
//! Not ported yet: `create`, which the server makes the packet with.

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_packet_data::BLOCK_ENTITY_TYPE_STREAM_CODEC;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundBlockEntityDataPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite3(
            BlockPos::STREAM_CODEC,
            |packet: &ClientboundBlockEntityDataPacket| packet.pos,
            &*BLOCK_ENTITY_TYPE_STREAM_CODEC,
            |packet: &ClientboundBlockEntityDataPacket| packet.r#type,
            ByteBufCodecs::TRUSTED_COMPOUND_TAG,
            |packet: &ClientboundBlockEntityDataPacket| packet.tag.clone(),
            ClientboundBlockEntityDataPacket::new,
        ))
    });

/// A block entity's data, as the server sends it when the block entity changes.
#[derive(Clone, Debug)]
pub struct ClientboundBlockEntityDataPacket {
    pos: BlockPos,
    r#type: BlockEntityType,
    tag: CompoundTag,
}

impl ClientboundBlockEntityDataPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundBlockEntityDataPacket,
    > = &STREAM_CODEC;

    fn new(
        pos: BlockPos,
        r#type: BlockEntityType,
        tag: CompoundTag,
    ) -> ClientboundBlockEntityDataPacket {
        ClientboundBlockEntityDataPacket { pos, r#type, tag }
    }

    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    pub fn get_type(&self) -> BlockEntityType {
        self.r#type
    }

    pub fn get_tag(&self) -> &CompoundTag {
        &self.tag
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBlockEntityDataPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_block_entity_data(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundBlockEntityDataPacket;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::{
        decoded, registry_reader, registry_writer,
    };

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                registry_reader,
                registry_writer,
                ClientboundBlockEntityDataPacket::STREAM_CODEC,
                |packet: &ClientboundBlockEntityDataPacket| {
                    let pos = packet.get_pos();
                    json!([
                        pos.get_x(),
                        pos.get_y(),
                        pos.get_z(),
                        packet.get_type().key(),
                        packet.get_tag().to_string()
                    ])
                },
            )
        });
        golden.finish();
    }
}
