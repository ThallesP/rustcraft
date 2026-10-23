//! The offline replay, which has no Java file: a scenario's ticks run on the port's `LocalPlayer`
//! without a server, the way the oracle client runs them in vanilla (`OracleGameTest.replay`),
//! giving the trace and the serverbound packets a run on a server gives, to compare with
//! vanilla's `trace.jsonl` and `capture.jsonl` (TESTING.md).
//!
//! The level is a `ClientLevel` filled from a capture the way the client's listener fills it:
//! the registries from configuration, the login's level, then every chunk, forgotten chunk, block
//! update, change of the view and added or removed entity before the scenario-start marker, in the
//! order they came. The
//! port's own captures (CI records the walk's with `tools/port_client.py capture`) have the same
//! chunks as vanilla's, which the server sends either way. Without a capture, the level is the
//! flat preset's (`FlatLevel`).
//!
//! A tick is what `Minecraft.tick` does for the player, in its order: `level.tickEntities()`
//! (`ClientLevel.tickNonPassenger`: `commonTick`, then the player's `tick`), then
//! `player.sendChanges()`, then `client_tick_end`. Before the marker, the player is put where the
//! scenario says and left to land like the oracle client's settling (20 ticks, then until it's on
//! the ground), sending what it sends then, which the comparison leaves out like `oracle-diff`
//! leaves out the join's per-tick packets. From the marker on, the packets are compared strictly,
//! each by its name and its payload (`capture.jsonl`'s `name` and `payload`).
//!
//! The server's answers after the marker aren't replayed, but at a step's `wait_for`: elsewhere
//! they're its echoes of the player's own state (`set_entity_data`'s flags and pose,
//! `update_attributes`' sprinting modifier), which change nothing the client has. Where a step waits
//! for the server's answer (TESTING.md), the live clients handle what has come by then after the
//! tick's trace line, before the next tick; the replay takes the capture's clientbound packets in
//! the order they came, up to the waited-for ones, and applies those that change the player: its
//! motion
//! (`set_entity_motion`, `lerpMotion`) and its health and food (`set_health`, `hurtTo`), and the
//! damage event's hurt timers (`damage_event`). A scenario that waits can only be replayed on a
//! capture.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;

use crate::io::netty::buffer::unpooled::{self, wrapped_buffer};
use crate::java::util::uuid::UUID;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::user::User;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::custom::discarded_payload::DiscardedPayload;
use crate::net::minecraft::network::protocol::common::serverbound_custom_payload_packet::ServerboundCustomPayloadPacket;
use crate::net::minecraft::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use crate::net::minecraft::network::protocol::game::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use crate::net::minecraft::network::protocol::game::clientbound_block_update_packet::ClientboundBlockUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_damage_event_packet::ClientboundDamageEventPacket;
use crate::net::minecraft::network::protocol::game::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
use crate::net::minecraft::network::protocol::game::clientbound_login_packet::ClientboundLoginPacket;
use crate::net::minecraft::network::protocol::game::clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket;
use crate::net::minecraft::network::protocol::game::clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_entity_motion_packet::ClientboundSetEntityMotionPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_health_packet::ClientboundSetHealthPacket;
use crate::net::minecraft::network::protocol::game::game_protocols::{self, GameProtocols};
use crate::net::minecraft::network::protocol::game::server_game_packet_listener::ServerGamePacketListener;
use crate::net::minecraft::network::protocol::game::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::entity::entity::{Entity, EntityClass};
use crate::net::minecraft::world::level::block::blocks::{self, Blocks};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::border::world_border::WorldBorder;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::rustcraft::oracle::client::oracle_game_test::OracleGameTest;
use crate::rustcraft::oracle::client::scenario::Scenario;
use crate::rustcraft::oracle::client::trace_writer;

/// One packet of a `capture.jsonl` (TESTING.md).
#[derive(Clone, Debug)]
pub struct CapturedPacket {
    pub connection: i64,
    pub serverbound: bool,
    pub state: String,
    pub name: String,
    pub payload: Vec<i8>,
}

impl CapturedPacket {
    /// `name payload`, the line a packet is compared by.
    pub fn line(&self) -> String {
        format!("{} {}", self.name, hex(&self.payload))
    }

    fn is_marker(&self) -> bool {
        self.serverbound
            && self.name == "minecraft:custom_payload"
            && self.payload == marker_payload()
    }
}

/// A packet capture, in the order `packet-proxy` saw the packets.
#[derive(Clone, Debug)]
pub struct Capture {
    pub packets: Vec<CapturedPacket>,
}

