//! net/minecraft/network/protocol/game/ClientboundSectionBlocksUpdatePacket.java
//!
//! Not ported yet: the constructor making the packet from a section's changes, the server's. A
//! state id the registry doesn't have reads as no state (Java's null), which `runUpdates` hands
//! out as it is and `write` writes as 0 (`Block.getId(null)`).

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSectionBlocksUpdatePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSectionBlocksUpdatePacket::write,
            ClientboundSectionBlocksUpdatePacket::from_friendly_byte_buf,
        ))
    });

/// Blocks the server changed in one section: each a position in the section (x, z and y, 4 bits
/// each) and its state.
#[derive(Clone, Debug)]
pub struct ClientboundSectionBlocksUpdatePacket {
    section_pos: SectionPos,
    positions: Vec<i16>,
    states: Vec<Option<BlockState>>,
}

impl ClientboundSectionBlocksUpdatePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSectionBlocksUpdatePacket,
    > = &STREAM_CODEC;

    /// Panics on a negative count, where Java throws `NegativeArraySizeException`.
    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSectionBlocksUpdatePacket {
        let section_pos = SectionPos::STREAM_CODEC.decode(input);
        let count = input.read_var_int();
        assert!(count >= 0, "NegativeArraySizeException: {count}");
        let mut positions = Vec::with_capacity(count as usize);
        let mut states = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let packed_change = input.read_var_long();
            positions.push((packed_change & 4095) as i16);
            states.push(Block::block_state_registry().by_id(((packed_change as u64) >> 12) as i32));
        }

        ClientboundSectionBlocksUpdatePacket {
            section_pos,
            positions,
            states,
        }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        SectionPos::STREAM_CODEC.encode(output, &self.section_pos);
        output.write_var_int(self.positions.len() as i32);

        for (position, state) in self.positions.iter().zip(&self.states) {
            output.write_var_long((Block::get_id(*state) as i64) << 12 | *position as i64);
        }
    }

    pub fn section_pos(&self) -> SectionPos {
        self.section_pos
    }

    /// Each change's block and state, in order.
    pub fn run_updates(&self, update_function: &mut dyn FnMut(BlockPos, Option<BlockState>)) {
        for (&packed_pos, &state) in self.positions.iter().zip(&self.states) {
            update_function(self.section_pos.relative_to_block_pos(packed_pos), state);
        }
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSectionBlocksUpdatePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SECTION_BLOCKS_UPDATE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_chunk_blocks_update(self);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ClientboundSectionBlocksUpdatePacket;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::{
        decoded, friendly, friendly_writer,
    };

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket",
        );
        golden.check("STREAM_CODEC.decode", |bytes: String| {
            decoded(
                &bytes,
                friendly,
                friendly_writer,
                ClientboundSectionBlocksUpdatePacket::STREAM_CODEC,
                |packet: &ClientboundSectionBlocksUpdatePacket| {
                    let mut changes = Vec::new();
                    packet.run_updates(&mut |pos, state| {
                        changes.push(json!([
                            pos.get_x(),
                            pos.get_y(),
                            pos.get_z(),
                            state.map(|state| state.id())
                        ]));
                    });
                    let section = packet.section_pos();
                    json!([section.x(), section.y(), section.z(), changes])
                },
            )
        });
        golden.finish();
    }
}
