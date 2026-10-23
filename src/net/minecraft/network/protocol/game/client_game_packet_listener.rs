//! net/minecraft/network/protocol/game/ClientGamePacketListener.java
//!
//! Not ported yet: the handlers of the packets the game protocol has the `Unported` stand-in for
//! (GameProtocols), which come with their packets, and its `protocol()`, which comes with
//! `PacketListener`'s.

use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use crate::net::minecraft::network::protocol::game::clientbound_block_entity_data_packet::ClientboundBlockEntityDataPacket;
use crate::net::minecraft::network::protocol::game::clientbound_block_update_packet::ClientboundBlockUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_bundle_packet::ClientboundBundlePacket;
use crate::net::minecraft::network::protocol::game::clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket;
use crate::net::minecraft::network::protocol::game::clientbound_chunks_biomes_packet::ClientboundChunksBiomesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_clear_titles_packet::ClientboundClearTitlesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_damage_event_packet::ClientboundDamageEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_entity_event_packet::ClientboundEntityEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;
use crate::net::minecraft::network::protocol::game::clientbound_game_event_packet::ClientboundGameEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_hurt_animation_packet::ClientboundHurtAnimationPacket;
use crate::net::minecraft::network::protocol::game::clientbound_initialize_border_packet::ClientboundInitializeBorderPacket;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet::ClientboundLightUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_login_packet::ClientboundLoginPacket;
use crate::net::minecraft::network::protocol::game::clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_player_position_packet::ClientboundPlayerPositionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_recipe_book_settings_packet::ClientboundRecipeBookSettingsPacket;
use crate::net::minecraft::network::protocol::game::clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_server_data_packet::ClientboundServerDataPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_action_bar_text_packet::ClientboundSetActionBarTextPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_default_spawn_position_packet::ClientboundSetDefaultSpawnPositionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_entity_motion_packet::ClientboundSetEntityMotionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_experience_packet::ClientboundSetExperiencePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_health_packet::ClientboundSetHealthPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_held_slot_packet::ClientboundSetHeldSlotPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_simulation_distance_packet::ClientboundSetSimulationDistancePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_subtitle_text_packet::ClientboundSetSubtitleTextPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_time_packet::ClientboundSetTimePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_title_text_packet::ClientboundSetTitleTextPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_titles_animation_packet::ClientboundSetTitlesAnimationPacket;
use crate::net::minecraft::network::protocol::game::clientbound_start_configuration_packet::ClientboundStartConfigurationPacket;
use crate::net::minecraft::network::protocol::game::clientbound_system_chat_packet::ClientboundSystemChatPacket;
use crate::net::minecraft::network::protocol::game::clientbound_tab_list_packet::ClientboundTabListPacket;
use crate::net::minecraft::network::protocol::game::clientbound_ticking_state_packet::ClientboundTickingStatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_ticking_step_packet::ClientboundTickingStepPacket;
use crate::net::minecraft::network::protocol::game::clientbound_update_attributes_packet::ClientboundUpdateAttributesPacket;
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
    fn handle_level_chunk_with_light(&mut self, packet: &ClientboundLevelChunkWithLightPacket);

    fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket);

    fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket);

    fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket);

    fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket);

    fn handle_chunk_blocks_update(&mut self, packet: &ClientboundSectionBlocksUpdatePacket);

    fn handle_chunks_biomes(&mut self, packet: &ClientboundChunksBiomesPacket);

    fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket);

    fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket);

    fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket);

    fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket);

    fn handle_recipe_book_settings(&mut self, packet: &ClientboundRecipeBookSettingsPacket);

    fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket);

    fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket);

    fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket);

    fn handle_damage_event(&mut self, packet: &ClientboundDamageEventPacket);

    fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket);

    fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket);
    fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket);

    fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket);

    fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket);

    fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket);

    fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket);

    fn set_action_bar_text(&mut self, packet: &ClientboundSetActionBarTextPacket);

    fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket);

    fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket);
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

    // The world data's packets have no `handle` golden cases (the harness has no
    // RegistryFriendlyByteBuf generator), so the recorder records their types.
    fn handle_level_chunk_with_light(&mut self, packet: &ClientboundLevelChunkWithLightPacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleLevelChunkWithLight", r#type);
    }

    fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleLightUpdatePacket", r#type);
    }

    fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleForgetLevelChunk", r#type);
    }

    fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleBlockUpdate", r#type);
    }

    fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleBlockEntityData", r#type);
    }

    fn handle_chunk_blocks_update(&mut self, packet: &ClientboundSectionBlocksUpdatePacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleChunkBlocksUpdate", r#type);
    }

    fn handle_chunks_biomes(&mut self, packet: &ClientboundChunksBiomesPacket) {
        let r#type =
            crate::net::minecraft::network::protocol::packet::Packet::<Self>::r#type(packet);
        self.record("handleChunksBiomes", r#type);
    }

    fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket) {
        self.record("handleSetTime", packet);
    }

    fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket) {
        self.record("handleUpdateAttributes", packet);
    }

    fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket) {
        self.record("handleSetSpawn", packet);
    }

    fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket) {
        self.record("handleSetEntityData", packet);
    }

    fn handle_recipe_book_settings(&mut self, packet: &ClientboundRecipeBookSettingsPacket) {
        self.record("handleRecipeBookSettings", packet);
    }

    fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket) {
        self.record("handleSetEntityMotion", packet);
    }

    fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket) {
        self.record("handleHurtAnimation", packet);
    }

    fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket) {
        self.record("handleEntityEvent", packet);
    }

    fn handle_damage_event(&mut self, packet: &ClientboundDamageEventPacket) {
        self.record("handleDamageEvent", packet);
    }

    fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket) {
        self.record("handleAddEntity", packet);
    }

    fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket) {
        self.record("handleRemoveEntities", packet);
    }
    fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket) {
        self.record("handleSystemChat", packet);
    }

    fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket) {
        self.record("setTitleText", packet);
    }

    fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket) {
        self.record("setTitlesAnimation", packet);
    }

    fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket) {
        self.record("handleTitlesClear", packet);
    }

    fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket) {
        self.record("setSubtitleText", packet);
    }

    fn set_action_bar_text(&mut self, packet: &ClientboundSetActionBarTextPacket) {
        self.record("setActionBarText", packet);
    }

    fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket) {
        self.record("handleTabListCustomisation", packet);
    }

    fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket) {
        self.record("handleServerData", packet);
    }
}
