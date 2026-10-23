//! net/minecraft/network/protocol/game/ClientGamePacketListener.java
//!
//! Not ported yet: the handlers of the packets the game protocol has the `Unported` stand-in for
//! (GameProtocols), which come with their packets (the world data's with the world track), and
//! its `protocol()`, which comes with `PacketListener`'s.

use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_bundle_packet::ClientboundBundlePacket;
use crate::net::minecraft::network::protocol::game::clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket;
use crate::net::minecraft::network::protocol::game::clientbound_game_event_packet::ClientboundGameEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_initialize_border_packet::ClientboundInitializeBorderPacket;
use crate::net::minecraft::network::protocol::game::clientbound_login_packet::ClientboundLoginPacket;
use crate::net::minecraft::network::protocol::game::clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_player_position_packet::ClientboundPlayerPositionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_experience_packet::ClientboundSetExperiencePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_health_packet::ClientboundSetHealthPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_held_slot_packet::ClientboundSetHeldSlotPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_simulation_distance_packet::ClientboundSetSimulationDistancePacket;
use crate::net::minecraft::network::protocol::game::clientbound_start_configuration_packet::ClientboundStartConfigurationPacket;
use crate::net::minecraft::network::protocol::game::clientbound_ticking_state_packet::ClientboundTickingStatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_ticking_step_packet::ClientboundTickingStepPacket;
use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;

/// The client's listener in the game protocol.
pub trait ClientGamePacketListener: ClientCommonPacketListener + ClientPongPacketListener {
    fn handle_login(&mut self, packet: &ClientboundLoginPacket);

    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket);

    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket);

    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket);

    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket);

    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket);

    fn handle_bundle_packet(&mut self, packet: &ClientboundBundlePacket);

    fn handle_change_difficulty(&mut self, packet: &ClientboundChangeDifficultyPacket);

    fn handle_initialize_border(&mut self, packet: &ClientboundInitializeBorderPacket);

    fn handle_player_abilities(&mut self, packet: &ClientboundPlayerAbilitiesPacket);

    fn handle_set_chunk_cache_center(&mut self, packet: &ClientboundSetChunkCacheCenterPacket);

    fn handle_set_chunk_cache_radius(&mut self, packet: &ClientboundSetChunkCacheRadiusPacket);

    fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket);

    fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket);

    fn handle_set_held_slot(&mut self, packet: &ClientboundSetHeldSlotPacket);

    fn handle_set_simulation_distance(&mut self, packet: &ClientboundSetSimulationDistancePacket);

    fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket);

    fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket);
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

    // It has no golden cases (its sub-packets are any of the protocol's), so the recorder only
    // records its type.
    fn handle_bundle_packet(&mut self, packet: &ClientboundBundlePacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleBundlePacket", r#type);
    }

    fn handle_change_difficulty(&mut self, packet: &ClientboundChangeDifficultyPacket) {
        self.record("handleChangeDifficulty", packet);
    }

    fn handle_initialize_border(&mut self, packet: &ClientboundInitializeBorderPacket) {
        self.record("handleInitializeBorder", packet);
    }

    fn handle_player_abilities(&mut self, packet: &ClientboundPlayerAbilitiesPacket) {
        self.record("handlePlayerAbilities", packet);
    }

    fn handle_set_chunk_cache_center(&mut self, packet: &ClientboundSetChunkCacheCenterPacket) {
        self.record("handleSetChunkCacheCenter", packet);
    }

    fn handle_set_chunk_cache_radius(&mut self, packet: &ClientboundSetChunkCacheRadiusPacket) {
        self.record("handleSetChunkCacheRadius", packet);
    }

    fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket) {
        self.record("handleSetExperience", packet);
    }

    fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket) {
        self.record("handleSetHealth", packet);
    }

    fn handle_set_held_slot(&mut self, packet: &ClientboundSetHeldSlotPacket) {
        self.record("handleSetHeldSlot", packet);
    }

    fn handle_set_simulation_distance(&mut self, packet: &ClientboundSetSimulationDistancePacket) {
        self.record("handleSetSimulationDistance", packet);
    }

    fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket) {
        self.record("handleTickingState", packet);
    }

    fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket) {
        self.record("handleTickingStep", packet);
    }
}
