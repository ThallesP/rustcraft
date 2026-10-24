//! net/minecraft/client/multiplayer/ClientPacketListener.java
//!
//! Not ported yet: nearly all of it. The game protocol's packets it handles so far are the ones a
//! join needs: the login, the teleports, bundles, the chunk batches (the chunks themselves are the
//! world track's), the chunk cache's radius, the simulation distance, the game events (the
//! level's loading, the game mode, and the rain and thunder) and the return to configuration, with
//! the common ones. The chunks, their light, block updates and biomes fill the level
//! (`ClientLevel`), whose light updates the handlers queue like Java's, and the border packets set
//! its world border. The chunk cache's center, the player's abilities, health, experience and held
//! slot, the difficulty and the tick rate are read too, and so are an entity's motion, hurt
//! animation, events and damage, and their handlers leave what they set (the player's, the
//! level's, the entities') for the tracks that port it. The time sets the level's game time and the clocks (`clockManager`, which the sky's
//! timelines read), the spawn the level's, and the player's attributes and synced data are kept on
//! the player for the player track (`LocalPlayer`); other entities' aren't ported, like the
//! entities. Every other packet is `Unported`, and answers nothing. Also not ported: chat signing,
//! the command tree (`sendCommand` sends every command unsigned, which is what a command without
//! signed arguments is), recipes, advancements, the scoreboard, telemetry and the debug
//! subscriptions.
//!
//! Java's level is the game's (`minecraft.level`), which the listener makes on login, and the
//! listener reads it from there. The player's connection, and the game mode's, is this listener.
//!
//! Its half on Netty's thread (`NettyHalf`, PORTING.md "The connection's threads") runs the chunk
//! batches' handlers and `handlePongResponse`, like Java's, with the common ones, and queues
//! every other packet for the game thread's. Java also runs `handlePlayerCombatEnd`,
//! `handlePlayerCombatEnter` (both empty), `handleDebugSample` and
//! `handleLowDiskSpaceWarning` there, whose packets aren't ported yet.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::java::util::bit_set::BitSet;
use crate::net::minecraft::client::client_clock_manager::ClientClockManager;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::chunk_batch_size_calculator::ChunkBatchSizeCalculator;
use crate::net::minecraft::client::multiplayer::client_common_packet_listener_impl::{
    self, ClientCommonPacketListenerImpl, client_common_handlers, client_common_netty_handlers,
};
use crate::net::minecraft::client::multiplayer::client_configuration_packet_listener_impl::ClientConfigurationPacketListenerImpl;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::common_listener_cookie::CommonListenerCookie;
use crate::net::minecraft::client::multiplayer::level_load_tracker::LevelLoadTracker;
use crate::net::minecraft::client::multiplayer::multi_player_game_mode::MultiPlayerGameMode;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::extract::level_extractor::LevelExtractor;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::connection::{Connection, NettyListener, SplitListener};
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;
use crate::net::minecraft::network::packet_listener::PacketListener;
use crate::net::minecraft::network::packet_processor::{GameThreadListener, lock};
use crate::net::minecraft::network::protocol::configuration::configuration_protocols::ConfigurationProtocols;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
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
use crate::net::minecraft::network::protocol::game::clientbound_set_border_center_packet::ClientboundSetBorderCenterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_border_lerp_size_packet::ClientboundSetBorderLerpSizePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_border_size_packet::ClientboundSetBorderSizePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_border_warning_delay_packet::ClientboundSetBorderWarningDelayPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_border_warning_distance_packet::ClientboundSetBorderWarningDistancePacket;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet::ClientboundLightUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet_data::ClientboundLightUpdatePacketData;
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
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_accept_teleportation_packet::ServerboundAcceptTeleportationPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chat_command_packet::ServerboundChatCommandPacket;
use crate::net::minecraft::network::protocol::game::serverbound_chunk_batch_received_packet::ServerboundChunkBatchReceivedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_configuration_acknowledged_packet::ServerboundConfigurationAcknowledgedPacket;
use crate::net::minecraft::network::protocol::game::serverbound_player_loaded_packet::ServerboundPlayerLoadedPacket;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_utils::PacketUtils;
use crate::net::minecraft::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use crate::net::minecraft::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use crate::net::minecraft::network::syncher::entity_data_serializer::EntityDataValue;
use crate::net::minecraft::network::tickable_packet_listener::TickablePacketListener;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::stats::recipe_book_settings::RecipeBookSettings;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::problem_reporter::ScopedCollector;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::living_entity::LivingEntityClass;
use crate::net::minecraft::world::entity::position_move_rotation::PositionMoveRotation;
use crate::net::minecraft::world::entity::relative::Relative;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::rustcraft::bench;

/// The client's listener in the game protocol.
pub struct ClientPacketListener {
    this: Weak<RefCell<ClientPacketListener>>,
    common: ClientCommonPacketListenerImpl<dyn ServerGamePacketListener>,
    local_game_profile: GameProfile,
    registry_access: Arc<RegistryAccess>,
    enabled_features: Vec<Identifier>,
    server_chunk_radius: i32,
    server_simulation_distance: i32,
    levels: Vec<ResourceKey>,
    shared: Arc<Shared>,
    level_load_tracker: Option<LevelLoadTracker>,
    server_enforces_secure_chat: bool,
    online_mode: bool,
    client_loaded: bool,
    clock_manager: ClientClockManager,
    /// What Java's player's recipe book (`ClientRecipeBook`, not ported) holds of its settings.
    recipe_book_settings: RecipeBookSettings,
    // What `Gui` shows of the server's texts (titles, the action bar, the tab list), which isn't
    // ported: kept here for it.
    texts: ServerTexts,
}

impl ClientPacketListener {
    pub fn new(
        minecraft: Rc<Minecraft>,
        connection: Arc<Connection>,
        cookie: CommonListenerCookie,
    ) -> Rc<RefCell<ClientPacketListener>> {
        let common = ClientCommonPacketListenerImpl::new(minecraft, connection, &cookie);
        Rc::new_cyclic(|this| {
            RefCell::new(ClientPacketListener {
                this: this.clone(),
                common,
                local_game_profile: cookie.local_game_profile,
                registry_access: cookie.received_registries,
                enabled_features: cookie.enabled_features,
                server_chunk_radius: 3,
                server_simulation_distance: 3,
                levels: Vec::new(),
                shared: Arc::new(Shared {
                    closed: AtomicBool::new(false),
                    chunk_batch_size_calculator: Mutex::new(ChunkBatchSizeCalculator::new()),
                }),
                level_load_tracker: Some(cookie.level_load_tracker),
                server_enforces_secure_chat: false,
                online_mode: false,
                client_loaded: false,
                clock_manager: ClientClockManager::new(),
                recipe_book_settings: RecipeBookSettings::new(),
                texts: ServerTexts::default(),
            })
        })
    }