impl Capture {
    pub fn load(path: &Path) -> Option<Capture> {
        let text = std::fs::read_to_string(path).ok()?;
        let packets = text
            .lines()
            .map(|line| {
                let packet: Value = serde_json::from_str(line).expect("a capture line is JSON");
                CapturedPacket {
                    connection: packet["connection"].as_i64().expect("connection"),
                    serverbound: packet["direction"] == "serverbound",
                    state: packet["state"].as_str().expect("state").to_string(),
                    name: packet["name"].as_str().expect("name").to_string(),
                    payload: unhex(packet["payload"].as_str().expect("payload")),
                }
            })
            .collect();
        Some(Capture { packets })
    }

    /// The scenario-start marker's index.
    fn marker(&self) -> usize {
        self.packets
            .iter()
            .position(CapturedPacket::is_marker)
            .expect("the capture has the scenario-start marker")
    }

    /// The serverbound packets from the marker on, on its connection: what a scenario compares
    /// strictly, leaving out the ones `oracle-diff` leaves out by default (`DEFAULT_IGNORED`),
    /// which answer the server's clock and its chunk batches, and which the replay doesn't send.
    pub fn serverbound_from_marker(&self) -> Vec<String> {
        const IGNORED: [&str; 4] = [
            "minecraft:keep_alive",
            "minecraft:pong",
            "minecraft:ping_request",
            "minecraft:chunk_batch_received",
        ];
        let marker = self.marker();
        let connection = self.packets[marker].connection;
        self.packets[marker..]
            .iter()
            .filter(|packet| {
                packet.serverbound
                    && packet.connection == connection
                    && !IGNORED.contains(&packet.name.as_str())
            })
            .map(CapturedPacket::line)
            .collect()
    }

    /// The clientbound packets of the game protocol from the marker on, on its connection: the
    /// server's answers during the scenario.
    pub fn play_from_marker(&self) -> Vec<CapturedPacket> {
        let marker = self.marker();
        let connection = self.packets[marker].connection;
        self.packets[marker..]
            .iter()
            .filter(|packet| {
                !packet.serverbound && packet.state == "play" && packet.connection == connection
            })
            .cloned()
            .collect()
    }

    /// The clientbound packets of the game protocol before the marker, on its connection, by
    /// name.
    fn play_before_marker(&self) -> impl Iterator<Item = &CapturedPacket> {
        let marker = self.marker();
        let connection = self.packets[marker].connection;
        self.packets[..marker].iter().filter(move |packet| {
            !packet.serverbound && packet.state == "play" && packet.connection == connection
        })
    }
}

/// Where a scenario's capture is: vanilla's (`vanilla/<version>/oracle/<scenario>/`), or else the
/// port's (`target/oracle/<scenario>/`), whose chunks are the same server's.
pub fn capture_path(scenario: &str) -> Option<PathBuf> {
    let vanilla = vanilla_dir(scenario).join("capture.jsonl");
    let port = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/oracle")
        .join(scenario)
        .join("capture.jsonl");
    [vanilla, port].into_iter().find(|path| path.exists())
}

/// `vanilla/<version>/oracle/<scenario>/`.
pub fn vanilla_dir(scenario: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "vanilla/{}/oracle/{scenario}",
        include_str!("../../../../MINECRAFT_VERSION").trim()
    ))
}

/// The level the client had when it sent the marker, with the registries the game protocol
/// reads with and the player's entity id.
pub struct CapturedLevel {
    pub level: ClientLevel,
    pub registries: Arc<RegistryAccess>,
    pub player_id: i32,
}

