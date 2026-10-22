//! net/minecraft/network/protocol/game/ServerGamePacketListener.java
//!
//! Not ported yet: the handlers of the packets the game protocol has the `Unported` stand-in for
//! (GameProtocols), which come with their packets, its `protocol()`, which comes with
//! `PacketListener`'s, and its parent `ServerPingPacketListener`, which comes with the ping
//! packet. The port only sends these packets, so only the golden cases' recorder handles them.

use crate::net::minecraft::network::protocol::common::server_common_packet_listener::ServerCommonPacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_accept_teleportation_packet::ServerboundAcceptTeleportationPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chat_command_packet::ServerboundChatCommandPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chunk_batch_received_packet::ServerboundChunkBatchReceivedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::network::protocol::game::serverbound_configuration_acknowledged_packet::ServerboundConfigurationAcknowledgedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_move_player_packet::ServerboundMovePlayerPacket;
use crate::net::minecraft::network::protocol::game::serverbound_player_loaded_packet::ServerboundPlayerLoadedPacket;

/// The server's listener in the game protocol.
pub trait ServerGamePacketListener: ServerCommonPacketListener {
    fn handle_accept_teleport_packet(&mut self, packet: &ServerboundAcceptTeleportationPacket);

    fn handle_chat_command(&mut self, packet: &ServerboundChatCommandPacket);

    fn handle_chunk_batch_received(&mut self, packet: &ServerboundChunkBatchReceivedPacket);

    fn handle_client_tick_end(&mut self, packet: &ServerboundClientTickEndPacket);

    fn handle_configuration_acknowledged(
        &mut self,
        packet: &ServerboundConfigurationAcknowledgedPacket,
    );

    /// Every kind of move packet, by the class they all extend.
    fn handle_move_player(&mut self, packet: &ServerboundMovePlayerPacket);

    fn handle_accept_player_load(&mut self, packet: &ServerboundPlayerLoadedPacket);
}

#[cfg(test)]
impl ServerGamePacketListener for crate::net::minecraft::network::packet_listener::tests::Recorder {
    fn handle_accept_teleport_packet(&mut self, packet: &ServerboundAcceptTeleportationPacket) {
        self.record("handleAcceptTeleportPacket", packet);
    }

    fn handle_chat_command(&mut self, packet: &ServerboundChatCommandPacket) {
        self.record("handleChatCommand", packet);
    }

    fn handle_chunk_batch_received(&mut self, packet: &ServerboundChunkBatchReceivedPacket) {
        self.record("handleChunkBatchReceived", packet);
    }

    fn handle_client_tick_end(&mut self, packet: &ServerboundClientTickEndPacket) {
        self.record("handleClientTickEnd", packet);
    }

    fn handle_configuration_acknowledged(
        &mut self,
        packet: &ServerboundConfigurationAcknowledgedPacket,
    ) {
        self.record("handleConfigurationAcknowledged", packet);
    }

    fn handle_move_player(&mut self, packet: &ServerboundMovePlayerPacket) {
        self.record("handleMovePlayer", packet);
    }

    fn handle_accept_player_load(&mut self, packet: &ServerboundPlayerLoadedPacket) {
        self.record("handleAcceptPlayerLoad", packet);
    }
}