    pub fn get_connection(&self) -> Arc<Connection> {
        Arc::clone(&self.common.connection)
    }

    pub fn get_local_game_profile(&self) -> &GameProfile {
        &self.local_game_profile
    }

    pub fn registry_access(&self) -> &RegistryAccess {
        &self.registry_access
    }

    pub fn levels(&self) -> &[ResourceKey] {
        &self.levels
    }

    /// The clocks the server sets (`handleSetTime`): what the sky's timelines read the time of day
    /// from (`ClientLevel.clockManager()`, Java's `Timeline` and `AttributeTrackSampler`).
    pub fn clock_manager(&self) -> &ClientClockManager {
        &self.clock_manager
    }

    pub fn clock_manager_mut(&mut self) -> &mut ClientClockManager {
        &mut self.clock_manager
    }

    /// The recipe books' settings the server sent, which Java keeps in the player's recipe book.
    /// The titles, action bar text, tab list and message of the day the server sent last.
    pub fn texts(&self) -> &ServerTexts {
        &self.texts
    }

    pub fn recipe_book_settings(&self) -> &RecipeBookSettings {
        &self.recipe_book_settings
    }

    pub fn enabled_features(&self) -> &[Identifier] {
        &self.enabled_features
    }

    pub fn send(&self, packet: impl Packet<dyn ServerGamePacketListener>) {
        self.common.send(packet);
    }

    pub fn close(&mut self) {
        self.shared.closed.store(true, Ordering::SeqCst);
        self.clear_level();
    }

    pub fn clear_level(&mut self) {
        self.level_load_tracker = None;
    }

    /// A command, without its slash. Java parses it with the server's command tree first, to
    /// sign the arguments that are messages; that tree isn't ported yet, so it's sent unsigned,
    /// which is right for every command without such arguments.
    pub fn send_command(&self, command: &str) {
        self.send(ServerboundChatCommandPacket::new(command.to_string()));
    }

    pub fn has_client_loaded(&self) -> bool {
        self.client_loaded
    }

    fn set_client_loaded(&mut self, client_loaded: bool) {
        self.client_loaded = client_loaded;
    }

    fn notify_player_loaded(&mut self) {
        if !self.has_client_loaded() {
            self.send(ServerboundPlayerLoadedPacket);
            self.set_client_loaded(true);
        }
    }

    fn start_waiting_for_new_level(&mut self) {
        self.level_load_tracker
            .get_or_insert_with(LevelLoadTracker::new)
            .start_client_load();
        self.common.minecraft.show_level_loading();
    }

    /// The renderer's part of the level's load: once it has compiled the section the camera is
    /// in, it calls the tracker back (`LevelExtractor`'s `playerCompiledSectionCallback`). Until
    /// the render track moves the call there, the chunk the player is in being in the level stands
    /// for it (`LevelLoadTracker`): what the game thread has handled, so the moment doesn't depend
    /// on when Netty's thread reads the chunk batches.
    fn check_player_section(&self) {
        let Some(callback) = self
            .level_load_tracker
            .as_ref()
            .and_then(LevelLoadTracker::get_player_compiled_section_callback)
        else {
            return;
        };
        let minecraft = &self.common.minecraft;
        let player = minecraft.player.borrow();
        let level = minecraft.level.borrow();
        let (Some(player), Some(level)) = (player.as_ref(), level.as_ref()) else {
            return;
        };
        let chunk_x = SectionPos::block_to_section_coord(Mth::floor_f64(player.get_x()));
        let chunk_z = SectionPos::block_to_section_coord(Mth::floor_f64(player.get_z()));
        if level
            .get_chunk_source()
            .get_chunk(chunk_x, chunk_z)
            .is_some()
        {
            callback();
        }
    }

    /// `handleLevelChunkWithLight`'s body on the game thread, on the level: the chunk goes into
    /// the level's storage, and its light is queued, to be kept and turned on.
    pub fn handle_level_chunk_with_light_in(
        level: &mut ClientLevel,
        packet: &ClientboundLevelChunkWithLightPacket,
    ) {
        let x = packet.x();
        let z = packet.z();
        // `ClientChunkCache.replaceWithPacketData` tells the level of a chunk it took.
        if level
            .get_chunk_source_mut()
            .replace_with_packet_data(x, z, packet.chunk_data())
            .is_some()
        {
            level.on_chunk_loaded(ChunkPos::new(x, z));
        }
        let light_data = packet.light_data().clone();
        level.queue_light_update(Box::new(move |level| {
            ClientPacketListener::apply_light_data(level, x, z, &light_data, false);
            if level.get_chunk_source().get_chunk(x, z).is_some() {
                ClientPacketListener::enable_chunk_light(level, x, z);
            }
        }));
    }

    /// `handleChunksBiomes`' body on the game thread, on the level: each chunk's biomes, then its
    /// tints worked out again, then the sections of it and around it are dirty for the renderer.
    pub fn handle_chunks_biomes_in(
        level: &mut ClientLevel,
        packet: &ClientboundChunksBiomesPacket,
    ) {
        for data in packet.chunk_biome_data() {
            level.get_chunk_source_mut().replace_biomes(
                data.pos().x(),
                data.pos().z(),
                &mut data.get_read_buffer(),
            );
        }

        for data in packet.chunk_biome_data() {
            level.on_chunk_loaded(ChunkPos::new(data.pos().x(), data.pos().z()));
        }

        for data in packet.chunk_biome_data() {
            for x_offset in -1..=1 {
                for z_offset in -1..=1 {
                    for y in level.get_min_section_y()..=level.get_max_section_y() {
                        level.level_extractor().set_section_dirty(
                            data.pos().x() + x_offset,
                            y,
                            data.pos().z() + z_offset,
                        );
                    }
                }
            }
        }
    }

