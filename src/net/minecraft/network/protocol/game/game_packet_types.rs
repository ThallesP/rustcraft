//! net/minecraft/network/protocol/game/GamePacketTypes.java

use std::sync::LazyLock;

use crate::net::minecraft::network::protocol::packet_flow::PacketFlow;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::resources::identifier::Identifier;

static CLIENTBOUND_BUNDLE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("bundle"));
static CLIENTBOUND_BUNDLE_DELIMITER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("bundle_delimiter"));
static CLIENTBOUND_ADD_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("add_entity"));
static CLIENTBOUND_ANIMATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("animate"));
static CLIENTBOUND_AWARD_STATS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("award_stats"));
static CLIENTBOUND_BLOCK_CHANGED_ACK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("block_changed_ack"));
static CLIENTBOUND_BLOCK_DESTRUCTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("block_destruction"));
static CLIENTBOUND_BLOCK_ENTITY_DATA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("block_entity_data"));
static CLIENTBOUND_BLOCK_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("block_event"));
static CLIENTBOUND_BLOCK_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("block_update"));
static CLIENTBOUND_BOSS_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("boss_event"));
static CLIENTBOUND_CHANGE_DIFFICULTY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("change_difficulty"));
static CLIENTBOUND_CHUNK_BATCH_FINISHED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("chunk_batch_finished"));
static CLIENTBOUND_CHUNK_BATCH_START: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("chunk_batch_start"));
static CLIENTBOUND_CHUNKS_BIOMES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("chunks_biomes"));
static CLIENTBOUND_CLEAR_TITLES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("clear_titles"));
static CLIENTBOUND_COMMAND_SUGGESTIONS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("command_suggestions"));
static CLIENTBOUND_COMMANDS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("commands"));
static CLIENTBOUND_CONTAINER_CLOSE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("container_close"));
static CLIENTBOUND_CONTAINER_SET_CONTENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("container_set_content"));
static CLIENTBOUND_CONTAINER_SET_DATA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("container_set_data"));
static CLIENTBOUND_CONTAINER_SET_SLOT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("container_set_slot"));
static CLIENTBOUND_COOLDOWN: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("cooldown"));
static CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("custom_chat_completions"));
static CLIENTBOUND_DAMAGE_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("damage_event"));
static CLIENTBOUND_DEBUG_BLOCK_VALUE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("debug/block_value"));
static CLIENTBOUND_DEBUG_CHUNK_VALUE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("debug/chunk_value"));
static CLIENTBOUND_DEBUG_ENTITY_VALUE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("debug/entity_value"));
static CLIENTBOUND_DEBUG_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("debug/event"));
static CLIENTBOUND_DEBUG_SAMPLE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("debug_sample"));
static CLIENTBOUND_DELETE_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("delete_chat"));
static CLIENTBOUND_DISGUISED_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("disguised_chat"));
static CLIENTBOUND_ENTITY_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("entity_event"));
static CLIENTBOUND_ENTITY_POSITION_SYNC: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("entity_position_sync"));
static CLIENTBOUND_EXPLODE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("explode"));
static CLIENTBOUND_ADD_TRANSIENT_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("add_transient_block"));
static CLIENTBOUND_FORGET_LEVEL_CHUNK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("forget_level_chunk"));
static CLIENTBOUND_GAME_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("game_event"));
static CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("game_test_highlight_pos"));
static CLIENTBOUND_MOUNT_SCREEN_OPEN: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("mount_screen_open"));
static CLIENTBOUND_HURT_ANIMATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("hurt_animation"));
static CLIENTBOUND_INITIALIZE_BORDER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("initialize_border"));
static CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("level_chunk_with_light"));
static CLIENTBOUND_LEVEL_EVENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("level_event"));
static CLIENTBOUND_LEVEL_PARTICLES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("level_particles"));
static CLIENTBOUND_LIGHT_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("light_update"));
static CLIENTBOUND_LOGIN: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("login"));
static CLIENTBOUND_LOW_DISK_SPACE_WARNING: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("low_disk_space_warning"));
static CLIENTBOUND_MAP_ITEM_DATA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("map_item_data"));
static CLIENTBOUND_MERCHANT_OFFERS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("merchant_offers"));
static CLIENTBOUND_MOVE_ENTITY_POS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("move_entity_pos"));
static CLIENTBOUND_MOVE_ENTITY_POS_ROT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("move_entity_pos_rot"));
static CLIENTBOUND_MOVE_MINECART_ALONG_TRACK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("move_minecart_along_track"));
static CLIENTBOUND_MOVE_ENTITY_ROT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("move_entity_rot"));
static CLIENTBOUND_MOVE_VEHICLE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("move_vehicle"));
static CLIENTBOUND_OPEN_BOOK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("open_book"));
static CLIENTBOUND_OPEN_SCREEN: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("open_screen"));
static CLIENTBOUND_OPEN_SIGN_EDITOR: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("open_sign_editor"));
static CLIENTBOUND_PLACE_GHOST_RECIPE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("place_ghost_recipe"));
static CLIENTBOUND_PLAYER_ABILITIES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_abilities"));
static CLIENTBOUND_GAME_RULE_VALUES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("game_rule_values"));
static CLIENTBOUND_PLAYER_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_chat"));
static CLIENTBOUND_PLAYER_COMBAT_END: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_combat_end"));
static CLIENTBOUND_PLAYER_COMBAT_ENTER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_combat_enter"));
static CLIENTBOUND_PLAYER_COMBAT_KILL: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_combat_kill"));
static CLIENTBOUND_PLAYER_INFO_REMOVE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_info_remove"));
static CLIENTBOUND_PLAYER_INFO_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_info_update"));
static CLIENTBOUND_PLAYER_LOOK_AT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_look_at"));
static CLIENTBOUND_PLAYER_POSITION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_position"));
static CLIENTBOUND_PLAYER_ROTATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("player_rotation"));
static CLIENTBOUND_RECIPE_BOOK_ADD: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("recipe_book_add"));
static CLIENTBOUND_RECIPE_BOOK_REMOVE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("recipe_book_remove"));
static CLIENTBOUND_RECIPE_BOOK_SETTINGS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("recipe_book_settings"));
static CLIENTBOUND_REMOVE_ENTITIES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("remove_entities"));
static CLIENTBOUND_REMOVE_MOB_EFFECT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("remove_mob_effect"));
static CLIENTBOUND_RESPAWN: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("respawn"));
static CLIENTBOUND_ROTATE_HEAD: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("rotate_head"));
static CLIENTBOUND_SECTION_BLOCKS_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("section_blocks_update"));
static CLIENTBOUND_SELECT_ADVANCEMENTS_TAB: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("select_advancements_tab"));
static CLIENTBOUND_SERVER_DATA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("server_data"));
static CLIENTBOUND_SET_ACTION_BAR_TEXT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_action_bar_text"));
static CLIENTBOUND_SET_BORDER_CENTER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_border_center"));
static CLIENTBOUND_SET_BORDER_LERP_SIZE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_border_lerp_size"));
static CLIENTBOUND_SET_BORDER_SIZE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_border_size"));
static CLIENTBOUND_SET_BORDER_WARNING_DELAY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_border_warning_delay"));
static CLIENTBOUND_SET_BORDER_WARNING_DISTANCE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_border_warning_distance"));
static CLIENTBOUND_SET_CAMERA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_camera"));
static CLIENTBOUND_SET_CHUNK_CACHE_CENTER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_chunk_cache_center"));
static CLIENTBOUND_SET_CHUNK_CACHE_RADIUS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_chunk_cache_radius"));
static CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_default_spawn_position"));
static CLIENTBOUND_SET_DISPLAY_OBJECTIVE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_display_objective"));
static CLIENTBOUND_SET_ENTITY_DATA: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_entity_data"));
static CLIENTBOUND_SET_ENTITY_LINK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_entity_link"));
static CLIENTBOUND_SET_ENTITY_MOTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_entity_motion"));
static CLIENTBOUND_SET_EQUIPMENT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_equipment"));
static CLIENTBOUND_SET_EXPERIENCE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_experience"));
static CLIENTBOUND_SET_HEALTH: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_health"));
static CLIENTBOUND_SET_HELD_SLOT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_held_slot"));
static CLIENTBOUND_SET_OBJECTIVE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_objective"));
static CLIENTBOUND_SET_PASSENGERS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_passengers"));
static CLIENTBOUND_SET_PLAYER_TEAM: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_player_team"));
static CLIENTBOUND_SET_SCORE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_score"));
static CLIENTBOUND_SET_SIMULATION_DISTANCE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_simulation_distance"));
static CLIENTBOUND_SET_SUBTITLE_TEXT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_subtitle_text"));
static CLIENTBOUND_SET_TIME: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_time"));
static CLIENTBOUND_SET_TITLE_TEXT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_title_text"));
static CLIENTBOUND_SET_TITLES_ANIMATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_titles_animation"));
static CLIENTBOUND_SOUND_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("sound_entity"));
static CLIENTBOUND_SOUND: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("sound"));
static CLIENTBOUND_START_CONFIGURATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("start_configuration"));
static CLIENTBOUND_STOP_SOUND: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("stop_sound"));
static CLIENTBOUND_SWING_ANIMATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("swing_animation"));
static CLIENTBOUND_SYSTEM_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("system_chat"));
static CLIENTBOUND_TAB_LIST: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("tab_list"));
static CLIENTBOUND_TAG_QUERY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("tag_query"));
static CLIENTBOUND_TAKE_ITEM_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("take_item_entity"));
static CLIENTBOUND_TELEPORT_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("teleport_entity"));
static CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("test_instance_block_status"));
static CLIENTBOUND_UPDATE_ADVANCEMENTS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("update_advancements"));
static CLIENTBOUND_UPDATE_ATTRIBUTES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("update_attributes"));
static CLIENTBOUND_UPDATE_MOB_EFFECT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("update_mob_effect"));
static CLIENTBOUND_UPDATE_RECIPES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("update_recipes"));
static CLIENTBOUND_PROJECTILE_POWER: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("projectile_power"));
static CLIENTBOUND_WAYPOINT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("waypoint"));
static SERVERBOUND_ACCEPT_TELEPORTATION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("accept_teleportation"));
static SERVERBOUND_ATTACK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("attack"));
static SERVERBOUND_BLOCK_ENTITY_TAG_QUERY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("block_entity_tag_query"));
static SERVERBOUND_BUNDLE_ITEM_SELECTED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("bundle_item_selected"));
static SERVERBOUND_CHANGE_DIFFICULTY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("change_difficulty"));
static SERVERBOUND_CHANGE_GAME_MODE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("change_game_mode"));
static SERVERBOUND_CHAT_ACK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chat_ack"));
static SERVERBOUND_CHAT_COMMAND: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chat_command"));
static SERVERBOUND_CHAT_COMMAND_SIGNED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chat_command_signed"));
static SERVERBOUND_CHAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chat"));
static SERVERBOUND_CHAT_SESSION_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chat_session_update"));
static SERVERBOUND_CHUNK_BATCH_RECEIVED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("chunk_batch_received"));
static SERVERBOUND_CLIENT_COMMAND: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("client_command"));
static SERVERBOUND_CLIENT_TICK_END: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("client_tick_end"));
static SERVERBOUND_COMMAND_SUGGESTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("command_suggestion"));
static SERVERBOUND_CONFIGURATION_ACKNOWLEDGED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("configuration_acknowledged"));
static SERVERBOUND_CONTAINER_BUTTON_CLICK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("container_button_click"));
static SERVERBOUND_CONTAINER_CLICK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("container_click"));
static SERVERBOUND_CONTAINER_CLOSE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("container_close"));
static SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("container_slot_state_changed"));
static SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("debug_subscription_request"));
static SERVERBOUND_EDIT_BOOK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("edit_book"));
static SERVERBOUND_ENTITY_TAG_QUERY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("entity_tag_query"));
static SERVERBOUND_INTERACT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("interact"));
static SERVERBOUND_JIGSAW_GENERATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("jigsaw_generate"));
static SERVERBOUND_LOCK_DIFFICULTY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("lock_difficulty"));
static SERVERBOUND_MOVE_PLAYER_POS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("move_player_pos"));
static SERVERBOUND_MOVE_PLAYER_POS_ROT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("move_player_pos_rot"));
static SERVERBOUND_MOVE_PLAYER_ROT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("move_player_rot"));
static SERVERBOUND_MOVE_PLAYER_STATUS_ONLY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("move_player_status_only"));
static SERVERBOUND_MOVE_VEHICLE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("move_vehicle"));
static SERVERBOUND_PADDLE_BOAT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("paddle_boat"));
static SERVERBOUND_PICK_ITEM_FROM_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("pick_item_from_block"));
static SERVERBOUND_PICK_ITEM_FROM_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("pick_item_from_entity"));
static SERVERBOUND_PLACE_RECIPE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("place_recipe"));
static SERVERBOUND_PLAYER_ABILITIES: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("player_abilities"));
static SERVERBOUND_PLAYER_ACTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("player_action"));
static SERVERBOUND_PLAYER_COMMAND: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("player_command"));
static SERVERBOUND_PLAYER_INPUT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("player_input"));
static SERVERBOUND_PLAYER_LOADED: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("player_loaded"));
static SERVERBOUND_PUNCH: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("punch"));
static SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("recipe_book_change_settings"));
static SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("recipe_book_seen_recipe"));
static SERVERBOUND_RENAME_ITEM: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("rename_item"));
static SERVERBOUND_SEEN_ADVANCEMENTS: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("seen_advancements"));
static SERVERBOUND_SELECT_TRADE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("select_trade"));
static SERVERBOUND_SET_BEACON: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_beacon"));
static SERVERBOUND_SET_CARRIED_ITEM: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_carried_item"));
static SERVERBOUND_SET_COMMAND_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_command_block"));
static SERVERBOUND_SET_COMMAND_MINECART: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_command_minecart"));
static SERVERBOUND_SET_CREATIVE_MODE_SLOT: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_creative_mode_slot"));
static SERVERBOUND_SET_GAME_RULE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_game_rule"));
static SERVERBOUND_SET_JIGSAW_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_jigsaw_block"));
static SERVERBOUND_SET_STRUCTURE_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_structure_block"));
static SERVERBOUND_SET_TEST_BLOCK: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("set_test_block"));
static SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("test_instance_block_action"));
static SERVERBOUND_SIGN_UPDATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("sign_update"));
static SERVERBOUND_SPECTATOR_ACTION: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("spectator_action"));
static SERVERBOUND_TELEPORT_TO_ENTITY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("teleport_to_entity"));
static SERVERBOUND_USE_ITEM_ON: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("use_item_on"));
static SERVERBOUND_USE_ITEM: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_serverbound("use_item"));
static CLIENTBOUND_RESET_SCORE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("reset_score"));
static CLIENTBOUND_TICKING_STATE: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("ticking_state"));
static CLIENTBOUND_TICKING_STEP: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("ticking_step"));
static CLIENTBOUND_SET_CURSOR_ITEM: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_cursor_item"));
static CLIENTBOUND_SET_PLAYER_INVENTORY: LazyLock<PacketType> =
    LazyLock::new(|| GamePacketTypes::create_clientbound("set_player_inventory"));

