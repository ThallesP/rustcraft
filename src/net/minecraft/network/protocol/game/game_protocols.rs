//! net/minecraft/network/protocol/game/GameProtocols.java
//!
//! Not ported yet: `HAS_INFINITE_MATERIALS` and `dispatchCodec`, the codec modifiers of
//! `set_creative_mode_slot` and `command_suggestion`, which come with those packets; until then
//! the protocols have the `Unported` stand-in for them, like for every other packet whose class
//! isn't ported. The world data's packets (chunks, light, block updates, biomes) are the world
//! track's, which replaces their stand-ins. Both templates list every one of vanilla's packets, in vanilla's order, so the
//! ids match (`packets.json`).

use std::sync::LazyLock;

use crate::net::minecraft::network::connection_protocol::ConnectionProtocol;
use crate::net::minecraft::network::protocol::common::clientbound_clear_dialog_packet::ClientboundClearDialogPacket;
use crate::net::minecraft::network::protocol::common::clientbound_custom_report_details_packet::ClientboundCustomReportDetailsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_keep_alive_packet::ClientboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::clientbound_ping_packet::ClientboundPingPacket;
use crate::net::minecraft::network::protocol::common::clientbound_post_effects_packet::ClientboundPostEffectsPacket;
use crate::net::minecraft::network::protocol::common::clientbound_resource_pack_pop_packet::ClientboundResourcePackPopPacket;
use crate::net::minecraft::network::protocol::common::clientbound_store_cookie_packet::ClientboundStoreCookiePacket;
use crate::net::minecraft::network::protocol::common::clientbound_transfer_packet::ClientboundTransferPacket;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::common::serverbound_client_information_packet::ServerboundClientInformationPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_click_action_packet::ServerboundCustomClickActionPacket;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::common::serverbound_keep_alive_packet::ServerboundKeepAlivePacket;
use crate::net::minecraft::network::protocol::common::serverbound_pong_packet::ServerboundPongPacket;
use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::ServerboundResourcePackPacket;
use crate::net::minecraft::network::protocol::cookie::clientbound_cookie_request_packet::ClientboundCookieRequestPacket;
use crate::net::minecraft::network::protocol::cookie::cookie_packet_types::CookiePacketTypes;
use crate::net::minecraft::network::protocol::cookie::serverbound_cookie_response_packet::ServerboundCookieResponsePacket;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::clientbound_bundle_delimiter_packet::ClientboundBundleDelimiterPacket;
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
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_accept_teleportation_packet::ServerboundAcceptTeleportationPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chat_command_packet::ServerboundChatCommandPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chunk_batch_received_packet::ServerboundChunkBatchReceivedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::network::protocol::game::serverbound_configuration_acknowledged_packet::ServerboundConfigurationAcknowledgedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_move_player_packet;
use crate::net::minecraft::network::protocol::game::serverbound_player_loaded_packet::ServerboundPlayerLoadedPacket;
use crate::net::minecraft::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use crate::net::minecraft::network::protocol::ping::ping_packet_types::PingPacketTypes;
use crate::net::minecraft::network::protocol::ping::serverbound_ping_request_packet::ServerboundPingRequestPacket;
use crate::net::minecraft::network::protocol::protocol_info_builder::ProtocolInfoBuilder;
use crate::net::minecraft::network::protocol::simple_unbound_protocol::StaticSimpleUnboundProtocol;
use crate::net::minecraft::network::protocol::unbound_protocol::StaticUnboundProtocol;
use crate::net::minecraft::network::protocol::unported::Unported;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;