impl CapturedLevel {
    /// What `ClientConfigurationPacketListenerImpl` collects, then what `ClientPacketListener`'s
    /// handlers do with the level's packets: `handleLogin` makes the level, and the chunk, view and
    /// block handlers change it, in the order the packets came.
    pub fn from_capture(capture: &Capture) -> CapturedLevel {
        let mut collector = RegistryDataCollector::new();
        for packet in &capture.packets {
            if !packet.serverbound
                && packet.state == "configuration"
                && packet.name == "minecraft:registry_data"
            {
                let mut input = friendly(&packet.payload);
                let packet = ClientboundRegistryDataPacket::STREAM_CODEC.decode(&mut input);
                collector.append_contents(packet.registry(), packet.entries());
            }
        }
        let registries = Arc::new(collector.collect_game_registries());
        let registry_buffer = |payload: &[i8]| {
            RegistryFriendlyByteBuf::new(wrapped_buffer(payload.to_vec()), Arc::clone(&registries))
        };

        let mut level: Option<ClientLevel> = None;
        let mut player_id = 0;
        for packet in capture.play_before_marker() {
            if packet.name == "minecraft:login" {
                let login = ClientboundLoginPacket::STREAM_CODEC
                    .decode(&mut registry_buffer(&packet.payload));
                let spawn_info = login.common_player_spawn_info();
                player_id = login.player_id();
                level = Some(ClientLevel::new(
                    &registries,
                    login.hardcore(),
                    spawn_info.is_flat(),
                    spawn_info.dimension(),
                    spawn_info.dimension_type(),
                    login.chunk_radius(),
                    login.simulation_distance(),
                    spawn_info.is_debug(),
                    spawn_info.seed(),
                    spawn_info.sea_level(),
                ));
                continue;
            }
            let Some(level) = level.as_mut() else {
                continue;
            };
            match packet.name.as_str() {
                "minecraft:level_chunk_with_light" => {
                    let chunk = ClientboundLevelChunkWithLightPacket::STREAM_CODEC
                        .decode(&mut registry_buffer(&packet.payload));
                    level.get_chunk_source_mut().replace_with_packet_data(
                        chunk.x(),
                        chunk.z(),
                        chunk.chunk_data(),
                    );
                }
                "minecraft:forget_level_chunk" => {
                    let forget = ClientboundForgetLevelChunkPacket::STREAM_CODEC
                        .decode(&mut friendly(&packet.payload));
                    level.get_chunk_source_mut().drop(forget.pos());
                }
                "minecraft:block_update" => {
                    let update = ClientboundBlockUpdatePacket::STREAM_CODEC
                        .decode(&mut registry_buffer(&packet.payload));
                    level.set_server_verified_block_state(
                        &update.get_pos(),
                        update.get_block_state(),
                        19,
                    );
                }
                "minecraft:section_blocks_update" => {
                    let update = ClientboundSectionBlocksUpdatePacket::STREAM_CODEC
                        .decode(&mut friendly(&packet.payload));
                    update.run_updates(&mut |pos, state| {
                        if let Some(state) = state {
                            level.set_server_verified_block_state(&pos, state, 19);
                        }
                    });
                }
                "minecraft:set_chunk_cache_center" => {
                    let center = ClientboundSetChunkCacheCenterPacket::STREAM_CODEC
                        .decode(&mut friendly(&packet.payload));
                    level
                        .get_chunk_source_mut()
                        .update_view_center(center.get_x(), center.get_z());
                }
                "minecraft:add_entity" => {
                    let add = ClientboundAddEntityPacket::STREAM_CODEC
                        .decode(&mut registry_buffer(&packet.payload));
                    if add.get_id() != player_id {
                        let r#type = add.get_type();
                        let mut entity = Entity::new(r#type, r#type.get_dimensions());
                        entity.recreate_from_packet(&add);
                        level.add_entity(entity);
                    }
                }
                "minecraft:remove_entities" => {
                    let remove = ClientboundRemoveEntitiesPacket::STREAM_CODEC
                        .decode(&mut unpooled::wrapped_buffer(packet.payload.to_vec()));
                    for entity_id in remove.entity_ids() {
                        level.remove_entity(entity_id);
                    }
                }
                "minecraft:set_chunk_cache_radius" => {
                    let radius = ClientboundSetChunkCacheRadiusPacket::STREAM_CODEC
                        .decode(&mut friendly(&packet.payload));
                    level
                        .get_chunk_source_mut()
                        .update_view_radius(radius.get_radius());
                }
                _ => {}
            }
        }
        CapturedLevel {
            level: level.expect("the capture has the login"),
            registries,
            player_id,
        }
    }
}

/// A scenario's world without a capture: the flat preset (bedrock at y = -64, two dirt, grass at
/// -61, air above) with the blocks the scenario's setup commands place, every chunk loaded, the
/// entities its `summon`s make, the default border.
///
/// The commands are `fill` and `setblock` in their `strict` mode, which places the states as they
/// are written, with no shape updates from or to their neighbours (`FillCommand`'s flags 816), so
/// the blocks are the commands' own. Where vanilla's capture is, the tests check they're the ones
/// its server sent. A `summon` makes an entity of its type where it says, which is where the
/// server adds it, in the order the server sends them; it's the client's as long as no entity's
/// class ticks it anywhere (none is ported), and its data (`NoAI`, `NoGravity`) is the server's.
pub struct FlatLevel {
    blocks: HashMap<BlockPos, BlockState>,
    entities: Vec<Entity>,
    world_border: WorldBorder,
}

impl FlatLevel {
    pub fn new() -> FlatLevel {
        FlatLevel {
            blocks: HashMap::new(),
            entities: Vec::new(),
            world_border: WorldBorder::new(),
        }
    }

