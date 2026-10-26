//! net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::{self, Blocks};

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundBlockEventPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundBlockEventPacket::write,
            ClientboundBlockEventPacket::from_friendly_byte_buf,
        ))
    });

/// A block's event, which the server's block handled and each client's handles too: a chest's or
/// shulker box's lid, a piston moving, a note, a bell.
#[derive(Clone, Debug)]
pub struct ClientboundBlockEventPacket {
    pos: BlockPos,
    b0: i32,
    b1: i32,
    block: Block,
}

impl ClientboundBlockEventPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundBlockEventPacket,
    > = &STREAM_CODEC;

    pub fn new(pos: BlockPos, block: Block, b0: i32, b1: i32) -> ClientboundBlockEventPacket {
        ClientboundBlockEventPacket { pos, b0, b1, block }
    }

    /// The block is `ByteBufCodecs.registry(Registries.BLOCK)`'s: its id, air for one the
    /// registry doesn't have (`BuiltInRegistries.BLOCK` is a `DefaultedRegistry`, whose `byId`
    /// falls back on air, so `byIdOrThrow` never throws).
    fn from_friendly_byte_buf(input: &mut RegistryFriendlyByteBuf) -> ClientboundBlockEventPacket {
        let pos = input.read_block_pos();
        let b0 = i32::from(input.read_unsigned_byte());
        let b1 = i32::from(input.read_unsigned_byte());
        let id = input.read_var_int();
        let block = blocks::registry().by_id(id).unwrap_or(Blocks::AIR);
        ClientboundBlockEventPacket { pos, b0, b1, block }
    }

    fn write(&self, output: &mut RegistryFriendlyByteBuf) {
        output.write_block_pos(&self.pos);
        output.write_byte(self.b0);
        output.write_byte(self.b1);
        output.write_var_int(self.block.get_id_in_registry());
    }

    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    pub fn get_b0(&self) -> i32 {
        self.b0
    }

    pub fn get_b1(&self) -> i32 {
        self.b1
    }

    pub fn get_block(&self) -> Block {
        self.block
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundBlockEventPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_BLOCK_EVENT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_block_event(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundBlockEventPacket;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::{
        decoded, registry_reader, registry_writer,
    };
    use crate::net::minecraft::world::level::block::blocks;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundBlockEventPacket");
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                registry_reader,
                registry_writer,
                ClientboundBlockEventPacket::STREAM_CODEC,
                |packet: &ClientboundBlockEventPacket| {
                    let pos = packet.get_pos();
                    json!([
                        pos.get_x(),
                        pos.get_y(),
                        pos.get_z(),
                        packet.get_b0(),
                        packet.get_b1(),
                        blocks::registry().get_key(packet.get_block()).to_string()
                    ])
                },
            )
        });
        golden.finish();
    }
}