    /// `handleLightUpdatePacket`'s body on the game thread, on the level.
    pub fn handle_light_update_packet_in(
        level: &mut ClientLevel,
        packet: &ClientboundLightUpdatePacket,
    ) {
        let x = packet.x();
        let z = packet.z();
        let light_data = packet.light_data().clone();
        level.queue_light_update(Box::new(move |level| {
            ClientPacketListener::apply_light_data(level, x, z, &light_data, true);
        }));
    }

    /// `handleForgetLevelChunk`'s body on the game thread, on the level.
    pub fn handle_forget_level_chunk_in(
        level: &mut ClientLevel,
        packet: &ClientboundForgetLevelChunkPacket,
    ) {
        level.get_chunk_source_mut().drop(packet.pos());
        ClientPacketListener::queue_light_removal(level, packet);
    }

    /// Each section's emptiness, which turns the light engine's storage of it on, for a chunk the
    /// level has, then the sections around it are dirty for the renderer.
    fn enable_chunk_light(level: &mut ClientLevel, x: i32, z: i32) {
        let chunk = level
            .get_chunk_source()
            .get_chunk(x, z)
            .expect("the caller checked the chunk is there");
        let chunk_pos = chunk.get_pos();
        let emptiness: Vec<bool> = chunk
            .get_sections()
            .iter()
            .map(|section| section.has_only_air())
            .collect();

        for (section_index, has_only_air) in emptiness.into_iter().enumerate() {
            let section_y = level.get_section_y_from_section_index(section_index as i32);
            let (light_engine, _) = level.get_light_engine_mut();
            light_engine.update_section_status(
                SectionPos::of_chunk_pos(&chunk_pos, section_y),
                has_only_air,
            );
        }

        level.set_section_range_dirty(
            x - 1,
            level.get_min_section_y(),
            z - 1,
            x + 1,
            level.get_max_section_y(),
            z + 1,
        );
    }

    /// The chunk's light goes off, its queued layers are dropped, and its sections stop being
    /// stored, once the level runs it.
    fn queue_light_removal(level: &mut ClientLevel, packet: &ClientboundForgetLevelChunkPacket) {
        let chunk_pos = packet.pos();
        level.queue_light_update(Box::new(move |level| {
            let min_section_y = level.get_min_section_y();
            let max_section_y = level.get_max_section_y();
            let (light_engine, chunks) = level.get_light_engine_mut();
            light_engine.set_light_enabled(chunks, chunk_pos, false);

            for section_y in
                light_engine.get_min_light_section()..light_engine.get_max_light_section()
            {
                let section_pos = SectionPos::of_chunk_pos(&chunk_pos, section_y);
                light_engine.queue_section_data(LightLayer::BLOCK, &section_pos, None);
                light_engine.queue_section_data(LightLayer::SKY, &section_pos, None);
            }

            for section_y in min_section_y..=max_section_y {
                light_engine
                    .update_section_status(SectionPos::of_chunk_pos(&chunk_pos, section_y), true);
            }
        }));
    }

    /// The server's light for a chunk: each layer's sections queued, then the chunk's light on.
    /// With `schedule_rebuild`, each section it read and its neighbours are dirty for the
    /// renderer.
    fn apply_light_data(
        level: &mut ClientLevel,
        x: i32,
        z: i32,
        light_data: &ClientboundLightUpdatePacketData,
        schedule_rebuild: bool,
    ) {
        let level_extractor = Rc::clone(level.level_extractor());
        let (light_engine, chunks) = level.get_light_engine_mut();
        ClientPacketListener::read_section_list(
            x,
            z,
            light_engine,
            &level_extractor,
            LightLayer::SKY,
            light_data.sky_y_mask(),
            light_data.empty_sky_y_mask(),
            &mut light_data.sky_updates().iter(),
            schedule_rebuild,
        );
        ClientPacketListener::read_section_list(
            x,
            z,
            light_engine,
            &level_extractor,
            LightLayer::BLOCK,
            light_data.block_y_mask(),
            light_data.empty_block_y_mask(),
            &mut light_data.block_updates().iter(),
            schedule_rebuild,
        );
        light_engine.set_light_enabled(chunks, ChunkPos::new(x, z), true);
    }

    /// Each light section the masks name: its layer from the updates, in order, or an empty one.
    /// Panics when the updates run out, or a layer isn't 2048 bytes, where Java throws.
    #[allow(clippy::too_many_arguments)]
    fn read_section_list<'a>(
        chunk_x: i32,
        chunk_z: i32,
        light_engine: &mut LevelLightEngine,
        level_extractor: &LevelExtractor,
        layer: LightLayer,
        y_mask: &BitSet,
        empty_y_mask: &BitSet,
        updates: &mut impl Iterator<Item = &'a Vec<i8>>,
        schedule_rebuild: bool,
    ) {
        for section_index in 0..light_engine.get_light_section_count() {
            let section_y = light_engine.get_min_light_section() + section_index;
            let have_data = y_mask.get(section_index);
            let have_empty = empty_y_mask.get(section_index);
            if have_data || have_empty {
                let data = if have_data {
                    DataLayer::from_data(updates.next().expect("an update for each section"))
                } else {
                    DataLayer::new()
                };
                light_engine.queue_section_data(
                    layer,
                    &SectionPos::of(chunk_x, section_y, chunk_z),
                    Some(data),
                );
                if schedule_rebuild {
                    // The level's setSectionDirtyWithNeighbors.
                    level_extractor.set_section_dirty_with_neighbors(chunk_x, section_y, chunk_z);
                }
            }
        }
    }

    /// `setValuesFromPositionPacket` without interpolation, which is what the player's teleport
    /// is: the values the change makes of the entity's, and of where it was last tick.
    fn set_values_from_position_packet(
        change: &PositionMoveRotation,
        relatives: &[Relative],
        entity: &mut LocalPlayer,
    ) {
        let current_values = PositionMoveRotation::new(
            entity.position(),
            entity.get_known_movement(),
            entity.get_y_rot(),
            entity.get_x_rot(),
        );
        let new_values =
            PositionMoveRotation::calculate_absolute(&current_values, change, relatives);
        entity.set_pos(new_values.position());
        entity.set_delta_movement(new_values.delta_movement());
        entity.set_y_rot(new_values.y_rot());
        entity.set_x_rot(new_values.x_rot());
        let current_interpolation_values = PositionMoveRotation::new(
            entity.old_position(),
            Vec3::ZERO,
            entity.y_rot_o,
            entity.x_rot_o,
        );
        let interpolation_values = PositionMoveRotation::calculate_absolute(
            &current_interpolation_values,
            change,
            relatives,
        );
        entity.set_old_pos_and_rot(
            interpolation_values.position(),
            interpolation_values.y_rot(),
            interpolation_values.x_rot(),
        );
    }
}