    /// The entities the setup summoned, in order.
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// The flat preset with the commands' blocks, in their order, and the entities they summon; a
    /// `gamerule` places none. Panics on a command it doesn't know or one that isn't strict.
    pub fn with_setup(commands: &[String]) -> FlatLevel {
        let mut level = FlatLevel::new();
        for command in commands {
            let words: Vec<&str> = command.split(' ').collect();
            if words.first() == Some(&"gamerule") {
                continue;
            }
            if let ["summon", r#type, x, y, z, ..] = words.as_slice() {
                let position = |word: &str| -> f64 {
                    word.parse()
                        .unwrap_or_else(|_| panic!("{command}: {word} isn't a position"))
                };
                let r#type = BuiltInRegistries::ENTITY_TYPE
                    .get_optional(&Identifier::parse(r#type))
                    .unwrap_or_else(|| panic!("{command}: no entity type {type}"));
                let mut entity = Entity::new(r#type, r#type.get_dimensions());
                entity.snap_to(position(x), position(y), position(z), 0.0, 0.0);
                level.entities.push(entity);
                continue;
            }
            let coordinate = |word: &str| -> i32 {
                word.parse()
                    .unwrap_or_else(|_| panic!("{command}: {word} isn't a block coordinate"))
            };
            let (from, to, state, mode) = match words.as_slice() {
                ["fill", x1, y1, z1, x2, y2, z2, state, mode] => (
                    [coordinate(x1), coordinate(y1), coordinate(z1)],
                    [coordinate(x2), coordinate(y2), coordinate(z2)],
                    state,
                    mode,
                ),
                ["setblock", x, y, z, state, mode] => {
                    let pos = [coordinate(x), coordinate(y), coordinate(z)];
                    (pos, pos, state, mode)
                }
                _ => panic!("{command}: the flat level only knows fill and setblock"),
            };
            assert_eq!(
                *mode, "strict",
                "{command}: only strict commands place what they say"
            );
            let state = parse_block_state(state);
            for x in from[0].min(to[0])..=from[0].max(to[0]) {
                for y in from[1].min(to[1])..=from[1].max(to[1]) {
                    for z in from[2].min(to[2])..=from[2].max(to[2]) {
                        level.blocks.insert(BlockPos::new(x, y, z), state);
                    }
                }
            }
        }
        level
    }
}

/// A block state as commands write it, `minecraft:oak_stairs[facing=south,half=bottom]`: the
/// block's default state with the properties named set (`BlockStateParser`, without tags, block
/// entity data or errors a test needs).
pub fn parse_block_state(text: &str) -> BlockState {
    let (name, properties) = match text.split_once('[') {
        Some((name, properties)) => (
            name,
            properties
                .strip_suffix(']')
                .unwrap_or_else(|| panic!("{text}: no ] after the properties")),
        ),
        None => (text, ""),
    };
    let block = blocks::registry()
        .get_value(&Identifier::parse(name))
        .unwrap_or_else(|| panic!("{text}: no block {name}"));
    let mut state = block.default_block_state();
    for property in properties
        .split(',')
        .filter(|property| !property.is_empty())
    {
        let (key, value) = property
            .split_once('=')
            .unwrap_or_else(|| panic!("{text}: {property} has no value"));
        let property = block
            .get_state_definition()
            .get_property(key)
            .unwrap_or_else(|| panic!("{text}: {name} has no property {key}"));
        let value = property
            .get_value(value)
            .unwrap_or_else(|| panic!("{text}: {key} can't be {value}"));
        state = state.set_comparable(property, value);
    }
    state
}

impl Default for FlatLevel {
    fn default() -> FlatLevel {
        FlatLevel::new()
    }
}

impl BlockGetter for FlatLevel {
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        if let Some(state) = self.blocks.get(pos) {
            return *state;
        }
        match pos.get_y() {
            -64 => Blocks::BEDROCK.default_block_state(),
            -63 | -62 => Blocks::DIRT.default_block_state(),
            -61 => Blocks::GRASS_BLOCK.default_block_state(),
            _ => Blocks::AIR.default_block_state(),
        }
    }

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        self.get_block_state(pos).get_fluid_state()
    }
}

impl CollisionGetter for FlatLevel {
    fn get_world_border(&self) -> &WorldBorder {
        &self.world_border
    }

    fn get_chunk_for_collisions(&self, _chunk_x: i32, _chunk_z: i32) -> Option<&dyn BlockGetter> {
        Some(self)
    }

    fn get_entity_collisions(
        &self,
        _source: Option<&dyn EntityClass>,
        _test_area: &AABB,
    ) -> Vec<VoxelShape> {
        Vec::new()
    }
}

impl Level for FlatLevel {
    fn is_client_side(&self) -> bool {
        true
    }

    fn get_min_y(&self) -> i32 {
        -64
    }

    fn has_chunk_at(&self, _pos: &BlockPos) -> bool {
        true
    }