/// The types of the packets only the game protocol has. The ones it shares with configuration
/// are `CommonPacketTypes`', `CookiePacketTypes`' and `PingPacketTypes`'.
pub struct GamePacketTypes;

impl GamePacketTypes {
    pub const CLIENTBOUND_BUNDLE: &'static LazyLock<PacketType> = &CLIENTBOUND_BUNDLE;
    pub const CLIENTBOUND_BUNDLE_DELIMITER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_BUNDLE_DELIMITER;
    pub const CLIENTBOUND_ADD_ENTITY: &'static LazyLock<PacketType> = &CLIENTBOUND_ADD_ENTITY;
    pub const CLIENTBOUND_ANIMATE: &'static LazyLock<PacketType> = &CLIENTBOUND_ANIMATE;
    pub const CLIENTBOUND_AWARD_STATS: &'static LazyLock<PacketType> = &CLIENTBOUND_AWARD_STATS;
    pub const CLIENTBOUND_BLOCK_CHANGED_ACK: &'static LazyLock<PacketType> =
        &CLIENTBOUND_BLOCK_CHANGED_ACK;
    pub const CLIENTBOUND_BLOCK_DESTRUCTION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_BLOCK_DESTRUCTION;
    pub const CLIENTBOUND_BLOCK_ENTITY_DATA: &'static LazyLock<PacketType> =
        &CLIENTBOUND_BLOCK_ENTITY_DATA;
    pub const CLIENTBOUND_BLOCK_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_BLOCK_EVENT;
    pub const CLIENTBOUND_BLOCK_UPDATE: &'static LazyLock<PacketType> = &CLIENTBOUND_BLOCK_UPDATE;
    pub const CLIENTBOUND_BOSS_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_BOSS_EVENT;
    pub const CLIENTBOUND_CHANGE_DIFFICULTY: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CHANGE_DIFFICULTY;
    pub const CLIENTBOUND_CHUNK_BATCH_FINISHED: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CHUNK_BATCH_FINISHED;
    pub const CLIENTBOUND_CHUNK_BATCH_START: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CHUNK_BATCH_START;
    pub const CLIENTBOUND_CHUNKS_BIOMES: &'static LazyLock<PacketType> = &CLIENTBOUND_CHUNKS_BIOMES;
    pub const CLIENTBOUND_CLEAR_TITLES: &'static LazyLock<PacketType> = &CLIENTBOUND_CLEAR_TITLES;
    pub const CLIENTBOUND_COMMAND_SUGGESTIONS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_COMMAND_SUGGESTIONS;
    pub const CLIENTBOUND_COMMANDS: &'static LazyLock<PacketType> = &CLIENTBOUND_COMMANDS;
    pub const CLIENTBOUND_CONTAINER_CLOSE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CONTAINER_CLOSE;
    pub const CLIENTBOUND_CONTAINER_SET_CONTENT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CONTAINER_SET_CONTENT;
    pub const CLIENTBOUND_CONTAINER_SET_DATA: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CONTAINER_SET_DATA;
    pub const CLIENTBOUND_CONTAINER_SET_SLOT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CONTAINER_SET_SLOT;
    pub const CLIENTBOUND_COOLDOWN: &'static LazyLock<PacketType> = &CLIENTBOUND_COOLDOWN;
    pub const CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS;
    pub const CLIENTBOUND_DAMAGE_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_DAMAGE_EVENT;
    pub const CLIENTBOUND_DEBUG_BLOCK_VALUE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_DEBUG_BLOCK_VALUE;
    pub const CLIENTBOUND_DEBUG_CHUNK_VALUE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_DEBUG_CHUNK_VALUE;
    pub const CLIENTBOUND_DEBUG_ENTITY_VALUE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_DEBUG_ENTITY_VALUE;
    pub const CLIENTBOUND_DEBUG_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_DEBUG_EVENT;
    pub const CLIENTBOUND_DEBUG_SAMPLE: &'static LazyLock<PacketType> = &CLIENTBOUND_DEBUG_SAMPLE;
    pub const CLIENTBOUND_DELETE_CHAT: &'static LazyLock<PacketType> = &CLIENTBOUND_DELETE_CHAT;
    pub const CLIENTBOUND_DISGUISED_CHAT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_DISGUISED_CHAT;
    pub const CLIENTBOUND_ENTITY_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_ENTITY_EVENT;
    pub const CLIENTBOUND_ENTITY_POSITION_SYNC: &'static LazyLock<PacketType> =
        &CLIENTBOUND_ENTITY_POSITION_SYNC;
    pub const CLIENTBOUND_EXPLODE: &'static LazyLock<PacketType> = &CLIENTBOUND_EXPLODE;
    pub const CLIENTBOUND_ADD_TRANSIENT_BLOCK: &'static LazyLock<PacketType> =
        &CLIENTBOUND_ADD_TRANSIENT_BLOCK;
    pub const CLIENTBOUND_FORGET_LEVEL_CHUNK: &'static LazyLock<PacketType> =
        &CLIENTBOUND_FORGET_LEVEL_CHUNK;
    pub const CLIENTBOUND_GAME_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_GAME_EVENT;
    pub const CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS;
    pub const CLIENTBOUND_MOUNT_SCREEN_OPEN: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MOUNT_SCREEN_OPEN;
    pub const CLIENTBOUND_HURT_ANIMATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_HURT_ANIMATION;
    pub const CLIENTBOUND_INITIALIZE_BORDER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_INITIALIZE_BORDER;
    pub const CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT;
    pub const CLIENTBOUND_LEVEL_EVENT: &'static LazyLock<PacketType> = &CLIENTBOUND_LEVEL_EVENT;
    pub const CLIENTBOUND_LEVEL_PARTICLES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LEVEL_PARTICLES;
    pub const CLIENTBOUND_LIGHT_UPDATE: &'static LazyLock<PacketType> = &CLIENTBOUND_LIGHT_UPDATE;
    pub const CLIENTBOUND_LOGIN: &'static LazyLock<PacketType> = &CLIENTBOUND_LOGIN;
    pub const CLIENTBOUND_LOW_DISK_SPACE_WARNING: &'static LazyLock<PacketType> =
        &CLIENTBOUND_LOW_DISK_SPACE_WARNING;
    pub const CLIENTBOUND_MAP_ITEM_DATA: &'static LazyLock<PacketType> = &CLIENTBOUND_MAP_ITEM_DATA;
    pub const CLIENTBOUND_MERCHANT_OFFERS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MERCHANT_OFFERS;
    pub const CLIENTBOUND_MOVE_ENTITY_POS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MOVE_ENTITY_POS;
    pub const CLIENTBOUND_MOVE_ENTITY_POS_ROT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MOVE_ENTITY_POS_ROT;
    pub const CLIENTBOUND_MOVE_MINECART_ALONG_TRACK: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MOVE_MINECART_ALONG_TRACK;
    pub const CLIENTBOUND_MOVE_ENTITY_ROT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_MOVE_ENTITY_ROT;
    pub const CLIENTBOUND_MOVE_VEHICLE: &'static LazyLock<PacketType> = &CLIENTBOUND_MOVE_VEHICLE;
    pub const CLIENTBOUND_OPEN_BOOK: &'static LazyLock<PacketType> = &CLIENTBOUND_OPEN_BOOK;
    pub const CLIENTBOUND_OPEN_SCREEN: &'static LazyLock<PacketType> = &CLIENTBOUND_OPEN_SCREEN;
    pub const CLIENTBOUND_OPEN_SIGN_EDITOR: &'static LazyLock<PacketType> =
        &CLIENTBOUND_OPEN_SIGN_EDITOR;
    pub const CLIENTBOUND_PLACE_GHOST_RECIPE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLACE_GHOST_RECIPE;
    pub const CLIENTBOUND_PLAYER_ABILITIES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_ABILITIES;
    pub const CLIENTBOUND_GAME_RULE_VALUES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_GAME_RULE_VALUES;
    pub const CLIENTBOUND_PLAYER_CHAT: &'static LazyLock<PacketType> = &CLIENTBOUND_PLAYER_CHAT;
    pub const CLIENTBOUND_PLAYER_COMBAT_END: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_COMBAT_END;
    pub const CLIENTBOUND_PLAYER_COMBAT_ENTER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_COMBAT_ENTER;
    pub const CLIENTBOUND_PLAYER_COMBAT_KILL: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_COMBAT_KILL;
    pub const CLIENTBOUND_PLAYER_INFO_REMOVE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_INFO_REMOVE;
    pub const CLIENTBOUND_PLAYER_INFO_UPDATE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_INFO_UPDATE;
    pub const CLIENTBOUND_PLAYER_LOOK_AT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_LOOK_AT;
    pub const CLIENTBOUND_PLAYER_POSITION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_POSITION;
    pub const CLIENTBOUND_PLAYER_ROTATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PLAYER_ROTATION;
    pub const CLIENTBOUND_RECIPE_BOOK_ADD: &'static LazyLock<PacketType> =
        &CLIENTBOUND_RECIPE_BOOK_ADD;
    pub const CLIENTBOUND_RECIPE_BOOK_REMOVE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_RECIPE_BOOK_REMOVE;
    pub const CLIENTBOUND_RECIPE_BOOK_SETTINGS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_RECIPE_BOOK_SETTINGS;
    pub const CLIENTBOUND_REMOVE_ENTITIES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_REMOVE_ENTITIES;
    pub const CLIENTBOUND_REMOVE_MOB_EFFECT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_REMOVE_MOB_EFFECT;
    pub const CLIENTBOUND_RESPAWN: &'static LazyLock<PacketType> = &CLIENTBOUND_RESPAWN;
    pub const CLIENTBOUND_ROTATE_HEAD: &'static LazyLock<PacketType> = &CLIENTBOUND_ROTATE_HEAD;
    pub const CLIENTBOUND_SECTION_BLOCKS_UPDATE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SECTION_BLOCKS_UPDATE;
    pub const CLIENTBOUND_SELECT_ADVANCEMENTS_TAB: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SELECT_ADVANCEMENTS_TAB;
    pub const CLIENTBOUND_SERVER_DATA: &'static LazyLock<PacketType> = &CLIENTBOUND_SERVER_DATA;
    pub const CLIENTBOUND_SET_ACTION_BAR_TEXT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_ACTION_BAR_TEXT;
    pub const CLIENTBOUND_SET_BORDER_CENTER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_BORDER_CENTER;
    pub const CLIENTBOUND_SET_BORDER_LERP_SIZE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_BORDER_LERP_SIZE;
    pub const CLIENTBOUND_SET_BORDER_SIZE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_BORDER_SIZE;
    pub const CLIENTBOUND_SET_BORDER_WARNING_DELAY: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_BORDER_WARNING_DELAY;
    pub const CLIENTBOUND_SET_BORDER_WARNING_DISTANCE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_BORDER_WARNING_DISTANCE;
    pub const CLIENTBOUND_SET_CAMERA: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_CAMERA;
    pub const CLIENTBOUND_SET_CHUNK_CACHE_CENTER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_CHUNK_CACHE_CENTER;
    pub const CLIENTBOUND_SET_CHUNK_CACHE_RADIUS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_CHUNK_CACHE_RADIUS;
    pub const CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION;
    pub const CLIENTBOUND_SET_DISPLAY_OBJECTIVE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_DISPLAY_OBJECTIVE;
    pub const CLIENTBOUND_SET_ENTITY_DATA: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_ENTITY_DATA;
    pub const CLIENTBOUND_SET_ENTITY_LINK: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_ENTITY_LINK;
    pub const CLIENTBOUND_SET_ENTITY_MOTION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_ENTITY_MOTION;
    pub const CLIENTBOUND_SET_EQUIPMENT: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_EQUIPMENT;
    pub const CLIENTBOUND_SET_EXPERIENCE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_EXPERIENCE;
    pub const CLIENTBOUND_SET_HEALTH: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_HEALTH;
    pub const CLIENTBOUND_SET_HELD_SLOT: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_HELD_SLOT;
    pub const CLIENTBOUND_SET_OBJECTIVE: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_OBJECTIVE;
    pub const CLIENTBOUND_SET_PASSENGERS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_PASSENGERS;
    pub const CLIENTBOUND_SET_PLAYER_TEAM: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_PLAYER_TEAM;
    pub const CLIENTBOUND_SET_SCORE: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_SCORE;
    pub const CLIENTBOUND_SET_SIMULATION_DISTANCE: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_SIMULATION_DISTANCE;
    pub const CLIENTBOUND_SET_SUBTITLE_TEXT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_SUBTITLE_TEXT;
    pub const CLIENTBOUND_SET_TIME: &'static LazyLock<PacketType> = &CLIENTBOUND_SET_TIME;
    pub const CLIENTBOUND_SET_TITLE_TEXT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_TITLE_TEXT;
    pub const CLIENTBOUND_SET_TITLES_ANIMATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_TITLES_ANIMATION;
    pub const CLIENTBOUND_SOUND_ENTITY: &'static LazyLock<PacketType> = &CLIENTBOUND_SOUND_ENTITY;
    pub const CLIENTBOUND_SOUND: &'static LazyLock<PacketType> = &CLIENTBOUND_SOUND;
    pub const CLIENTBOUND_START_CONFIGURATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_START_CONFIGURATION;
    pub const CLIENTBOUND_STOP_SOUND: &'static LazyLock<PacketType> = &CLIENTBOUND_STOP_SOUND;
    pub const CLIENTBOUND_SWING_ANIMATION: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SWING_ANIMATION;
    pub const CLIENTBOUND_SYSTEM_CHAT: &'static LazyLock<PacketType> = &CLIENTBOUND_SYSTEM_CHAT;
    pub const CLIENTBOUND_TAB_LIST: &'static LazyLock<PacketType> = &CLIENTBOUND_TAB_LIST;
    pub const CLIENTBOUND_TAG_QUERY: &'static LazyLock<PacketType> = &CLIENTBOUND_TAG_QUERY;
    pub const CLIENTBOUND_TAKE_ITEM_ENTITY: &'static LazyLock<PacketType> =
        &CLIENTBOUND_TAKE_ITEM_ENTITY;
    pub const CLIENTBOUND_TELEPORT_ENTITY: &'static LazyLock<PacketType> =
        &CLIENTBOUND_TELEPORT_ENTITY;
    pub const CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS;
    pub const CLIENTBOUND_UPDATE_ADVANCEMENTS: &'static LazyLock<PacketType> =
        &CLIENTBOUND_UPDATE_ADVANCEMENTS;
    pub const CLIENTBOUND_UPDATE_ATTRIBUTES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_UPDATE_ATTRIBUTES;
    pub const CLIENTBOUND_UPDATE_MOB_EFFECT: &'static LazyLock<PacketType> =
        &CLIENTBOUND_UPDATE_MOB_EFFECT;
    pub const CLIENTBOUND_UPDATE_RECIPES: &'static LazyLock<PacketType> =
        &CLIENTBOUND_UPDATE_RECIPES;
    pub const CLIENTBOUND_PROJECTILE_POWER: &'static LazyLock<PacketType> =
        &CLIENTBOUND_PROJECTILE_POWER;
    pub const CLIENTBOUND_WAYPOINT: &'static LazyLock<PacketType> = &CLIENTBOUND_WAYPOINT;
    pub const SERVERBOUND_ACCEPT_TELEPORTATION: &'static LazyLock<PacketType> =
        &SERVERBOUND_ACCEPT_TELEPORTATION;
    pub const SERVERBOUND_ATTACK: &'static LazyLock<PacketType> = &SERVERBOUND_ATTACK;
    pub const SERVERBOUND_BLOCK_ENTITY_TAG_QUERY: &'static LazyLock<PacketType> =
        &SERVERBOUND_BLOCK_ENTITY_TAG_QUERY;
    pub const SERVERBOUND_BUNDLE_ITEM_SELECTED: &'static LazyLock<PacketType> =
        &SERVERBOUND_BUNDLE_ITEM_SELECTED;
    pub const SERVERBOUND_CHANGE_DIFFICULTY: &'static LazyLock<PacketType> =
        &SERVERBOUND_CHANGE_DIFFICULTY;
    pub const SERVERBOUND_CHANGE_GAME_MODE: &'static LazyLock<PacketType> =
        &SERVERBOUND_CHANGE_GAME_MODE;
    pub const SERVERBOUND_CHAT_ACK: &'static LazyLock<PacketType> = &SERVERBOUND_CHAT_ACK;
    pub const SERVERBOUND_CHAT_COMMAND: &'static LazyLock<PacketType> = &SERVERBOUND_CHAT_COMMAND;
    pub const SERVERBOUND_CHAT_COMMAND_SIGNED: &'static LazyLock<PacketType> =
        &SERVERBOUND_CHAT_COMMAND_SIGNED;
    pub const SERVERBOUND_CHAT: &'static LazyLock<PacketType> = &SERVERBOUND_CHAT;
    pub const SERVERBOUND_CHAT_SESSION_UPDATE: &'static LazyLock<PacketType> =
        &SERVERBOUND_CHAT_SESSION_UPDATE;
    pub const SERVERBOUND_CHUNK_BATCH_RECEIVED: &'static LazyLock<PacketType> =
        &SERVERBOUND_CHUNK_BATCH_RECEIVED;
    pub const SERVERBOUND_CLIENT_COMMAND: &'static LazyLock<PacketType> =
        &SERVERBOUND_CLIENT_COMMAND;
    pub const SERVERBOUND_CLIENT_TICK_END: &'static LazyLock<PacketType> =
        &SERVERBOUND_CLIENT_TICK_END;
    pub const SERVERBOUND_COMMAND_SUGGESTION: &'static LazyLock<PacketType> =
        &SERVERBOUND_COMMAND_SUGGESTION;
    pub const SERVERBOUND_CONFIGURATION_ACKNOWLEDGED: &'static LazyLock<PacketType> =
        &SERVERBOUND_CONFIGURATION_ACKNOWLEDGED;
    pub const SERVERBOUND_CONTAINER_BUTTON_CLICK: &'static LazyLock<PacketType> =
        &SERVERBOUND_CONTAINER_BUTTON_CLICK;
    pub const SERVERBOUND_CONTAINER_CLICK: &'static LazyLock<PacketType> =
        &SERVERBOUND_CONTAINER_CLICK;
    pub const SERVERBOUND_CONTAINER_CLOSE: &'static LazyLock<PacketType> =
        &SERVERBOUND_CONTAINER_CLOSE;
    pub const SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED: &'static LazyLock<PacketType> =
        &SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED;
    pub const SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST: &'static LazyLock<PacketType> =
        &SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST;
    pub const SERVERBOUND_EDIT_BOOK: &'static LazyLock<PacketType> = &SERVERBOUND_EDIT_BOOK;
    pub const SERVERBOUND_ENTITY_TAG_QUERY: &'static LazyLock<PacketType> =
        &SERVERBOUND_ENTITY_TAG_QUERY;
    pub const SERVERBOUND_INTERACT: &'static LazyLock<PacketType> = &SERVERBOUND_INTERACT;
    pub const SERVERBOUND_JIGSAW_GENERATE: &'static LazyLock<PacketType> =
        &SERVERBOUND_JIGSAW_GENERATE;
    pub const SERVERBOUND_LOCK_DIFFICULTY: &'static LazyLock<PacketType> =
        &SERVERBOUND_LOCK_DIFFICULTY;
    pub const SERVERBOUND_MOVE_PLAYER_POS: &'static LazyLock<PacketType> =
        &SERVERBOUND_MOVE_PLAYER_POS;
    pub const SERVERBOUND_MOVE_PLAYER_POS_ROT: &'static LazyLock<PacketType> =
        &SERVERBOUND_MOVE_PLAYER_POS_ROT;
    pub const SERVERBOUND_MOVE_PLAYER_ROT: &'static LazyLock<PacketType> =
        &SERVERBOUND_MOVE_PLAYER_ROT;
    pub const SERVERBOUND_MOVE_PLAYER_STATUS_ONLY: &'static LazyLock<PacketType> =
        &SERVERBOUND_MOVE_PLAYER_STATUS_ONLY;
    pub const SERVERBOUND_MOVE_VEHICLE: &'static LazyLock<PacketType> = &SERVERBOUND_MOVE_VEHICLE;
    pub const SERVERBOUND_PADDLE_BOAT: &'static LazyLock<PacketType> = &SERVERBOUND_PADDLE_BOAT;
    pub const SERVERBOUND_PICK_ITEM_FROM_BLOCK: &'static LazyLock<PacketType> =
        &SERVERBOUND_PICK_ITEM_FROM_BLOCK;
    pub const SERVERBOUND_PICK_ITEM_FROM_ENTITY: &'static LazyLock<PacketType> =
        &SERVERBOUND_PICK_ITEM_FROM_ENTITY;
    pub const SERVERBOUND_PLACE_RECIPE: &'static LazyLock<PacketType> = &SERVERBOUND_PLACE_RECIPE;
    pub const SERVERBOUND_PLAYER_ABILITIES: &'static LazyLock<PacketType> =
        &SERVERBOUND_PLAYER_ABILITIES;
    pub const SERVERBOUND_PLAYER_ACTION: &'static LazyLock<PacketType> = &SERVERBOUND_PLAYER_ACTION;
    pub const SERVERBOUND_PLAYER_COMMAND: &'static LazyLock<PacketType> =
        &SERVERBOUND_PLAYER_COMMAND;
    pub const SERVERBOUND_PLAYER_INPUT: &'static LazyLock<PacketType> = &SERVERBOUND_PLAYER_INPUT;
    pub const SERVERBOUND_PLAYER_LOADED: &'static LazyLock<PacketType> = &SERVERBOUND_PLAYER_LOADED;
    pub const SERVERBOUND_PUNCH: &'static LazyLock<PacketType> = &SERVERBOUND_PUNCH;
    pub const SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS: &'static LazyLock<PacketType> =
        &SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS;
    pub const SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE: &'static LazyLock<PacketType> =
        &SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE;
    pub const SERVERBOUND_RENAME_ITEM: &'static LazyLock<PacketType> = &SERVERBOUND_RENAME_ITEM;
    pub const SERVERBOUND_SEEN_ADVANCEMENTS: &'static LazyLock<PacketType> =
        &SERVERBOUND_SEEN_ADVANCEMENTS;
    pub const SERVERBOUND_SELECT_TRADE: &'static LazyLock<PacketType> = &SERVERBOUND_SELECT_TRADE;
    pub const SERVERBOUND_SET_BEACON: &'static LazyLock<PacketType> = &SERVERBOUND_SET_BEACON;
    pub const SERVERBOUND_SET_CARRIED_ITEM: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_CARRIED_ITEM;
    pub const SERVERBOUND_SET_COMMAND_BLOCK: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_COMMAND_BLOCK;
    pub const SERVERBOUND_SET_COMMAND_MINECART: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_COMMAND_MINECART;
    pub const SERVERBOUND_SET_CREATIVE_MODE_SLOT: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_CREATIVE_MODE_SLOT;
    pub const SERVERBOUND_SET_GAME_RULE: &'static LazyLock<PacketType> = &SERVERBOUND_SET_GAME_RULE;
    pub const SERVERBOUND_SET_JIGSAW_BLOCK: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_JIGSAW_BLOCK;
    pub const SERVERBOUND_SET_STRUCTURE_BLOCK: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_STRUCTURE_BLOCK;
    pub const SERVERBOUND_SET_TEST_BLOCK: &'static LazyLock<PacketType> =
        &SERVERBOUND_SET_TEST_BLOCK;
    pub const SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION: &'static LazyLock<PacketType> =
        &SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION;
    pub const SERVERBOUND_SIGN_UPDATE: &'static LazyLock<PacketType> = &SERVERBOUND_SIGN_UPDATE;
    pub const SERVERBOUND_SPECTATOR_ACTION: &'static LazyLock<PacketType> =
        &SERVERBOUND_SPECTATOR_ACTION;
    pub const SERVERBOUND_TELEPORT_TO_ENTITY: &'static LazyLock<PacketType> =
        &SERVERBOUND_TELEPORT_TO_ENTITY;
    pub const SERVERBOUND_USE_ITEM_ON: &'static LazyLock<PacketType> = &SERVERBOUND_USE_ITEM_ON;
    pub const SERVERBOUND_USE_ITEM: &'static LazyLock<PacketType> = &SERVERBOUND_USE_ITEM;
    pub const CLIENTBOUND_RESET_SCORE: &'static LazyLock<PacketType> = &CLIENTBOUND_RESET_SCORE;
    pub const CLIENTBOUND_TICKING_STATE: &'static LazyLock<PacketType> = &CLIENTBOUND_TICKING_STATE;
    pub const CLIENTBOUND_TICKING_STEP: &'static LazyLock<PacketType> = &CLIENTBOUND_TICKING_STEP;
    pub const CLIENTBOUND_SET_CURSOR_ITEM: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_CURSOR_ITEM;
    pub const CLIENTBOUND_SET_PLAYER_INVENTORY: &'static LazyLock<PacketType> =
        &CLIENTBOUND_SET_PLAYER_INVENTORY;