impl PacketListener for ClientPacketListener {
    /// Java's goes through `Minecraft.disconnect`, which closes this listener; it's the one
    /// calling, so it closes itself first.
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        self.close();
        self.common.on_disconnect(details);
    }

    fn on_packet_error(&mut self, packet: &dyn std::any::Any, cause: &str) {
        let _ = packet;
        self.common.on_packet_error("in play", cause);
    }

    fn is_accepting_messages(&self) -> bool {
        self.shared.is_accepting_messages(&self.common.connection)
    }

    fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.common.should_handle_message(packet)
    }

    fn as_tickable(&mut self) -> Option<&mut dyn TickablePacketListener> {
        Some(self)
    }
}

impl TickablePacketListener for ClientPacketListener {
    fn tick(&mut self) {
        self.common.send_deferred_packets();
        self.check_player_section();
        if let Some(level_load_tracker) = self.level_load_tracker.as_mut() {
            level_load_tracker.tick_client_load();
            if level_load_tracker.is_level_ready() {
                self.notify_player_loaded();
                self.level_load_tracker = None;
            }
        }
    }
}

client_common_handlers!(ClientPacketListener);

/// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
impl ClientPongPacketListener for ClientPacketListener {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
        self.shared.handle_pong_response(packet);
    }
}

/// The fields both of the listener's halves touch, and the handlers Java runs on Netty's thread,
/// which either half can run: Netty's, or the game thread's for a packet of a bundle.
struct Shared {
    closed: AtomicBool,
    chunk_batch_size_calculator: Mutex<ChunkBatchSizeCalculator>,
}

impl Shared {
    fn is_accepting_messages(&self, connection: &Connection) -> bool {
        connection.is_connected() && !self.closed.load(Ordering::SeqCst)
    }

    fn handle_chunk_batch_start(&self, packet: &ClientboundChunkBatchStartPacket) {
        let _ = packet;
        lock(&self.chunk_batch_size_calculator).on_batch_start();
    }

    /// The answer goes out at once, on the thread handling it.
    fn handle_chunk_batch_finished(
        &self,
        connection: &Connection,
        packet: &ClientboundChunkBatchFinishedPacket,
    ) {
        let mut chunk_batch_size_calculator = lock(&self.chunk_batch_size_calculator);
        chunk_batch_size_calculator.on_batch_finished(packet.batch_size());
        connection.send::<dyn ServerGamePacketListener>(ServerboundChunkBatchReceivedPacket::new(
            chunk_batch_size_calculator.get_desired_chunks_per_tick(),
        ));
    }

    /// Java logs the round trip in the debug screen's ping chart, which isn't ported, and pings
    /// only while that chart shows (`PingDebugMonitor`), so a server only sends a pong to a
    /// client that asked.
    fn handle_pong_response(&self, packet: &ClientboundPongResponsePacket) {
        let _ = packet;
    }
}

impl SplitListener<dyn ClientGamePacketListener> for ClientPacketListener {
    fn netty_listener(
        &self,
        this: GameThreadListener<ClientPacketListener>,
    ) -> Box<dyn NettyListener<dyn ClientGamePacketListener>> {
        Box::new(NettyHalf {
            this,
            common: self.common.netty_half(),
            shared: Arc::clone(&self.shared),
        })
    }
}

/// What Netty's thread runs of the listener: the handlers Java runs there, and for the others,
/// up to `ensureRunningOnSameThread`, which queues the packet for the game thread's half, `this`.
/// A handler ported on the game thread's half comes here too, Java's handler up to that call.
pub struct NettyHalf {
    this: GameThreadListener<ClientPacketListener>,
    common: client_common_packet_listener_impl::NettyHalf<dyn ServerGamePacketListener>,
    shared: Arc<Shared>,
}

impl NettyListener<dyn ClientGamePacketListener> for NettyHalf {
    fn listener(&mut self) -> &mut (dyn ClientGamePacketListener + 'static) {
        self
    }
}

impl PacketListener for NettyHalf {
    /// The game thread's half hears it, which the connection ticks.
    fn on_disconnect(&mut self, details: &DisconnectionDetails) {
        let _ = details;
    }

    fn is_accepting_messages(&self) -> bool {
        self.shared.is_accepting_messages(&self.common.connection)
    }

    fn should_handle_message(&self, packet: &dyn std::any::Any) -> bool {
        self.common.should_handle_message(packet)
    }
}

client_common_netty_handlers!(NettyHalf);

impl ClientPongPacketListener for NettyHalf {
    fn handle_pong_response(&mut self, packet: &ClientboundPongResponsePacket) {
        self.shared.handle_pong_response(packet);
    }
}

/// The server's texts that `Gui` and the server list would show, with the titles' times, which
/// Java's `Hud` keeps (`setTimes`, `setTitle`, `clearTitles`, `resetTitleTimes`); the HUD's tick
/// counting `title_time` down comes with the titles' drawing.
#[derive(Clone, Debug)]
pub struct ServerTexts {
    pub title: Option<Component>,
    pub subtitle: Option<Component>,
    pub action_bar: Option<Component>,
    pub tab_list_header: Option<Component>,
    pub tab_list_footer: Option<Component>,
    pub motd: Option<Component>,
    pub title_time: i32,
    pub title_fade_in_time: i32,
    pub title_stay_time: i32,
    pub title_fade_out_time: i32,
}

impl Default for ServerTexts {
    fn default() -> ServerTexts {
        let mut texts = ServerTexts {
            title: None,
            subtitle: None,
            action_bar: None,
            tab_list_header: None,
            tab_list_footer: None,
            motd: None,
            title_time: 0,
            title_fade_in_time: 0,
            title_stay_time: 0,
            title_fade_out_time: 0,
        };
        texts.reset_title_times();
        texts
    }
}

impl ServerTexts {
    pub fn reset_title_times(&mut self) {
        self.title_fade_in_time = 10;
        self.title_stay_time = 70;
        self.title_fade_out_time = 20;
    }