    fn chunk_has_fluid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        section_y0: i32,
        section_y1: i32,
    ) -> Option<bool> {
        Some(self.blocks.iter().any(|(pos, state)| {
            SectionPos::block_to_section_coord(pos.get_x()) == chunk_x
                && SectionPos::block_to_section_coord(pos.get_z()) == chunk_z
                && (section_y0..=section_y1)
                    .contains(&SectionPos::block_to_section_coord(pos.get_y()))
                && !state.get_fluid_state().is_empty()
        }))
    }

    /// The overworld's lava, which isn't fast.
    fn has_fast_lava(&self) -> bool {
        false
    }
}

/// `GameProtocols.Context` for the replay's protocol, which no ported packet asks.
struct ReplayContext;

impl game_protocols::Context for ReplayContext {
    fn has_infinite_materials(&self) -> bool {
        false
    }

    fn can_use_command_blocks(&self) -> bool {
        false
    }
}

/// What a replay gives: a trace line after each of the scenario's ticks, and the serverbound
/// packets from the marker on, each `name payload` like `CapturedPacket::line`.
pub struct Replay {
    pub trace: Vec<String>,
    pub packets: Vec<String>,
}

/// Replays the scenario on the level, as the player with the entity id, writing packets with the
/// game protocol bound to the registries. `answers` are the server's clientbound packets from the
/// marker on, which a step's `wait_for` takes from; a scenario that waits panics without them.
pub fn replay(
    scenario: &Scenario,
    level: &dyn Level,
    others: &[Entity],
    registries: Arc<RegistryAccess>,
    player_id: i32,
    answers: Option<&[CapturedPacket]>,
) -> Replay {
    let registry_buffer = {
        let registries = Arc::clone(&registries);
        move |payload: &[i8]| {
            RegistryFriendlyByteBuf::new(wrapped_buffer(payload.to_vec()), Arc::clone(&registries))
        }
    };
    let protocol = GameProtocols::SERVERBOUND_TEMPLATE.bind(
        RegistryFriendlyByteBuf::decorator(registries),
        Box::new(ReplayContext),
    );
    // The protocol writes the packet's id, then its own codec's bytes.
    let encode = |packet: Box<dyn Packet<dyn ServerGamePacketListener>>| -> String {
        let mut output = unpooled::buffer();
        protocol.codec().encode(&mut output, &packet);
        VarInt::read(&mut output);
        let mut payload = vec![0; output.readable_bytes() as usize];
        output.read_bytes_array(&mut payload);
        format!("{} {}", packet.r#type().id(), hex(&payload))
    };

    let minecraft = Minecraft::new(
        User::new(
            "rustcraft".to_string(),
            UUID::new(0, 0),
            String::new(),
            None,
            None,
        ),
        Options::new(),
    );
    let mut player = LocalPlayer::new(None);
    player.entity_mut().set_id(player_id);
    minecraft.set_camera_entity(Some(player_id));
    player.set_auto_jump_enabled(minecraft.options.borrow().auto_jump());
    player.set_pos(Vec3::new(
        scenario.player.x,
        scenario.player.y,
        scenario.player.z,
    ));
    player.set_y_rot(scenario.player.y_rot);
    player.set_x_rot(scenario.player.x_rot);

    let tick = |player: &mut LocalPlayer| {
        player.common_tick();
        player.tick(level, &minecraft.options.borrow());
        player.send_changes(&minecraft);
    };
    // OracleGameTest.settle: 20 ticks, then until the player is on the ground.
    for _ in 0..OracleGameTest::SETTLE_TICKS {
        tick(&mut player);
    }
    let mut settling = 0;
    while !player.on_ground() {
        settling += 1;
        assert!(
            settling < OracleGameTest::LOAD_TIMEOUT_TICKS,
            "the player never landed"
        );
        tick(&mut player);
    }
    player.sent.clear();

    let mut packets = vec![encode(Box::new(ServerboundCustomPayloadPacket::new(
        Arc::new(DiscardedPayload::new(OracleGameTest::scenario_start())),
    )))];
    let mut trace = Vec::new();
    // The next of the server's answers not yet taken.
    let mut answered = 0;
    for step in &scenario.steps {
        if step.screenshot.is_some() {
            continue;
        }
        OracleGameTest::hold_only(&mut minecraft.options.borrow_mut(), &step.keys);
        for tick_of_step in 0..step.ticks {
            if let Some([y_rot, x_rot]) = step.look {
                player.set_y_rot(y_rot);
                player.set_x_rot(x_rot);
            }
            tick(&mut player);
            packets.extend(player.sent.drain(..).map(&encode));
            packets.push(encode(Box::new(ServerboundClientTickEndPacket::INSTANCE)));
            trace.push(trace_writer::line(trace.len() as i32, &player, others));
            if tick_of_step == step.ticks - 1 && !step.wait_for.is_empty() {
                let answers = answers.unwrap_or_else(|| {
                    panic!(
                        "{}: waiting for {:?} needs the server's answers, from a capture",
                        scenario.name, step.wait_for
                    )
                });
                let mut waiting: Vec<&String> = step.wait_for.iter().collect();
                while !waiting.is_empty() {
                    let answer = answers.get(answered).unwrap_or_else(|| {
                        panic!("{}: the server never sent {waiting:?}", scenario.name)
                    });
                    answered += 1;
                    waiting.retain(|name| **name != answer.name);
                    apply_answer(&mut player, answer, &registry_buffer);
                }
            }
        }
    }
    Replay { trace, packets }
}

/// What the client's listener does with one of the server's answers to the player
/// (`handleSetEntityMotion`, `handleSetHealth`, `handleDamageEvent`), for the packets that change
/// the player; the others change nothing it has.
fn apply_answer(
    player: &mut LocalPlayer,
    answer: &CapturedPacket,
    registry_buffer: &impl Fn(&[i8]) -> RegistryFriendlyByteBuf,
) {
    match answer.name.as_str() {
        "minecraft:set_entity_motion" => {
            let packet = ClientboundSetEntityMotionPacket::STREAM_CODEC
                .decode(&mut unpooled::wrapped_buffer(answer.payload.to_vec()));
            if packet.id() == player.get_id() {
                player.entity_mut().lerp_motion(packet.movement());
            }
        }
        "minecraft:set_health" => {
            let packet =
                ClientboundSetHealthPacket::STREAM_CODEC.decode(&mut friendly(&answer.payload));
            player.hurt_to(packet.get_health());
            player.food_level = packet.get_food();
        }
        "minecraft:damage_event" => {
            let packet = ClientboundDamageEventPacket::STREAM_CODEC
                .decode(&mut registry_buffer(&answer.payload));
            if packet.entity_id() == player.get_id() {
                player.handle_damage_event();
            }
        }
        _ => {}
    }
}

/// The marker's payload: its identifier, as `DiscardedPayload` writes it.
fn marker_payload() -> Vec<i8> {
    let identifier = OracleGameTest::scenario_start().to_string();
    let mut payload = vec![identifier.len() as i8];
    payload.extend(identifier.bytes().map(|byte| byte as i8));
    payload
}

fn friendly(payload: &[i8]) -> FriendlyByteBuf {
    FriendlyByteBuf::new(wrapped_buffer(payload.to_vec()))
}

pub fn hex(bytes: &[i8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02x}", *byte as u8))
        .collect()
}

fn unhex(text: &str) -> Vec<i8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex") as i8)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use md5::{Digest, Md5};

    use super::{Capture, CapturedLevel, FlatLevel, Replay, capture_path, replay, vanilla_dir};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::world::entity::entity::Entity;
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::rustcraft::oracle::client::scenario::Scenario;

    fn digest(lines: &[String]) -> String {
        let text: String = lines.iter().map(|line| format!("{line}\n")).collect();
        format!("{:x}", Md5::digest(text.as_bytes()))
    }

    /// Compares a replay with vanilla's run of the scenario: line by line where vanilla's files
    /// are, and by the committed digests everywhere.
    fn check(scenario: &str, replay: &Replay, trace_md5: &str, packets_md5: &str) {
        let vanilla = vanilla_dir(scenario);
        if let Ok(expected) = std::fs::read_to_string(vanilla.join("trace.jsonl")) {
            let expected: Vec<String> = expected.lines().map(str::to_string).collect();
            assert_eq!(
                digest(&expected),
                trace_md5,
                "{scenario}: vanilla's trace changed: update the digest"
            );
            for (index, (port, vanilla)) in replay.trace.iter().zip(&expected).enumerate() {
                assert_eq!(port, vanilla, "{scenario}: tick {index} differs");
            }
            assert_eq!(replay.trace.len(), expected.len(), "{scenario}: the ticks");
        }
        if let Some(capture) = Capture::load(&vanilla.join("capture.jsonl")) {
            let expected = capture.serverbound_from_marker();
            assert_eq!(
                digest(&expected),
                packets_md5,
                "{scenario}: vanilla's packets changed: update the digest"
            );
            for (index, (port, vanilla)) in replay.packets.iter().zip(&expected).enumerate() {
                assert_eq!(
                    port, vanilla,
                    "{scenario}: packet {index} from the marker differs"
                );
            }
            assert_eq!(
                replay.packets.len(),
                expected.len(),
                "{scenario}: the packets from the marker"
            );
        }
        assert_eq!(digest(&replay.trace), trace_md5, "{scenario}: the trace");
        assert_eq!(
            digest(&replay.packets),
            packets_md5,
            "{scenario}: the packets"
        );
    }

    /// A scenario against vanilla's run of it, by the digests of its `trace.jsonl` and of its
    /// serverbound packets from the scenario-start marker on, each `name payload` on a line
    /// (`Capture::serverbound_from_marker`). Vanilla's files aren't committed (they're its output)
    /// and CI records none (it runs no oracle client); where they are, they're compared line by
    /// line, and the digests checked to be theirs.
    ///
    /// The scenario is replayed on the flat preset with its setup's blocks, which needs no capture
    /// and is what CI checks; where a capture is (vanilla's, or the port's own that
    /// `tools/port_client.py capture` records, which CI makes for the walk and `falls`), on the
    /// level the server sent, whose blocks must be the flat level's, with the server's answers where
    /// a step waits for them; and where the port's own run on a server wrote its trace, that run's
    /// trace and packets are checked too. A scenario that waits for the server is only replayed on a
    /// capture, which has the answers.
    fn check_scenario(name: &str, trace_md5: &str, packets_md5: &str) {
        let scenario = Scenario::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("oracle/scenarios/{name}.json")),
        )
        .unwrap();
        let waits = scenario.steps.iter().any(|step| !step.wait_for.is_empty());
        // Vanilla's player is the server's first entity.
        let flat = FlatLevel::with_setup(&scenario.setup);
        if !waits {
            let on_flat = replay(
                &scenario,
                &flat,
                flat.entities(),
                Arc::new(RegistryAccess::new(Vec::new())),
                1,
                None,
            );
            check(name, &on_flat, trace_md5, packets_md5);
        }

