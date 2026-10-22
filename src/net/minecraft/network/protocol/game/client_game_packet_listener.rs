//! net/minecraft/network/protocol/game/ClientGamePacketListener.java
//!
//! Not ported yet: the handlers of the packets the game protocol has the `Unported` stand-in for
//! (GameProtocols), which come with their packets (the world data's with the world track), its `protocol()`, which comes with
//! `PacketListener`'s, and its parent `ClientPongPacketListener`, which comes with the pong
//! packet.

use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_bundle_delimiter_packet::ClientboundBundleDelimiterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket;
use crate::net::minecraft::network::protocol::game::clientbound_game_event_packet::ClientboundGameEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_login_packet::ClientboundLoginPacket;
use crate::net::minecraft::network::protocol::game::clientbound_player_position_packet::ClientboundPlayerPositionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_start_configuration_packet::ClientboundStartConfigurationPacket;

/// The client's listener in the game protocol.
pub trait ClientGamePacketListener: ClientCommonPacketListener {
    fn handle_login(&mut self, packet: &ClientboundLoginPacket);

    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket);

    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket);

    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket);

    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket);

    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket);

    /// Java's delimiter is never handled: the bundle packer takes it out of the stream. The
    /// packer isn't ported yet, so the connection hands it on, and it does nothing.
    fn handle_bundle_delimiter(&mut self, packet: &ClientboundBundleDelimiterPacket) {
        let _ = packet;
    }
}

#[cfg(test)]
impl ClientGamePacketListener for crate::net::minecraft::network::packet_listener::tests::Recorder {
    // It has no golden cases yet (the harness has no RegistryFriendlyByteBuf), so the recorder
    // only records its type.
    fn handle_login(&mut self, packet: &ClientboundLoginPacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleLogin", r#type);
    }

    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket) {
        self.record("handleMovePlayer", packet);
    }

    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket) {
        self.record("handleGameEvent", packet);
    }

    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket) {
        self.record("handleConfigurationStart", packet);
    }

    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket) {
        self.record("handleChunkBatchStart", packet);
    }

    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket) {
        self.record("handleChunkBatchFinished", packet);
    }
}