    fn create_clientbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::CLIENTBOUND,
            Identifier::with_default_namespace(id),
        )
    }

    fn create_serverbound(id: &str) -> PacketType {
        PacketType::new(
            PacketFlow::SERVERBOUND,
            Identifier::with_default_namespace(id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::GamePacketTypes;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.GamePacketTypes");
        for (name, r#type) in [
            ("CLIENTBOUND_BUNDLE", GamePacketTypes::CLIENTBOUND_BUNDLE),
            (
                "CLIENTBOUND_BUNDLE_DELIMITER",
                GamePacketTypes::CLIENTBOUND_BUNDLE_DELIMITER,
            ),
            (
                "CLIENTBOUND_ADD_ENTITY",
                GamePacketTypes::CLIENTBOUND_ADD_ENTITY,
            ),
            ("CLIENTBOUND_ANIMATE", GamePacketTypes::CLIENTBOUND_ANIMATE),
            (
                "CLIENTBOUND_AWARD_STATS",
                GamePacketTypes::CLIENTBOUND_AWARD_STATS,
            ),
            (
                "CLIENTBOUND_BLOCK_CHANGED_ACK",
                GamePacketTypes::CLIENTBOUND_BLOCK_CHANGED_ACK,
            ),
            (
                "CLIENTBOUND_BLOCK_DESTRUCTION",
                GamePacketTypes::CLIENTBOUND_BLOCK_DESTRUCTION,
            ),
            (
                "CLIENTBOUND_BLOCK_ENTITY_DATA",
                GamePacketTypes::CLIENTBOUND_BLOCK_ENTITY_DATA,
            ),
            (
                "CLIENTBOUND_BLOCK_EVENT",
                GamePacketTypes::CLIENTBOUND_BLOCK_EVENT,
            ),
            (
                "CLIENTBOUND_BLOCK_UPDATE",
                GamePacketTypes::CLIENTBOUND_BLOCK_UPDATE,
            ),
            (
                "CLIENTBOUND_BOSS_EVENT",
                GamePacketTypes::CLIENTBOUND_BOSS_EVENT,
            ),
            (
                "CLIENTBOUND_CHANGE_DIFFICULTY",
                GamePacketTypes::CLIENTBOUND_CHANGE_DIFFICULTY,
            ),
            (
                "CLIENTBOUND_CHUNK_BATCH_FINISHED",
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_FINISHED,
            ),
            (
                "CLIENTBOUND_CHUNK_BATCH_START",
                GamePacketTypes::CLIENTBOUND_CHUNK_BATCH_START,
            ),
            (
                "CLIENTBOUND_CHUNKS_BIOMES",
                GamePacketTypes::CLIENTBOUND_CHUNKS_BIOMES,
            ),
            (
                "CLIENTBOUND_CLEAR_TITLES",
                GamePacketTypes::CLIENTBOUND_CLEAR_TITLES,
            ),
            (
                "CLIENTBOUND_COMMAND_SUGGESTIONS",
                GamePacketTypes::CLIENTBOUND_COMMAND_SUGGESTIONS,
            ),
            (
                "CLIENTBOUND_COMMANDS",
                GamePacketTypes::CLIENTBOUND_COMMANDS,
            ),
            (
                "CLIENTBOUND_CONTAINER_CLOSE",
                GamePacketTypes::CLIENTBOUND_CONTAINER_CLOSE,
            ),
            (
                "CLIENTBOUND_CONTAINER_SET_CONTENT",
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_CONTENT,
            ),
            (
                "CLIENTBOUND_CONTAINER_SET_DATA",
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_DATA,
            ),
            (
                "CLIENTBOUND_CONTAINER_SET_SLOT",
                GamePacketTypes::CLIENTBOUND_CONTAINER_SET_SLOT,
            ),
            (
                "CLIENTBOUND_COOLDOWN",
                GamePacketTypes::CLIENTBOUND_COOLDOWN,
            ),
            (
                "CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS",
                GamePacketTypes::CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS,
            ),
            (
                "CLIENTBOUND_DAMAGE_EVENT",
                GamePacketTypes::CLIENTBOUND_DAMAGE_EVENT,
            ),
            (
                "CLIENTBOUND_DEBUG_BLOCK_VALUE",
                GamePacketTypes::CLIENTBOUND_DEBUG_BLOCK_VALUE,
            ),
            (
                "CLIENTBOUND_DEBUG_CHUNK_VALUE",
                GamePacketTypes::CLIENTBOUND_DEBUG_CHUNK_VALUE,
            ),
            (
                "CLIENTBOUND_DEBUG_ENTITY_VALUE",
                GamePacketTypes::CLIENTBOUND_DEBUG_ENTITY_VALUE,
            ),
            (
                "CLIENTBOUND_DEBUG_EVENT",
                GamePacketTypes::CLIENTBOUND_DEBUG_EVENT,
            ),
            (
                "CLIENTBOUND_DEBUG_SAMPLE",
                GamePacketTypes::CLIENTBOUND_DEBUG_SAMPLE,
            ),
            (
                "CLIENTBOUND_DELETE_CHAT",
                GamePacketTypes::CLIENTBOUND_DELETE_CHAT,
            ),
            (
                "CLIENTBOUND_DISGUISED_CHAT",
                GamePacketTypes::CLIENTBOUND_DISGUISED_CHAT,
            ),
            (
                "CLIENTBOUND_ENTITY_EVENT",
                GamePacketTypes::CLIENTBOUND_ENTITY_EVENT,
            ),
            (
                "CLIENTBOUND_ENTITY_POSITION_SYNC",
                GamePacketTypes::CLIENTBOUND_ENTITY_POSITION_SYNC,
            ),
            ("CLIENTBOUND_EXPLODE", GamePacketTypes::CLIENTBOUND_EXPLODE),
            (
                "CLIENTBOUND_ADD_TRANSIENT_BLOCK",
                GamePacketTypes::CLIENTBOUND_ADD_TRANSIENT_BLOCK,
            ),
            (
                "CLIENTBOUND_FORGET_LEVEL_CHUNK",
                GamePacketTypes::CLIENTBOUND_FORGET_LEVEL_CHUNK,
            ),
            (
                "CLIENTBOUND_GAME_EVENT",
                GamePacketTypes::CLIENTBOUND_GAME_EVENT,
            ),
            (
                "CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS",
                GamePacketTypes::CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS,
            ),
            (
                "CLIENTBOUND_MOUNT_SCREEN_OPEN",
                GamePacketTypes::CLIENTBOUND_MOUNT_SCREEN_OPEN,
            ),
            (
                "CLIENTBOUND_HURT_ANIMATION",
                GamePacketTypes::CLIENTBOUND_HURT_ANIMATION,
            ),
            (
                "CLIENTBOUND_INITIALIZE_BORDER",
                GamePacketTypes::CLIENTBOUND_INITIALIZE_BORDER,
            ),
            (
                "CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT",
                GamePacketTypes::CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT,
            ),
            (
                "CLIENTBOUND_LEVEL_EVENT",
                GamePacketTypes::CLIENTBOUND_LEVEL_EVENT,
            ),
            (
                "CLIENTBOUND_LEVEL_PARTICLES",
                GamePacketTypes::CLIENTBOUND_LEVEL_PARTICLES,
            ),
            (
                "CLIENTBOUND_LIGHT_UPDATE",
                GamePacketTypes::CLIENTBOUND_LIGHT_UPDATE,
            ),
            ("CLIENTBOUND_LOGIN", GamePacketTypes::CLIENTBOUND_LOGIN),
            (
                "CLIENTBOUND_LOW_DISK_SPACE_WARNING",
                GamePacketTypes::CLIENTBOUND_LOW_DISK_SPACE_WARNING,
            ),
            (
                "CLIENTBOUND_MAP_ITEM_DATA",
                GamePacketTypes::CLIENTBOUND_MAP_ITEM_DATA,
            ),
            (
                "CLIENTBOUND_MERCHANT_OFFERS",
                GamePacketTypes::CLIENTBOUND_MERCHANT_OFFERS,
            ),
            (
                "CLIENTBOUND_MOVE_ENTITY_POS",
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS,
            ),
            (
                "CLIENTBOUND_MOVE_ENTITY_POS_ROT",
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_POS_ROT,
            ),
            (
                "CLIENTBOUND_MOVE_MINECART_ALONG_TRACK",
                GamePacketTypes::CLIENTBOUND_MOVE_MINECART_ALONG_TRACK,
            ),
            (
                "CLIENTBOUND_MOVE_ENTITY_ROT",
                GamePacketTypes::CLIENTBOUND_MOVE_ENTITY_ROT,
            ),
            (
                "CLIENTBOUND_MOVE_VEHICLE",
                GamePacketTypes::CLIENTBOUND_MOVE_VEHICLE,
            ),
            (
                "CLIENTBOUND_OPEN_BOOK",
                GamePacketTypes::CLIENTBOUND_OPEN_BOOK,
            ),
            (
                "CLIENTBOUND_OPEN_SCREEN",
                GamePacketTypes::CLIENTBOUND_OPEN_SCREEN,
            ),
            (
                "CLIENTBOUND_OPEN_SIGN_EDITOR",
                GamePacketTypes::CLIENTBOUND_OPEN_SIGN_EDITOR,
            ),
            (
                "CLIENTBOUND_PLACE_GHOST_RECIPE",
                GamePacketTypes::CLIENTBOUND_PLACE_GHOST_RECIPE,
            ),
            (
                "CLIENTBOUND_PLAYER_ABILITIES",
                GamePacketTypes::CLIENTBOUND_PLAYER_ABILITIES,
            ),
            (
                "CLIENTBOUND_GAME_RULE_VALUES",
                GamePacketTypes::CLIENTBOUND_GAME_RULE_VALUES,
            ),
            (
                "CLIENTBOUND_PLAYER_CHAT",
                GamePacketTypes::CLIENTBOUND_PLAYER_CHAT,
            ),
            (
                "CLIENTBOUND_PLAYER_COMBAT_END",
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_END,
            ),
            (
                "CLIENTBOUND_PLAYER_COMBAT_ENTER",
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_ENTER,
            ),
            (
                "CLIENTBOUND_PLAYER_COMBAT_KILL",
                GamePacketTypes::CLIENTBOUND_PLAYER_COMBAT_KILL,
            ),
            (
                "CLIENTBOUND_PLAYER_INFO_REMOVE",
                GamePacketTypes::CLIENTBOUND_PLAYER_INFO_REMOVE,
            ),
            (
                "CLIENTBOUND_PLAYER_INFO_UPDATE",
                GamePacketTypes::CLIENTBOUND_PLAYER_INFO_UPDATE,
            ),
            (
                "CLIENTBOUND_PLAYER_LOOK_AT",
                GamePacketTypes::CLIENTBOUND_PLAYER_LOOK_AT,
            ),
            (
                "CLIENTBOUND_PLAYER_POSITION",
                GamePacketTypes::CLIENTBOUND_PLAYER_POSITION,
            ),
            (
                "CLIENTBOUND_PLAYER_ROTATION",
                GamePacketTypes::CLIENTBOUND_PLAYER_ROTATION,
            ),
            (
                "CLIENTBOUND_RECIPE_BOOK_ADD",
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_ADD,
            ),
            (
                "CLIENTBOUND_RECIPE_BOOK_REMOVE",
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_REMOVE,
            ),
            (
                "CLIENTBOUND_RECIPE_BOOK_SETTINGS",
                GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS,
            ),
            (
                "CLIENTBOUND_REMOVE_ENTITIES",
                GamePacketTypes::CLIENTBOUND_REMOVE_ENTITIES,
            ),
            (
                "CLIENTBOUND_REMOVE_MOB_EFFECT",
                GamePacketTypes::CLIENTBOUND_REMOVE_MOB_EFFECT,
            ),
            ("CLIENTBOUND_RESPAWN", GamePacketTypes::CLIENTBOUND_RESPAWN),
            (
                "CLIENTBOUND_ROTATE_HEAD",
                GamePacketTypes::CLIENTBOUND_ROTATE_HEAD,
            ),
            (
                "CLIENTBOUND_SECTION_BLOCKS_UPDATE",
                GamePacketTypes::CLIENTBOUND_SECTION_BLOCKS_UPDATE,
            ),
            (
                "CLIENTBOUND_SELECT_ADVANCEMENTS_TAB",
                GamePacketTypes::CLIENTBOUND_SELECT_ADVANCEMENTS_TAB,
            ),
            (
                "CLIENTBOUND_SERVER_DATA",
                GamePacketTypes::CLIENTBOUND_SERVER_DATA,
            ),
            (
                "CLIENTBOUND_SET_ACTION_BAR_TEXT",
                GamePacketTypes::CLIENTBOUND_SET_ACTION_BAR_TEXT,
            ),
            (
                "CLIENTBOUND_SET_BORDER_CENTER",
                GamePacketTypes::CLIENTBOUND_SET_BORDER_CENTER,
            ),
            (
                "CLIENTBOUND_SET_BORDER_LERP_SIZE",
                GamePacketTypes::CLIENTBOUND_SET_BORDER_LERP_SIZE,
            ),
            (
                "CLIENTBOUND_SET_BORDER_SIZE",
                GamePacketTypes::CLIENTBOUND_SET_BORDER_SIZE,
            ),
            (
                "CLIENTBOUND_SET_BORDER_WARNING_DELAY",
                GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DELAY,
            ),
            (
                "CLIENTBOUND_SET_BORDER_WARNING_DISTANCE",
                GamePacketTypes::CLIENTBOUND_SET_BORDER_WARNING_DISTANCE,
            ),
            (
                "CLIENTBOUND_SET_CAMERA",
                GamePacketTypes::CLIENTBOUND_SET_CAMERA,
            ),
            (
                "CLIENTBOUND_SET_CHUNK_CACHE_CENTER",
                GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_CENTER,
            ),
            (
                "CLIENTBOUND_SET_CHUNK_CACHE_RADIUS",
                GamePacketTypes::CLIENTBOUND_SET_CHUNK_CACHE_RADIUS,
            ),
            (
                "CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION",
                GamePacketTypes::CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION,
            ),
            (
                "CLIENTBOUND_SET_DISPLAY_OBJECTIVE",
                GamePacketTypes::CLIENTBOUND_SET_DISPLAY_OBJECTIVE,
            ),
            (
                "CLIENTBOUND_SET_ENTITY_DATA",
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_DATA,
            ),
            (
                "CLIENTBOUND_SET_ENTITY_LINK",
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_LINK,
            ),
            (
                "CLIENTBOUND_SET_ENTITY_MOTION",
                GamePacketTypes::CLIENTBOUND_SET_ENTITY_MOTION,
            ),
            (
                "CLIENTBOUND_SET_EQUIPMENT",
                GamePacketTypes::CLIENTBOUND_SET_EQUIPMENT,
            ),
            (
                "CLIENTBOUND_SET_EXPERIENCE",
                GamePacketTypes::CLIENTBOUND_SET_EXPERIENCE,
            ),
            (
                "CLIENTBOUND_SET_HEALTH",
                GamePacketTypes::CLIENTBOUND_SET_HEALTH,
            ),
            (
                "CLIENTBOUND_SET_HELD_SLOT",
                GamePacketTypes::CLIENTBOUND_SET_HELD_SLOT,
            ),
            (
                "CLIENTBOUND_SET_OBJECTIVE",
                GamePacketTypes::CLIENTBOUND_SET_OBJECTIVE,
            ),
            (
                "CLIENTBOUND_SET_PASSENGERS",
                GamePacketTypes::CLIENTBOUND_SET_PASSENGERS,
            ),
            (
                "CLIENTBOUND_SET_PLAYER_TEAM",
                GamePacketTypes::CLIENTBOUND_SET_PLAYER_TEAM,
            ),
            (
                "CLIENTBOUND_SET_SCORE",
                GamePacketTypes::CLIENTBOUND_SET_SCORE,
            ),
            (
                "CLIENTBOUND_SET_SIMULATION_DISTANCE",
                GamePacketTypes::CLIENTBOUND_SET_SIMULATION_DISTANCE,
            ),
            (
                "CLIENTBOUND_SET_SUBTITLE_TEXT",
                GamePacketTypes::CLIENTBOUND_SET_SUBTITLE_TEXT,
            ),
            (
                "CLIENTBOUND_SET_TIME",
                GamePacketTypes::CLIENTBOUND_SET_TIME,
            ),
            (
                "CLIENTBOUND_SET_TITLE_TEXT",
                GamePacketTypes::CLIENTBOUND_SET_TITLE_TEXT,
            ),
            (
                "CLIENTBOUND_SET_TITLES_ANIMATION",
                GamePacketTypes::CLIENTBOUND_SET_TITLES_ANIMATION,
            ),
            (
                "CLIENTBOUND_SOUND_ENTITY",
                GamePacketTypes::CLIENTBOUND_SOUND_ENTITY,
            ),
            ("CLIENTBOUND_SOUND", GamePacketTypes::CLIENTBOUND_SOUND),
            (
                "CLIENTBOUND_START_CONFIGURATION",
                GamePacketTypes::CLIENTBOUND_START_CONFIGURATION,
            ),
            (
                "CLIENTBOUND_STOP_SOUND",
                GamePacketTypes::CLIENTBOUND_STOP_SOUND,
            ),
            (
                "CLIENTBOUND_SWING_ANIMATION",
                GamePacketTypes::CLIENTBOUND_SWING_ANIMATION,
            ),
            (
                "CLIENTBOUND_SYSTEM_CHAT",
                GamePacketTypes::CLIENTBOUND_SYSTEM_CHAT,
            ),
            (
                "CLIENTBOUND_TAB_LIST",
                GamePacketTypes::CLIENTBOUND_TAB_LIST,
            ),
            (
                "CLIENTBOUND_TAG_QUERY",
                GamePacketTypes::CLIENTBOUND_TAG_QUERY,
            ),
            (
                "CLIENTBOUND_TAKE_ITEM_ENTITY",
                GamePacketTypes::CLIENTBOUND_TAKE_ITEM_ENTITY,
            ),
            (
                "CLIENTBOUND_TELEPORT_ENTITY",
                GamePacketTypes::CLIENTBOUND_TELEPORT_ENTITY,
            ),
            (
                "CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS",
                GamePacketTypes::CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS,
            ),
            (
                "CLIENTBOUND_UPDATE_ADVANCEMENTS",
                GamePacketTypes::CLIENTBOUND_UPDATE_ADVANCEMENTS,
            ),
            (
                "CLIENTBOUND_UPDATE_ATTRIBUTES",
                GamePacketTypes::CLIENTBOUND_UPDATE_ATTRIBUTES,
            ),
            (
                "CLIENTBOUND_UPDATE_MOB_EFFECT",
                GamePacketTypes::CLIENTBOUND_UPDATE_MOB_EFFECT,
            ),
            (
                "CLIENTBOUND_UPDATE_RECIPES",
                GamePacketTypes::CLIENTBOUND_UPDATE_RECIPES,
            ),
            (
                "CLIENTBOUND_PROJECTILE_POWER",
                GamePacketTypes::CLIENTBOUND_PROJECTILE_POWER,
            ),
            (
                "CLIENTBOUND_WAYPOINT",
                GamePacketTypes::CLIENTBOUND_WAYPOINT,
            ),
            (
                "SERVERBOUND_ACCEPT_TELEPORTATION",
                GamePacketTypes::SERVERBOUND_ACCEPT_TELEPORTATION,
            ),
            ("SERVERBOUND_ATTACK", GamePacketTypes::SERVERBOUND_ATTACK),
            (
                "SERVERBOUND_BLOCK_ENTITY_TAG_QUERY",
                GamePacketTypes::SERVERBOUND_BLOCK_ENTITY_TAG_QUERY,
            ),
            (
                "SERVERBOUND_BUNDLE_ITEM_SELECTED",
                GamePacketTypes::SERVERBOUND_BUNDLE_ITEM_SELECTED,
            ),
            (
                "SERVERBOUND_CHANGE_DIFFICULTY",
                GamePacketTypes::SERVERBOUND_CHANGE_DIFFICULTY,
            ),
            (
                "SERVERBOUND_CHANGE_GAME_MODE",
                GamePacketTypes::SERVERBOUND_CHANGE_GAME_MODE,
            ),
            (
                "SERVERBOUND_CHAT_ACK",
                GamePacketTypes::SERVERBOUND_CHAT_ACK,
            ),
            (
                "SERVERBOUND_CHAT_COMMAND",
                GamePacketTypes::SERVERBOUND_CHAT_COMMAND,
            ),
            (
                "SERVERBOUND_CHAT_COMMAND_SIGNED",
                GamePacketTypes::SERVERBOUND_CHAT_COMMAND_SIGNED,
            ),
            ("SERVERBOUND_CHAT", GamePacketTypes::SERVERBOUND_CHAT),
            (
                "SERVERBOUND_CHAT_SESSION_UPDATE",
                GamePacketTypes::SERVERBOUND_CHAT_SESSION_UPDATE,
            ),
            (
                "SERVERBOUND_CHUNK_BATCH_RECEIVED",
                GamePacketTypes::SERVERBOUND_CHUNK_BATCH_RECEIVED,
            ),
            (
                "SERVERBOUND_CLIENT_COMMAND",
                GamePacketTypes::SERVERBOUND_CLIENT_COMMAND,
            ),
            (
                "SERVERBOUND_CLIENT_TICK_END",
                GamePacketTypes::SERVERBOUND_CLIENT_TICK_END,
            ),
            (
                "SERVERBOUND_COMMAND_SUGGESTION",
                GamePacketTypes::SERVERBOUND_COMMAND_SUGGESTION,
            ),
            (
                "SERVERBOUND_CONFIGURATION_ACKNOWLEDGED",
                GamePacketTypes::SERVERBOUND_CONFIGURATION_ACKNOWLEDGED,
            ),
            (
                "SERVERBOUND_CONTAINER_BUTTON_CLICK",
                GamePacketTypes::SERVERBOUND_CONTAINER_BUTTON_CLICK,
            ),
            (
                "SERVERBOUND_CONTAINER_CLICK",
                GamePacketTypes::SERVERBOUND_CONTAINER_CLICK,
            ),
            (
                "SERVERBOUND_CONTAINER_CLOSE",
                GamePacketTypes::SERVERBOUND_CONTAINER_CLOSE,
            ),
            (
                "SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED",
                GamePacketTypes::SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED,
            ),
            (
                "SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST",
                GamePacketTypes::SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST,
            ),
            (
                "SERVERBOUND_EDIT_BOOK",
                GamePacketTypes::SERVERBOUND_EDIT_BOOK,
            ),
            (
                "SERVERBOUND_ENTITY_TAG_QUERY",
                GamePacketTypes::SERVERBOUND_ENTITY_TAG_QUERY,
            ),
            (
                "SERVERBOUND_INTERACT",
                GamePacketTypes::SERVERBOUND_INTERACT,
            ),
            (
                "SERVERBOUND_JIGSAW_GENERATE",
                GamePacketTypes::SERVERBOUND_JIGSAW_GENERATE,
            ),
            (
                "SERVERBOUND_LOCK_DIFFICULTY",
                GamePacketTypes::SERVERBOUND_LOCK_DIFFICULTY,
            ),
            (
                "SERVERBOUND_MOVE_PLAYER_POS",
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_POS,
            ),
            (
                "SERVERBOUND_MOVE_PLAYER_POS_ROT",
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_POS_ROT,
            ),
            (
                "SERVERBOUND_MOVE_PLAYER_ROT",
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_ROT,
            ),
            (
                "SERVERBOUND_MOVE_PLAYER_STATUS_ONLY",
                GamePacketTypes::SERVERBOUND_MOVE_PLAYER_STATUS_ONLY,
            ),
            (
                "SERVERBOUND_MOVE_VEHICLE",
                GamePacketTypes::SERVERBOUND_MOVE_VEHICLE,
            ),
            (
                "SERVERBOUND_PADDLE_BOAT",
                GamePacketTypes::SERVERBOUND_PADDLE_BOAT,
            ),
            (
                "SERVERBOUND_PICK_ITEM_FROM_BLOCK",
                GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_BLOCK,
            ),
            (
                "SERVERBOUND_PICK_ITEM_FROM_ENTITY",
                GamePacketTypes::SERVERBOUND_PICK_ITEM_FROM_ENTITY,
            ),
            (
                "SERVERBOUND_PLACE_RECIPE",
                GamePacketTypes::SERVERBOUND_PLACE_RECIPE,
            ),
            (
                "SERVERBOUND_PLAYER_ABILITIES",
                GamePacketTypes::SERVERBOUND_PLAYER_ABILITIES,
            ),
            (
                "SERVERBOUND_PLAYER_ACTION",
                GamePacketTypes::SERVERBOUND_PLAYER_ACTION,
            ),
            (
                "SERVERBOUND_PLAYER_COMMAND",
                GamePacketTypes::SERVERBOUND_PLAYER_COMMAND,
            ),
            (
                "SERVERBOUND_PLAYER_INPUT",
                GamePacketTypes::SERVERBOUND_PLAYER_INPUT,
            ),
            (
                "SERVERBOUND_PLAYER_LOADED",
                GamePacketTypes::SERVERBOUND_PLAYER_LOADED,
            ),
            ("SERVERBOUND_PUNCH", GamePacketTypes::SERVERBOUND_PUNCH),
            (
                "SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS",
                GamePacketTypes::SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS,
            ),
            (
                "SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE",
                GamePacketTypes::SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE,
            ),
            (
                "SERVERBOUND_RENAME_ITEM",
                GamePacketTypes::SERVERBOUND_RENAME_ITEM,
            ),
            (
                "SERVERBOUND_SEEN_ADVANCEMENTS",
                GamePacketTypes::SERVERBOUND_SEEN_ADVANCEMENTS,
            ),
            (
                "SERVERBOUND_SELECT_TRADE",
                GamePacketTypes::SERVERBOUND_SELECT_TRADE,
            ),
            (
                "SERVERBOUND_SET_BEACON",
                GamePacketTypes::SERVERBOUND_SET_BEACON,
            ),
            (
                "SERVERBOUND_SET_CARRIED_ITEM",
                GamePacketTypes::SERVERBOUND_SET_CARRIED_ITEM,
            ),
            (
                "SERVERBOUND_SET_COMMAND_BLOCK",
                GamePacketTypes::SERVERBOUND_SET_COMMAND_BLOCK,
            ),
            (
                "SERVERBOUND_SET_COMMAND_MINECART",
                GamePacketTypes::SERVERBOUND_SET_COMMAND_MINECART,
            ),
            (
                "SERVERBOUND_SET_CREATIVE_MODE_SLOT",
                GamePacketTypes::SERVERBOUND_SET_CREATIVE_MODE_SLOT,
            ),
            (
                "SERVERBOUND_SET_GAME_RULE",
                GamePacketTypes::SERVERBOUND_SET_GAME_RULE,
            ),
            (
                "SERVERBOUND_SET_JIGSAW_BLOCK",
                GamePacketTypes::SERVERBOUND_SET_JIGSAW_BLOCK,
            ),
            (
                "SERVERBOUND_SET_STRUCTURE_BLOCK",
                GamePacketTypes::SERVERBOUND_SET_STRUCTURE_BLOCK,
            ),
            (
                "SERVERBOUND_SET_TEST_BLOCK",
                GamePacketTypes::SERVERBOUND_SET_TEST_BLOCK,
            ),
            (
                "SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION",
                GamePacketTypes::SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION,
            ),
            (
                "SERVERBOUND_SIGN_UPDATE",
                GamePacketTypes::SERVERBOUND_SIGN_UPDATE,
            ),
            (
                "SERVERBOUND_SPECTATOR_ACTION",
                GamePacketTypes::SERVERBOUND_SPECTATOR_ACTION,
            ),
            (
                "SERVERBOUND_TELEPORT_TO_ENTITY",
                GamePacketTypes::SERVERBOUND_TELEPORT_TO_ENTITY,
            ),
            (
                "SERVERBOUND_USE_ITEM_ON",
                GamePacketTypes::SERVERBOUND_USE_ITEM_ON,
            ),
            (
                "SERVERBOUND_USE_ITEM",
                GamePacketTypes::SERVERBOUND_USE_ITEM,
            ),
            (
                "CLIENTBOUND_RESET_SCORE",
                GamePacketTypes::CLIENTBOUND_RESET_SCORE,
            ),
            (
                "CLIENTBOUND_TICKING_STATE",
                GamePacketTypes::CLIENTBOUND_TICKING_STATE,
            ),
            (
                "CLIENTBOUND_TICKING_STEP",
                GamePacketTypes::CLIENTBOUND_TICKING_STEP,
            ),
            (
                "CLIENTBOUND_SET_CURSOR_ITEM",
                GamePacketTypes::CLIENTBOUND_SET_CURSOR_ITEM,
            ),
            (
                "CLIENTBOUND_SET_PLAYER_INVENTORY",
                GamePacketTypes::CLIENTBOUND_SET_PLAYER_INVENTORY,
            ),
        ] {
            golden.field(name, (**r#type).clone());
        }
        golden.finish();
    }
}