        if let Some(path) = capture_path(name) {
            let capture = Capture::load(&path).unwrap();
            let captured = CapturedLevel::from_capture(&capture);
            let chunks = captured.level.get_chunk_source();
            assert!(
                chunks.get_loaded_chunks_count() >= 81,
                "{name}: the render distance's chunks came"
            );
            for chunk_x in -8..=8 {
                for chunk_z in -8..=8 {
                    if chunks.get_chunk(chunk_x, chunk_z).is_none() {
                        continue;
                    }
                    for x in chunk_x * 16..chunk_x * 16 + 16 {
                        for z in chunk_z * 16..chunk_z * 16 + 16 {
                            for y in -64..-32 {
                                let pos = BlockPos::new(x, y, z);
                                assert_eq!(
                                    captured.level.get_block_state(&pos),
                                    flat.get_block_state(&pos),
                                    "{name}: the server's block at {pos:?}"
                                );
                            }
                        }
                    }
                }
            }
            let others: Vec<Entity> = captured.level.entities_for_rendering().cloned().collect();
            let on_captured = replay(
                &scenario,
                &captured.level,
                &others,
                Arc::clone(&captured.registries),
                captured.player_id,
                Some(&capture.play_from_marker()),
            );
            check(name, &on_captured, trace_md5, packets_md5);
        } else {
            assert!(!waits, "{name} waits for the server, so it needs a capture");
        }