    /// A negative time keeps the one before.
    pub fn set_times(&mut self, fade_in_time: i32, stay_time: i32, fade_out_time: i32) {
        if fade_in_time >= 0 {
            self.title_fade_in_time = fade_in_time;
        }
        if stay_time >= 0 {
            self.title_stay_time = stay_time;
        }
        if fade_out_time >= 0 {
            self.title_fade_out_time = fade_out_time;
        }
        if self.title_time > 0 {
            self.title_time = self.title_total_time();
        }
    }

    pub fn set_title(&mut self, title: Component) {
        self.title = Some(title);
        self.title_time = self.title_total_time();
    }

    pub fn clear_titles(&mut self) {
        self.title = None;
        self.subtitle = None;
        self.title_time = 0;
    }

    // Java's int sum, which wraps.
    fn title_total_time(&self) -> i32 {
        self.title_fade_in_time
            .wrapping_add(self.title_stay_time)
            .wrapping_add(self.title_fade_out_time)
    }
}

impl ClientGamePacketListener for ClientPacketListener {
    fn handle_login(&mut self, packet: &ClientboundLoginPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let this = self
            .this
            .upgrade()
            .expect("the listener handling a packet is alive");
        *minecraft.game_mode.borrow_mut() =
            Some(Rc::new(MultiPlayerGameMode::new(Rc::clone(&this))));
        let spawn_info = packet.common_player_spawn_info();
        // Java shuffles them with a random it seeds from the clock, so their order is never the
        // same twice; nothing reads it.
        self.levels = packet.levels().keys().to_vec();
        self.server_chunk_radius = packet.chunk_radius();
        self.server_simulation_distance = packet.simulation_distance();
        let mut level = ClientLevel::new(
            &self.registry_access,
            packet.hardcore(),
            spawn_info.is_flat(),
            spawn_info.dimension(),
            spawn_info.dimension_type(),
            self.server_chunk_radius,
            self.server_simulation_distance,
            spawn_info.is_debug(),
            spawn_info.seed(),
            spawn_info.sea_level(),
        );
        // What Java's level reads from the game's options when it blends tints.
        level.set_biome_blend_radius(minecraft.options.borrow().biome_blend_radius());
        minecraft.set_level(level);
        if minecraft.player.borrow().is_none() {
            let mut player = LocalPlayer::new(Some(this));
            // `Player`'s constructor: the local profile's id (`AbstractClientPlayer`).
            player.entity_mut().set_uuid(self.local_game_profile.id());
            player.set_y_rot(-180.0);
            *minecraft.player.borrow_mut() = Some(player);
        }

        self.set_client_loaded(false);
        {
            let mut player = minecraft.player.borrow_mut();
            let player = player.as_mut().expect("the player was just made");
            player.reset_pos();
            player.set_id(packet.player_id());
            minecraft.set_camera_entity(Some(player.get_id()));
        }
        if let Some(game_mode) = minecraft.game_mode.borrow().as_ref() {
            game_mode.set_local_mode(spawn_info.game_type(), spawn_info.previous_game_type());
        }
        self.start_waiting_for_new_level();
        minecraft
            .options
            .borrow_mut()
            .set_server_render_distance(packet.chunk_radius());
        self.online_mode = packet.online_mode();
        self.server_enforces_secure_chat = packet.enforces_secure_chat();
    }

    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let mut player = minecraft.player.borrow_mut();
        let player = player.as_mut().expect("a teleport comes after the login");
        if !player.is_passenger() {
            ClientPacketListener::set_values_from_position_packet(
                &packet.change(),
                &packet.relatives(),
                player,
            );
        }

