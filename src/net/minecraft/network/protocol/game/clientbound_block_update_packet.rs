//! net/minecraft/network/protocol/game/ClientboundBlockUpdatePacket.java
//!
//! Not ported yet: the constructor reading the state from a `BlockGetter`, the server's.

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundBlockUpdatePacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite2(
            BlockPos::STREAM_CODEC,
            |packet: &ClientboundBlockUpdatePacket| packet.pos,
            // `idMapper(Block.BLOCK_STATE_REGISTRY)`: `byIdOrThrow` and `getIdOrThrow`.
            ByteBufCodecs::id_mapper(
                |id| {
                    Block::block_state_registry()
                        .by_id(id)
                        .unwrap_or_else(|| panic!("No value with id {id}"))
                },
                |state: &BlockState| Block::block_state_registry().get_id(*state),
            ),
            |packet: &ClientboundBlockUpdatePacket| packet.block_state,
            ClientboundBlockUpdatePacket::new,
        ))
    });

/// A block the server changed.
#[derive(Clone, Debug)]
pub struct ClientboundBlockUpdatePacket {
    pos: BlockPos,
    block_state: BlockState,
}

impl ClientboundBlockUpdatePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundBlockUpdatePacket,
    > = &STREAM_CODEC;

    pub fn new(pos: BlockPos, state: BlockState) -> ClientboundBlockUpdatePacket {
        ClientboundBlockUpdatePacket {
            pos,
            block_state: state,
        }
    }

    pub fn get_block_state(&self) -> BlockState {
        self.block_state
    }

    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBlockUpdatePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BLOCK_UPDATE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_block_update(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundBlockUpdatePacket;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::{
        decoded, registry_reader, registry_writer,
    };

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                registry_reader,
                registry_writer,
                ClientboundBlockUpdatePacket::STREAM_CODEC,
                |packet: &ClientboundBlockUpdatePacket| {
                    let pos = packet.get_pos();
                    json!([
                        pos.get_x(),
                        pos.get_y(),
                        pos.get_z(),
                        packet.get_block_state().id()
                    ])
                },
            )
        });
        golden.finish();
    }
}