        let out = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/oracle")
            .join(name);
        if let Ok(trace) = std::fs::read_to_string(out.join("trace.jsonl")) {
            let trace: Vec<String> = trace.lines().map(str::to_string).collect();
            assert_eq!(
                digest(&trace),
                trace_md5,
                "{name}: the port's live run's trace"
            );
            let capture =
                Capture::load(&out.join("capture.jsonl")).expect("the live run's capture");
            assert_eq!(
                digest(&capture.serverbound_from_marker()),
                packets_md5,
                "{name}: the port's live run's packets from the marker on"
            );
        }
    }

    /// For writing a scenario: `SCENARIO=<name> cargo test --lib replays_a_draft -- --ignored`
    /// replays `oracle/scenarios/<name>.json` on the flat level and writes the trace and packets
    /// to `target/oracle/<name>-draft/` (TESTING.md). It checks nothing, and without a server its
    /// steps wait for nothing.
    #[test]
    #[ignore]
    fn replays_a_draft() {
        let name = std::env::var("SCENARIO").expect("SCENARIO names the draft");
        let mut scenario = Scenario::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("oracle/scenarios/{name}.json")),
        )
        .unwrap();
        for step in &mut scenario.steps {
            step.wait_for.clear();
        }
        let flat = FlatLevel::with_setup(&scenario.setup);
        let draft = replay(
            &scenario,
            &flat,
            flat.entities(),
            Arc::new(RegistryAccess::new(Vec::new())),
            1,
            None,
        );
        let out = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/oracle")
            .join(format!("{name}-draft"));
        std::fs::create_dir_all(&out).unwrap();
        let lines =
            |lines: &[String]| -> String { lines.iter().map(|line| format!("{line}\n")).collect() };
        std::fs::write(out.join("trace.jsonl"), lines(&draft.trace)).unwrap();
        std::fs::write(out.join("packets.txt"), lines(&draft.packets)).unwrap();
    }

    #[test]
    fn walk() {
        check_scenario(
            "walk",
            "b4da2320a6595f98e58b7590d2107891",
            "042b5bbd34696b409bd631458f0d7428",
        );
    }

    #[test]
    fn surface_concrete() {
        check_scenario(
            "surface-concrete",
            "3197055afb152f1fd07e2ba81bad0a02",
            "10f996be122484d9d815ed00146a6bc7",
        );
    }

    #[test]
    fn surface_packed_ice() {
        check_scenario(
            "surface-packed-ice",
            "8fa79d31cc3ddae18d240ca8a1e13f83",
            "9a94533634e8bd5e191e7d733633784b",
        );
    }

    #[test]
    fn surface_blue_ice() {
        check_scenario(
            "surface-blue-ice",
            "a0538c566e42529a17552379a4ffa81e",
            "ace24a1a6b6abd3814eb2885daad5979",
        );
    }

    #[test]
    fn surface_soul_sand() {
        check_scenario(
            "surface-soul-sand",
            "db1cc75db00e6aa6a2ad619889b15cfa",
            "4ddb287e5d85ce677c8bb7cacc9ec87e",
        );
    }

    #[test]
    fn surface_transitions() {
        check_scenario(
            "surface-transitions",
            "de225977ef43a54777c549665afbfbc3",
            "22fbe3963efb8c588ed1cf5cabb0c15a",
        );
    }

    #[test]
    fn block_steps() {
        check_scenario(
            "block-steps",
            "22d4afba3225e3ee9a4d110de2274417",
            "20eaa34078c682cb249ee59a3807353b",
        );
    }

    #[test]
    fn stairs() {
        check_scenario(
            "stairs",
            "b35ea9c4aced5137dcba9df1bef56887",
            "678d10595a6ef6514febbe890b73cbdd",
        );
    }

    #[test]
    fn walls_and_corners() {
        check_scenario(
            "walls-and-corners",
            "a61c9659d9444f19b4469277122fc190",
            "270af4d56236a36845a09caadec9e1a9",
        );
    }

    #[test]
    fn fall() {
        check_scenario(
            "fall",
            "add7a4829d815351ba213f54ad08ae78",
            "6d2c4cc92285b5f055ea9ac2c034bb66",
        );
    }

    #[test]
    fn low_ceiling() {
        check_scenario(
            "low-ceiling",
            "d705b1d2bc00b7a1e5d04ef601c3e829",
            "d724a7fa758d41e6646516d6101be976",
        );
    }

    #[test]
    fn sneak_edge() {
        check_scenario(
            "sneak-edge",
            "5806edc5f16a9c7a0eb65dc636abed6c",
            "41b0d3972df2cc42427a29b94ed16782",
        );
    }

    #[test]
    fn slabs_and_fences() {
        check_scenario(
            "slabs-and-fences",
            "af2df57626f0524ee6916e65a7cf0c22",
            "e79d419709cf590483b41be6d7b3173d",
        );
    }

    #[test]
    fn water() {
        check_scenario(
            "water",
            "cdf67b202b739602e265d78dfdc1f121",
            "1f7ca123dcd646509c2434714da7cc6e",
        );
    }

    #[test]
    fn water_current() {
        check_scenario(
            "water-current",
            "c73ce8788dd6ade9f3830c4a840360ea",
            "d5812b7c69a1dd60c71d92d9b745bc08",
        );
    }

    #[test]
    fn climbing() {
        check_scenario(
            "climbing",
            "8f3e1f767010fc7a59166dcbd7aabadb",
            "f349cd376f7b0beb17b9d9e30062e50e",
        );
    }

    #[test]
    fn scaffolding() {
        check_scenario(
            "scaffolding",
            "86511a2d62d1dbbe19ea60502d28a297",
            "3357e9129e84cd4aad9a5ea68a51f6fa",
        );
    }

    #[test]
    fn walls_and_panes() {
        check_scenario(
            "walls-and-panes",
            "af77fc79cb0d357eb688e4f566e2f149",
            "eb104cf96d7efc303b0da1446b9b0549",
        );
    }

    #[test]
    fn falls() {
        check_scenario(
            "falls",
            "2f928d8ecf2e4023ea31ca262fa6b8e6",
            "71840c6ffe3f52b3f30db9b7ae551a37",
        );
    }

    #[test]
    fn entities() {
        check_scenario(
            "entities",
            "e912051e4123b118cc6da239e409f820",
            "31f64f12f277ec87bfca9717685ffdf1",
        );
    }
}