        self.common.connection.send::<dyn ServerGamePacketListener>(
            ServerboundAcceptTeleportationPacket::new(
                packet.id(),
                player.get_x(),
                player.get_y(),
                player.get_z(),
                player.get_y_rot(),
                player.get_x_rot(),
            ),
        );
    }

    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let event = packet.get_event();
        let param_float = packet.get_param();
        if let Some(event) = event
            && [
                ClientboundGameEventPacket::START_RAINING,
                ClientboundGameEventPacket::STOP_RAINING,
                ClientboundGameEventPacket::RAIN_LEVEL_CHANGE,
                ClientboundGameEventPacket::THUNDER_LEVEL_CHANGE,
            ]
            .contains(&event)
        {
            let mut level = minecraft.level.borrow_mut();
            let Some(level) = level.as_mut() else {
                return;
            };
            if event == ClientboundGameEventPacket::START_RAINING {
                level.set_rain_level(0.0);
            } else if event == ClientboundGameEventPacket::STOP_RAINING {
                level.set_rain_level(1.0);
            } else if event == ClientboundGameEventPacket::RAIN_LEVEL_CHANGE {
                level.set_rain_level(param_float);
            } else {
                level.set_thunder_level(param_float);
            }
        } else if event == Some(ClientboundGameEventPacket::LEVEL_CHUNKS_LOAD_START)
            && let Some(level_load_tracker) = self.level_load_tracker.as_mut()
        {
            level_load_tracker.loading_packets_received();
        } else if event == Some(ClientboundGameEventPacket::CHANGE_GAME_MODE)
            && let Some(game_mode) = minecraft.game_mode.borrow().as_ref()
        {
            let param = Mth::floor_f32(packet.get_param() + 0.5);
            game_mode.set_local_mode(
                crate::net::minecraft::world::level::game_type::GameType::by_id(param),
                None,
            );
        }
    }

    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.clear_level();
        minecraft.clear_client_level();
        let cookie = CommonListenerCookie {
            level_load_tracker: LevelLoadTracker::new(),
            local_game_profile: self.local_game_profile.clone(),
            received_registries: Arc::clone(&self.registry_access),
            enabled_features: self.enabled_features.clone(),
            server_brand: self.common.server_brand.clone(),
            server_data: self.common.server_data.clone(),
            server_cookies: self.common.server_cookies.clone(),
            custom_report_details: self.common.custom_report_details.clone(),
            server_links: self.common.server_links.clone(),
        };
        let connection = Arc::clone(&self.common.connection);
        let listener = ClientConfigurationPacketListenerImpl::new(
            Rc::clone(&minecraft),
            Arc::clone(&connection),
            cookie,
        );
        connection
            .setup_inbound_protocol(Box::new(&***ConfigurationProtocols::CLIENTBOUND), listener);
        connection.send::<dyn ServerGamePacketListener>(ServerboundConfigurationAcknowledgedPacket);
        connection.setup_outbound_protocol(Box::new(&***ConfigurationProtocols::SERVERBOUND));
    }

    /// Its packets one after the other, on the game thread: the ones Java handles on the Netty
    /// thread too, since they're the bundle's.
    fn handle_bundle_packet(&mut self, packet: &ClientboundBundlePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }

        for sub_packet in packet.sub_packets() {
            sub_packet.handle(self);
        }
    }

    /// The chunk goes in the level's chunk storage. Its light isn't kept yet (`applyLightData`
    /// and `enableChunkLight` queue it for the light engine, which isn't ported), nor are its
    /// block entities.
    fn handle_level_chunk_with_light(&mut self, packet: &ClientboundLevelChunkWithLightPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let start = bench::now();
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            ClientPacketListener::handle_level_chunk_with_light_in(level, packet);
        }
        bench::record(bench::Kind::Chunk, start);
    }

    /// The light the server sent for the chunk, queued for the level.
    fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            ClientPacketListener::handle_light_update_packet_in(level, packet);
        }
    }

    /// The chunk leaves the level's chunk storage, and its light's removal is queued. Java also
    /// drops the debug subscriptions' data for it, which isn't ported.
    fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            ClientPacketListener::handle_forget_level_chunk_in(level, packet);
        }
    }

    fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.set_server_verified_block_state(&packet.get_pos(), packet.get_block_state(), 19);
        }
    }

    /// The block entity there of the packet's type (made if its block makes one), loaded from
    /// the tag with the client's registries.
    fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let pos = packet.get_pos();
        if let Some(level) = minecraft.level.borrow_mut().as_mut()
            && let Some(block_entity) = level.get_block_entity_typed(&pos, packet.get_type())
        {
            let reporter = ScopedCollector::new(block_entity.get_name_for_reporting());
            block_entity.load_with_components(&TagValueInput::create(
                &reporter,
                &self.registry_access,
                packet.get_tag(),
            ));
        }
    }

    /// Each change, in order. A state the registry doesn't have is null in Java, where setting
    /// it throws; it's skipped here.
    fn handle_chunk_blocks_update(&mut self, packet: &ClientboundSectionBlocksUpdatePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            packet.run_updates(&mut |pos, state| {
                if let Some(state) = state {
                    level.set_server_verified_block_state(&pos, state, 19);
                }
            });
        }
    }

    /// Each chunk's biomes, read into the chunk the level has.
    fn handle_chunks_biomes(&mut self, packet: &ClientboundChunksBiomesPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            ClientPacketListener::handle_chunks_biomes_in(level, packet);
        }
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket) {
        self.shared.handle_chunk_batch_start(packet);
    }

    /// On the Netty thread, like Java's (`NettyHalf`): only a bundle's reaches the game thread.
    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket) {
        self.shared
            .handle_chunk_batch_finished(&self.common.connection, packet);
    }

    fn handle_change_difficulty(&mut self, packet: &ClientboundChangeDifficultyPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java sets the level data's difficulty and whether it's locked, and tells a screen that shows
        // them; the level data is the world track's, and the screens aren't ported.
        let _ = packet;
    }

    fn handle_initialize_border(&mut self, packet: &ClientboundInitializeBorderPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            let game_time = level.get_game_time();
            let border = level.get_world_border_mut();
            border.set_center(packet.get_new_center_x(), packet.get_new_center_z());
            let lerp_time = packet.get_lerp_time();
            if lerp_time > 0 {
                border.lerp_size_between(
                    packet.get_old_size(),
                    packet.get_new_size(),
                    lerp_time,
                    game_time,
                );
            } else {
                border.set_size(packet.get_new_size());
            }

            border.set_absolute_max_size(packet.get_new_absolute_max_size());
            border.set_warning_blocks(packet.get_warning_blocks());
            border.set_warning_time(packet.get_warning_time());
        }
    }

    fn handle_set_border_center(&mut self, packet: &ClientboundSetBorderCenterPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level
                .get_world_border_mut()
                .set_center(packet.get_new_center_x(), packet.get_new_center_z());
        }
    }

    fn handle_set_border_lerp_size(&mut self, packet: &ClientboundSetBorderLerpSizePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            let game_time = level.get_game_time();
            level.get_world_border_mut().lerp_size_between(
                packet.get_old_size(),
                packet.get_new_size(),
                packet.get_lerp_time(),
                game_time,
            );
        }
    }

    fn handle_set_border_size(&mut self, packet: &ClientboundSetBorderSizePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.get_world_border_mut().set_size(packet.get_size());
        }
    }

    fn handle_set_border_warning_delay(&mut self, packet: &ClientboundSetBorderWarningDelayPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level
                .get_world_border_mut()
                .set_warning_time(packet.get_warning_delay());
        }
    }

    fn handle_set_border_warning_distance(
        &mut self,
        packet: &ClientboundSetBorderWarningDistancePacket,
    ) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level
                .get_world_border_mut()
                .set_warning_blocks(packet.get_warning_blocks());
        }
    }

    fn handle_player_abilities(&mut self, packet: &ClientboundPlayerAbilitiesPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java copies them into the player's `Abilities`, which the player track ports with movement.
        let _ = packet;
    }

    fn handle_set_chunk_cache_center(&mut self, packet: &ClientboundSetChunkCacheCenterPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level
                .get_chunk_source_mut()
                .update_view_center(packet.get_x(), packet.get_z());
        }
    }

    fn handle_set_chunk_cache_radius(&mut self, packet: &ClientboundSetChunkCacheRadiusPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.server_chunk_radius = packet.get_radius();
        minecraft
            .options
            .borrow_mut()
            .set_server_render_distance(self.server_chunk_radius);
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level
                .get_chunk_source_mut()
                .update_view_radius(self.server_chunk_radius);
        }
    }

    fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(player) = minecraft.player.borrow_mut().as_mut() {
            player.set_experience_values(
                packet.get_experience_progress(),
                packet.get_total_experience(),
                packet.get_experience_level(),
            );
        }
    }

    fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(player) = minecraft.player.borrow_mut().as_mut() {
            player.hurt_to(packet.get_health());
            player.food_level = packet.get_food();
            // The HUD's food jitters when the saturation runs out.
            player.saturation_level = packet.get_saturation();
        }
    }

    fn handle_set_held_slot(&mut self, packet: &ClientboundSetHeldSlotPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java selects the slot in the player's inventory when it's a hotbar slot; the inventory
        // isn't ported yet.
        let _ = packet;
    }

    fn handle_set_simulation_distance(&mut self, packet: &ClientboundSetSimulationDistancePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.server_simulation_distance = packet.simulation_distance();
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.set_server_simulation_distance(self.server_simulation_distance);
        }
    }

    fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java sets the level's `TickRateManager`, whose rate the game's ticks follow; it isn't ported
        // yet, so the client ticks 20 times a second, the rate every server starts at.
        let _ = packet;
    }

    fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java sets the level's `TickRateManager`'s frozen ticks to run, which isn't ported yet.
        let _ = packet;
    }

    fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let game_time = packet.game_time();
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.set_time_from_server(game_time);
        }
        // Java also tells the telemetry the time, which isn't ported.
        self.clock_manager
            .handle_updates(game_time, &packet.clock_updates());
        // Java also invalidates the level's environment attributes' tick cache, which come with
        // the renderer's sky (`EnvironmentAttributeSystem`).
    }

    fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // The player is the only entity ported; Java looks the entity up in the level, and does
        // nothing for one it doesn't have.
        let mut player = minecraft.player.borrow_mut();
        let Some(player) = player
            .as_mut()
            .filter(|player| player.get_id() == packet.get_entity_id())
        else {
            return;
        };
        let attributes = &mut player.living_entity_mut().attributes;
        for attribute in packet.get_values() {
            // An attribute the port hasn't got is one the player's map hasn't got either.
            let Some(instance) = Attributes::by_holder(&attribute.attribute())
                .and_then(|holder| attributes.get_instance_mut(holder))
            else {
                let Holder::Reference(key) = attribute.attribute();
                eprintln!(
                    "Entity {} does not have attribute {}",
                    packet.get_entity_id(),
                    key.identifier()
                );
                continue;
            };
            instance.set_base_value(attribute.base());
            instance.remove_modifiers();

            for modifier in attribute.modifiers() {
                instance.add_transient_modifier(modifier);
            }
        }
    }

    fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.set_respawn_data(packet.respawn_data());
        }
    }

    fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Like the attributes, only the player's. `SynchedEntityData` isn't ported, so
        // `assignValues` sets what the player has of its items: the shared flags (0, sprinting and
        // crouching among them), the pose (6), which resizes it (`onSyncedDataUpdated`), and its
        // main arm (15) and skin parts shown (16), which its hands are drawn with. Its
        // health (9) goes through `set_health` packets only, since `setHealth` clamps where the
        // item's assignment doesn't; the other items aren't ported. A value whose type isn't
        // ported ends the packet's values (ClientboundSetEntityDataPacket).
        let mut player = minecraft.player.borrow_mut();
        let Some(player) = player
            .as_mut()
            .filter(|player| player.get_id() == packet.id())
        else {
            return;
        };
        // `Entity.DATA_SHARED_FLAGS_ID`'s and `DATA_POSE`'s ids, the first items `defineId` gives,
        // and `Avatar`'s main arm and skin parts, after `Entity`'s 8 and `LivingEntity`'s 7.
        const DATA_SHARED_FLAGS_ID: i32 = 0;
        const DATA_POSE: i32 = 6;
        const DATA_PLAYER_MAIN_HAND: i32 = 15;
        const DATA_PLAYER_MODE_CUSTOMISATION: i32 = 16;
        for item in packet.packed_items() {
            match (item.id(), item.value()) {
                (DATA_SHARED_FLAGS_ID, EntityDataValue::Byte(flags)) => {
                    for flag in 0..8 {
                        player
                            .entity_mut()
                            .set_shared_flag(flag, flags & (1 << flag) != 0);
                    }
                }
                (DATA_POSE, EntityDataValue::Pose(pose)) => entity::set_pose(player, *pose),
                (DATA_PLAYER_MAIN_HAND, EntityDataValue::HumanoidArm(arm)) => {
                    player.main_arm = *arm;
                }
                (DATA_PLAYER_MODE_CUSTOMISATION, EntityDataValue::Byte(parts)) => {
                    player.model_customisation = *parts;
                }
                _ => {}
            }
        }
    }

    fn handle_recipe_book_settings(&mut self, packet: &ClientboundRecipeBookSettingsPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java sets them on the player's recipe book and rebuilds its collections, which the
        // recipe book's screen shows (`refreshRecipeBook`); neither is ported.
        self.recipe_book_settings = packet.book_settings();
    }

    fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java looks the entity up in the level, whose only entity so far is the player.
        if let Some(player) = minecraft
            .player
            .borrow_mut()
            .as_mut()
            .filter(|player| player.get_id() == packet.id())
        {
            player.entity_mut().lerp_motion(packet.movement());
        }
    }

    fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java looks the entity up in the level, whose only entity so far is the player, and
        // starts its hurt animation (`animateHurt`: the timers, and a player's `hurtDir`, which
        // only rendering reads).
        if let Some(player) = minecraft
            .player
            .borrow_mut()
            .as_mut()
            .filter(|player| player.get_id() == packet.id())
        {
            let living = player.living_entity_mut();
            living.hurt_duration = 10;
            living.hurt_time = living.hurt_duration;
        }
    }

    fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java looks the entity up in the level (`getEntity`) and plays the event: a guardian's
        // attack sound (21), a totem's particles, sound and item activation (35), a sniffer's
        // sounds (63), and any other through the entity's `handleEntityEvent`. The player's are
        // its permission level (24 to 28, which the server sends as it joins), using an item, the
        // reduced debug info and the death animation, none of which movement reads: not ported
        // yet.
        let _ = packet;
    }

    fn handle_damage_event(&mut self, packet: &ClientboundDamageEventPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java looks the entity up in the level, whose only entity so far is the player, and hands
        // it the damage source the packet makes of the level's entities (`getSource`), which only
        // the hurt sound and the last damage source read: the timers are what's ported.
        if let Some(player) = minecraft
            .player
            .borrow_mut()
            .as_mut()
            .filter(|player| player.get_id() == packet.entity_id())
        {
            player.handle_damage_event();
        }
    }

    fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java forgets the removed vehicle's id when the entity has it (riding isn't ported),
        // makes the entity of its type (`createEntityFromPacket`: an entity of a class not ported
        // yet is a plain `Entity` of the type, a player too, where Java makes a `RemotePlayer`),
        // sets it up from the packet, adds it to the level, plays its sound (a minecart's, a
        // bee's) and marks a player it has the info of as seen (sounds and the player list aren't
        // ported).
        let r#type = packet.get_type();
        let mut entity = Entity::new(r#type, r#type.get_dimensions());
        entity.recreate_from_packet(packet);
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            level.add_entity(entity);
        }
    }

    fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // Java also keeps the id of one the player rides (`removedPlayerVehicleId`) and drops it
        // from the debug subscriptions, neither ported.
        if let Some(level) = minecraft.level.borrow_mut().as_mut() {
            for entity_id in packet.entity_ids() {
                level.remove_entity(entity_id);
            }
        }
    }

    /// `ChatListener.handleSystemMessage`: a chat line, which `ChatComponent.addMessage` logs with
    /// its line breaks escaped, or the action bar's text for an overlay.
    fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        // ChatListener.handleSystemMessage: the action bar's overlay, or a line in the chat.
        if packet.overlay() {
            self.texts.action_bar = Some(packet.content());
        } else {
            minecraft.add_server_system_message(packet.content());
        }
    }

    fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.set_title(packet.text());
    }

    fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.set_times(
            packet.get_fade_in(),
            packet.get_stay(),
            packet.get_fade_out(),
        );
    }

    fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.clear_titles();
        if packet.should_reset_times() {
            self.texts.reset_title_times();
        }
    }

    fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.subtitle = Some(packet.text());
    }

    fn set_action_bar_text(&mut self, packet: &ClientboundSetActionBarTextPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.action_bar = Some(packet.text());
    }

    /// `Gui.getTabList()`'s header and footer, an empty text clearing either.
    fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        let header = packet.header();
        let footer = packet.footer();
        self.texts.tab_list_header = (!header.get_string().is_empty()).then_some(header);
        self.texts.tab_list_footer = (!footer.get_string().is_empty()).then_some(footer);
    }

    /// The server's message of the day, which Java puts on its `ServerData` (with the icon, not
    /// kept here) and saves to the server list.
    fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket) {
        let minecraft = Rc::clone(&self.common.minecraft);
        if PacketUtils::ensure_running_on_same_thread(
            packet,
            &self.this,
            minecraft.packet_processor(),
        )
        .is_err()
        {
            return;
        }
        self.texts.motd = Some(packet.motd());
    }
}