static SERVERBOUND_TEMPLATE: StaticUnboundProtocol<
    dyn ServerGamePacketListener,
    RegistryFriendlyByteBuf,
    Box<dyn Context>,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::context_serverbound_protocol(ConnectionProtocol::PLAY, |builder| {
        builder
            .add_packet(
                GamePacketTypes::SERVERBOUND_ACCEPT_TELEPORTATION,
                ServerboundAcceptTeleportationPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_ATTACK,
                Unported::codec(GamePacketTypes::SERVERBOUND_ATTACK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_BLOCK_ENTITY_TAG_QUERY,
                Unported::codec(GamePacketTypes::SERVERBOUND_BLOCK_ENTITY_TAG_QUERY),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_BUNDLE_ITEM_SELECTED,
                Unported::codec(GamePacketTypes::SERVERBOUND_BUNDLE_ITEM_SELECTED),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHANGE_DIFFICULTY,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHANGE_DIFFICULTY),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHANGE_GAME_MODE,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHANGE_GAME_MODE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHAT_ACK,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHAT_ACK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHAT_COMMAND,
                ServerboundChatCommandPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHAT_COMMAND_SIGNED,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHAT_COMMAND_SIGNED),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHAT,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHAT),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHAT_SESSION_UPDATE,
                Unported::codec(GamePacketTypes::SERVERBOUND_CHAT_SESSION_UPDATE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CHUNK_BATCH_RECEIVED,
                ServerboundChunkBatchReceivedPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CLIENT_COMMAND,
                Unported::codec(GamePacketTypes::SERVERBOUND_CLIENT_COMMAND),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CLIENT_TICK_END,
                ServerboundClientTickEndPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CLIENT_INFORMATION,
                ServerboundClientInformationPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_COMMAND_SUGGESTION,
                Unported::codec(GamePacketTypes::SERVERBOUND_COMMAND_SUGGESTION),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CONFIGURATION_ACKNOWLEDGED,
                ServerboundConfigurationAcknowledgedPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CONTAINER_BUTTON_CLICK,
                Unported::codec(GamePacketTypes::SERVERBOUND_CONTAINER_BUTTON_CLICK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CONTAINER_CLICK,
                Unported::codec(GamePacketTypes::SERVERBOUND_CONTAINER_CLICK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CONTAINER_CLOSE,
                Unported::codec(GamePacketTypes::SERVERBOUND_CONTAINER_CLOSE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED,
                Unported::codec(GamePacketTypes::SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED),
            )
            .add_packet(
                CookiePacketTypes::SERVERBOUND_COOKIE_RESPONSE,
                ServerboundCookieResponsePacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CUSTOM_PAYLOAD,
                ServerboundCustomPayloadPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST,
                Unported::codec(GamePacketTypes::SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_EDIT_BOOK,
                Unported::codec(GamePacketTypes::SERVERBOUND_EDIT_BOOK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_ENTITY_TAG_QUERY,
                Unported::codec(GamePacketTypes::SERVERBOUND_ENTITY_TAG_QUERY),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_INTERACT,
                Unported::codec(GamePacketTypes::SERVERBOUND_INTERACT),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_JIGSAW_GENERATE,
                Unported::codec(GamePacketTypes::SERVERBOUND_JIGSAW_GENERATE),
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_KEEP_ALIVE,
                ServerboundKeepAlivePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_LOCK_DIFFICULTY,
                Unported::codec(GamePacketTypes::SERVERBOUND_LOCK_DIFFICULTY),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_POS,
                serverbound_move_player_packet::Pos::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_POS_ROT,
                serverbound_move_player_packet::PosRot::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_ROT,
                serverbound_move_player_packet::Rot::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_STATUS_ONLY,
                serverbound_move_player_packet::StatusOnly::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_MOVE_VEHICLE,
                Unported::codec(GamePacketTypes::SERVERBOUND_MOVE_VEHICLE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PADDLE_BOAT,
                Unported::codec(GamePacketTypes::SERVERBOUND_PADDLE_BOAT),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_BLOCK,
                Unported::codec(GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_BLOCK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_ENTITY,
                Unported::codec(GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_ENTITY),
            )
            .add_packet(
                PingPacketTypes::SERVERBOUND_PING_REQUEST,
                ServerboundPingRequestPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLACE_RECIPE,
                Unported::codec(GamePacketTypes::SERVERBOUND_PLACE_RECIPE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLAYER_ABILITIES,
                Unported::codec(GamePacketTypes::SERVERBOUND_PLAYER_ABILITIES),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLAYER_ACTION,
                Unported::codec(GamePacketTypes::SERVERBOUND_PLAYER_ACTION),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLAYER_COMMAND,
                Unported::codec(GamePacketTypes::SERVERBOUND_PLAYER_COMMAND),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLAYER_INPUT,
                Unported::codec(GamePacketTypes::SERVERBOUND_PLAYER_INPUT),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PLAYER_LOADED,
                ServerboundPlayerLoadedPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_PONG,
                ServerboundPongPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_PUNCH,
                Unported::codec(GamePacketTypes::SERVERBOUND_PUNCH),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS,
                Unported::codec(GamePacketTypes::SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE,
                Unported::codec(GamePacketTypes::SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_RENAME_ITEM,
                Unported::codec(GamePacketTypes::SERVERBOUND_RENAME_ITEM),
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_RESOURCE_PACK,
                ServerboundResourcePackPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SEEN_ADVANCEMENTS,
                Unported::codec(GamePacketTypes::SERVERBOUND_SEEN_ADVANCEMENTS),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SELECT_TRADE,
                Unported::codec(GamePacketTypes::SERVERBOUND_SELECT_TRADE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_BEACON,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_BEACON),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_CARRIED_ITEM,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_CARRIED_ITEM),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_COMMAND_BLOCK,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_COMMAND_BLOCK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_COMMAND_MINECART,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_COMMAND_MINECART),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_CREATIVE_MODE_SLOT,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_CREATIVE_MODE_SLOT),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_GAME_RULE,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_GAME_RULE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_JIGSAW_BLOCK,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_JIGSAW_BLOCK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_STRUCTURE_BLOCK,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_STRUCTURE_BLOCK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SET_TEST_BLOCK,
                Unported::codec(GamePacketTypes::SERVERBOUND_SET_TEST_BLOCK),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SIGN_UPDATE,
                Unported::codec(GamePacketTypes::SERVERBOUND_SIGN_UPDATE),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_SPECTATOR_ACTION,
                Unported::codec(GamePacketTypes::SERVERBOUND_SPECTATOR_ACTION),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_TELEPORT_TO_ENTITY,
                Unported::codec(GamePacketTypes::SERVERBOUND_TELEPORT_TO_ENTITY),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION,
                Unported::codec(GamePacketTypes::SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_USE_ITEM_ON,
                Unported::codec(GamePacketTypes::SERVERBOUND_USE_ITEM_ON),
            )
            .add_packet(
                GamePacketTypes::SERVERBOUND_USE_ITEM,
                Unported::codec(GamePacketTypes::SERVERBOUND_USE_ITEM),
            )
            .add_packet(
                CommonPacketTypes::SERVERBOUND_CUSTOM_CLICK_ACTION,
                ServerboundCustomClickActionPacket::STREAM_CODEC,
            );
    })
});

static CLIENTBOUND_TEMPLATE: StaticSimpleUnboundProtocol<
    dyn ClientGamePacketListener,
    RegistryFriendlyByteBuf,
> = LazyLock::new(|| {
    ProtocolInfoBuilder::clientbound_protocol(ConnectionProtocol::PLAY, |builder| {
        builder
            .with_bundle_packet(
                GamePacketTypes::CLIENTBOUND_BUNDLE,
                ClientboundBundlePacket::new,
                ClientboundBundleDelimiterPacket,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ADD_ENTITY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ADD_ENTITY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ANIMATE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ANIMATE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_AWARD_STATS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_AWARD_STATS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BLOCK_CHANGED_ACK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BLOCK_CHANGED_ACK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BLOCK_DESTRUCTION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BLOCK_DESTRUCTION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BLOCK_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BLOCK_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BLOCK_UPDATE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BLOCK_UPDATE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_BOSS_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_BOSS_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CHANGE_DIFFICULTY,
                ClientboundChangeDifficultyPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED,
                ClientboundChunkBatchFinishedPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START,
                ClientboundChunkBatchStartPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CHUNKS_BIOMES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CHUNKS_BIOMES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CLEAR_TITLES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CLEAR_TITLES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_COMMAND_SUGGESTIONS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_COMMAND_SUGGESTIONS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_COMMANDS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_COMMANDS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CONTAINER_CLOSE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CONTAINER_CLOSE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_CONTENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CONTAINER_SET_CONTENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_DATA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CONTAINER_SET_DATA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_SLOT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CONTAINER_SET_SLOT),
            )
            .add_packet(
                CookiePacketTypes::CLIENTBOUND_COOKIE_REQUEST,
                ClientboundCookieRequestPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_COOLDOWN,
                Unported::codec(GamePacketTypes::CLIENTBOUND_COOLDOWN),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CUSTOM_PAYLOAD,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_CUSTOM_PAYLOAD),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DAMAGE_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DAMAGE_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DEBUG_BLOCK_VALUE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DEBUG_BLOCK_VALUE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DEBUG_CHUNK_VALUE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DEBUG_CHUNK_VALUE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DEBUG_ENTITY_VALUE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DEBUG_ENTITY_VALUE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DEBUG_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DEBUG_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DEBUG_SAMPLE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DEBUG_SAMPLE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DELETE_CHAT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DELETE_CHAT),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_DISCONNECT,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_DISCONNECT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_DISGUISED_CHAT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_DISGUISED_CHAT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ENTITY_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ENTITY_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ENTITY_POSITION_SYNC,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ENTITY_POSITION_SYNC),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_EXPLODE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_EXPLODE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ADD_TRANSIENT_BLOCK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ADD_TRANSIENT_BLOCK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_FORGET_LEVEL_CHUNK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_FORGET_LEVEL_CHUNK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_GAME_EVENT,
                ClientboundGameEventPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_GAME_RULE_VALUES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_GAME_RULE_VALUES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOUNT_SCREEN_OPEN,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOUNT_SCREEN_OPEN),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_HURT_ANIMATION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_HURT_ANIMATION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_INITIALIZE_BORDER,
                ClientboundInitializeBorderPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_KEEP_ALIVE,
                ClientboundKeepAlivePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LEVEL_EVENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_LEVEL_EVENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LEVEL_PARTICLES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_LEVEL_PARTICLES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LIGHT_UPDATE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_LIGHT_UPDATE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LOGIN,
                ClientboundLoginPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_LOW_DISK_SPACE_WARNING,
                Unported::codec(GamePacketTypes::CLIENTBOUND_LOW_DISK_SPACE_WARNING),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MAP_ITEM_DATA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MAP_ITEM_DATA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MERCHANT_OFFERS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MERCHANT_OFFERS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS_ROT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS_ROT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOVE_MINECART_ALONG_TRACK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOVE_MINECART_ALONG_TRACK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_ROT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_ROT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_MOVE_VEHICLE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_MOVE_VEHICLE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_OPEN_BOOK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_OPEN_BOOK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_OPEN_SCREEN,
                Unported::codec(GamePacketTypes::CLIENTBOUND_OPEN_SCREEN),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_OPEN_SIGN_EDITOR,
                Unported::codec(GamePacketTypes::CLIENTBOUND_OPEN_SIGN_EDITOR),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_PING,
                ClientboundPingPacket::STREAM_CODEC,
            )
            .add_packet(
                PingPacketTypes::CLIENTBOUND_PONG_RESPONSE,
                ClientboundPongResponsePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLACE_GHOST_RECIPE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLACE_GHOST_RECIPE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_ABILITIES,
                ClientboundPlayerAbilitiesPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_CHAT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_CHAT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_END,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_END),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_ENTER,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_ENTER),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_KILL,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_KILL),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_INFO_REMOVE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_INFO_REMOVE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_INFO_UPDATE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_INFO_UPDATE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_LOOK_AT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_LOOK_AT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_POSITION,
                ClientboundPlayerPositionPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PLAYER_ROTATION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PLAYER_ROTATION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_ADD,
                Unported::codec(GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_ADD),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_REMOVE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_REMOVE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_REMOVE_ENTITIES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_REMOVE_ENTITIES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_REMOVE_MOB_EFFECT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_REMOVE_MOB_EFFECT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_RESET_SCORE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_RESET_SCORE),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_POP,
                ClientboundResourcePackPopPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_RESOURCE_PACK_PUSH),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_POST_EFFECTS,
                ClientboundPostEffectsPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_RESPAWN,
                Unported::codec(GamePacketTypes::CLIENTBOUND_RESPAWN),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_ROTATE_HEAD,
                Unported::codec(GamePacketTypes::CLIENTBOUND_ROTATE_HEAD),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SECTION_BLOCKS_UPDATE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SECTION_BLOCKS_UPDATE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SELECT_ADVANCEMENTS_TAB,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SELECT_ADVANCEMENTS_TAB),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SERVER_DATA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SERVER_DATA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_ACTION_BAR_TEXT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_ACTION_BAR_TEXT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_BORDER_CENTER,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_BORDER_CENTER),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_BORDER_LERP_SIZE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_BORDER_LERP_SIZE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_BORDER_SIZE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_BORDER_SIZE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DELAY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DELAY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DISTANCE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DISTANCE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_CAMERA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_CAMERA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_CENTER,
                ClientboundSetChunkCacheCenterPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_RADIUS,
                ClientboundSetChunkCacheRadiusPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_CURSOR_ITEM,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_CURSOR_ITEM),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_DISPLAY_OBJECTIVE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_DISPLAY_OBJECTIVE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_DATA,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_ENTITY_DATA),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_LINK,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_ENTITY_LINK),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_MOTION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_ENTITY_MOTION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_EQUIPMENT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_EQUIPMENT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_EXPERIENCE,
                ClientboundSetExperiencePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_HEALTH,
                ClientboundSetHealthPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_HELD_SLOT,
                ClientboundSetHeldSlotPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_OBJECTIVE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_OBJECTIVE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_PASSENGERS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_PASSENGERS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_PLAYER_INVENTORY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_PLAYER_INVENTORY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_PLAYER_TEAM,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_PLAYER_TEAM),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_SCORE,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_SCORE),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_SIMULATION_DISTANCE,
                ClientboundSetSimulationDistancePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_TIME,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_TIME),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_TITLE_TEXT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_TITLE_TEXT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SET_TITLES_ANIMATION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SET_TITLES_ANIMATION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SOUND_ENTITY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SOUND_ENTITY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SOUND,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SOUND),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_START_CONFIGURATION,
                ClientboundStartConfigurationPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_STOP_SOUND,
                Unported::codec(GamePacketTypes::CLIENTBOUND_STOP_SOUND),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_STORE_COOKIE,
                ClientboundStoreCookiePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SWING_ANIMATION,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SWING_ANIMATION),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_SYSTEM_CHAT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_SYSTEM_CHAT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TAB_LIST,
                Unported::codec(GamePacketTypes::CLIENTBOUND_TAB_LIST),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TAG_QUERY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_TAG_QUERY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TAKE_ITEM_ENTITY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_TAKE_ITEM_ENTITY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TELEPORT_ENTITY,
                Unported::codec(GamePacketTypes::CLIENTBOUND_TELEPORT_ENTITY),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TICKING_STATE,
                ClientboundTickingStatePacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_TICKING_STEP,
                ClientboundTickingStepPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_TRANSFER,
                ClientboundTransferPacket::STREAM_CODEC,
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_UPDATE_ADVANCEMENTS,
                Unported::codec(GamePacketTypes::CLIENTBOUND_UPDATE_ADVANCEMENTS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_UPDATE_ATTRIBUTES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_UPDATE_ATTRIBUTES),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_UPDATE_MOB_EFFECT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_UPDATE_MOB_EFFECT),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_UPDATE_RECIPES,
                Unported::codec(GamePacketTypes::CLIENTBOUND_UPDATE_RECIPES),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_UPDATE_TAGS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_PROJECTILE_POWER,
                Unported::codec(GamePacketTypes::CLIENTBOUND_PROJECTILE_POWER),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CUSTOM_REPORT_DETAILS,
                ClientboundCustomReportDetailsPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_SERVER_LINKS,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_SERVER_LINKS),
            )
            .add_packet(
                GamePacketTypes::CLIENTBOUND_WAYPOINT,
                Unported::codec(GamePacketTypes::CLIENTBOUND_WAYPOINT),
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_CLEAR_DIALOG,
                ClientboundClearDialogPacket::STREAM_CODEC,
            )
            .add_packet(
                CommonPacketTypes::CLIENTBOUND_SHOW_DIALOG,
                Unported::codec(CommonPacketTypes::CLIENTBOUND_SHOW_DIALOG),
            );
    })
});

/// The game protocol, which a connection binds to the registries the server sent, once
/// configuration is over.
pub struct GameProtocols;

impl GameProtocols {
    pub const SERVERBOUND_TEMPLATE: &'static StaticUnboundProtocol<
        dyn ServerGamePacketListener,
        RegistryFriendlyByteBuf,
        Box<dyn Context>,
    > = &SERVERBOUND_TEMPLATE;
    pub const CLIENTBOUND_TEMPLATE: &'static StaticSimpleUnboundProtocol<
        dyn ClientGamePacketListener,
        RegistryFriendlyByteBuf,
    > = &CLIENTBOUND_TEMPLATE;
}

/// `GameProtocols.Context`: what the serverbound codec modifiers ask about the player.
pub trait Context: Send + Sync {
    fn has_infinite_materials(&self) -> bool;

    fn can_use_command_blocks(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::GameProtocols;
    use crate::golden::Golden;
    use crate::net::minecraft::network::protocol_info::tests::listed;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.GameProtocols");
        golden.pending(
            "HAS_INFINITE_MATERIALS",
            "a codec modifier, which comes with set_creative_mode_slot",
        );
        golden.field(
            "SERVERBOUND_TEMPLATE",
            listed(GameProtocols::SERVERBOUND_TEMPLATE.details()),
        );
        golden.field(
            "CLIENTBOUND_TEMPLATE",
            listed(GameProtocols::CLIENTBOUND_TEMPLATE.details()),
        );
        golden.finish();
    }
}