impl ClientGamePacketListener for NettyHalf {
    fn handle_login(&mut self, packet: &ClientboundLoginPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_move_player(&mut self, packet: &ClientboundPlayerPositionPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_game_event(&mut self, packet: &ClientboundGameEventPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_configuration_start(&mut self, packet: &ClientboundStartConfigurationPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_chunk_batch_start(&mut self, packet: &ClientboundChunkBatchStartPacket) {
        self.shared.handle_chunk_batch_start(packet);
    }

    fn handle_chunk_batch_finished(&mut self, packet: &ClientboundChunkBatchFinishedPacket) {
        self.shared
            .handle_chunk_batch_finished(&self.common.connection, packet);
    }

    fn handle_bundle_packet(&mut self, packet: &ClientboundBundlePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_change_difficulty(&mut self, packet: &ClientboundChangeDifficultyPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_initialize_border(&mut self, packet: &ClientboundInitializeBorderPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_border_center(&mut self, packet: &ClientboundSetBorderCenterPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_border_lerp_size(&mut self, packet: &ClientboundSetBorderLerpSizePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_border_size(&mut self, packet: &ClientboundSetBorderSizePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_border_warning_delay(&mut self, packet: &ClientboundSetBorderWarningDelayPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_border_warning_distance(
        &mut self,
        packet: &ClientboundSetBorderWarningDistancePacket,
    ) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_player_abilities(&mut self, packet: &ClientboundPlayerAbilitiesPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_chunk_cache_center(&mut self, packet: &ClientboundSetChunkCacheCenterPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_chunk_cache_radius(&mut self, packet: &ClientboundSetChunkCacheRadiusPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_experience(&mut self, packet: &ClientboundSetExperiencePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_health(&mut self, packet: &ClientboundSetHealthPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_held_slot(&mut self, packet: &ClientboundSetHeldSlotPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_simulation_distance(&mut self, packet: &ClientboundSetSimulationDistancePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_ticking_state(&mut self, packet: &ClientboundTickingStatePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_ticking_step(&mut self, packet: &ClientboundTickingStepPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_level_chunk_with_light(&mut self, packet: &ClientboundLevelChunkWithLightPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_light_update_packet(&mut self, packet: &ClientboundLightUpdatePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_forget_level_chunk(&mut self, packet: &ClientboundForgetLevelChunkPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_block_update(&mut self, packet: &ClientboundBlockUpdatePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_block_entity_data(&mut self, packet: &ClientboundBlockEntityDataPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_chunk_blocks_update(&mut self, packet: &ClientboundSectionBlocksUpdatePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_chunks_biomes(&mut self, packet: &ClientboundChunksBiomesPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_time(&mut self, packet: &ClientboundSetTimePacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_update_attributes(&mut self, packet: &ClientboundUpdateAttributesPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_spawn(&mut self, packet: &ClientboundSetDefaultSpawnPositionPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_entity_data(&mut self, packet: &ClientboundSetEntityDataPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_recipe_book_settings(&mut self, packet: &ClientboundRecipeBookSettingsPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_set_entity_motion(&mut self, packet: &ClientboundSetEntityMotionPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_hurt_animation(&mut self, packet: &ClientboundHurtAnimationPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_entity_event(&mut self, packet: &ClientboundEntityEventPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_damage_event(&mut self, packet: &ClientboundDamageEventPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_add_entity(&mut self, packet: &ClientboundAddEntityPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_remove_entities(&mut self, packet: &ClientboundRemoveEntitiesPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_system_chat(&mut self, packet: &ClientboundSystemChatPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn set_title_text(&mut self, packet: &ClientboundSetTitleTextPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn set_titles_animation(&mut self, packet: &ClientboundSetTitlesAnimationPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_titles_clear(&mut self, packet: &ClientboundClearTitlesPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn set_subtitle_text(&mut self, packet: &ClientboundSetSubtitleTextPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn set_action_bar_text(&mut self, packet: &ClientboundSetActionBarTextPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_tab_list_customisation(&mut self, packet: &ClientboundTabListPacket) {
        self.common.on_game_thread(&self.this, packet);
    }

    fn handle_server_data(&mut self, packet: &ClientboundServerDataPacket) {
        self.common.on_game_thread(&self.this, packet);
    }
}
