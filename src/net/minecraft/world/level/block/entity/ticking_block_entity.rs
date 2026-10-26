//! net/minecraft/world/level/block/entity/TickingBlockEntity.java
//!
//! What a level ticks for a block entity. Java's interface has three implementations, all in
//! `LevelChunk`: its `NULL_TICKER`, `BoundTickingBlockEntity` and the wrapper the level keeps
//! (`RebindableTickingBlockEntityWrapper`), which holds one of the other two. Here the first two are
//! this enum's variants and the wrapper is `level_chunk`'s.

use std::rc::Rc;

use crate::net::minecraft::client::multiplayer::client_level::{ClientLevel, LevelTicking};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::chunk::level_chunk::BoundTickingBlockEntity;

pub enum TickingBlockEntity {
    /// `LevelChunk.NULL_TICKER`: a ticker whose block entity is gone, removed from the level's
    /// list the next time it ticks.
    Null,
    Bound(Rc<BoundTickingBlockEntity>),
}

impl TickingBlockEntity {
    pub fn tick(&self, level: &mut LevelTicking<'_>) {
        match self {
            TickingBlockEntity::Null => {}
            TickingBlockEntity::Bound(bound) => bound.tick(level),
        }
    }

    pub fn is_removed(&self, level: &ClientLevel) -> bool {
        match self {
            TickingBlockEntity::Null => true,
            TickingBlockEntity::Bound(bound) => bound.is_removed(level),
        }
    }

    pub fn get_pos(&self) -> BlockPos {
        match self {
            TickingBlockEntity::Null => BlockPos::ZERO,
            TickingBlockEntity::Bound(bound) => bound.get_pos(),
        }
    }

    /// `getType()`: the block entity type's registered name, `<null>` for none.
    pub fn get_type(&self) -> &'static str {
        match self {
            TickingBlockEntity::Null => "<null>",
            TickingBlockEntity::Bound(bound) => bound.get_type().key(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use serde_json::{Value, json};

    use crate::golden::{Decode, Golden, unhex};
    use crate::net::minecraft::client::color::block::block_colors::BlockColors;
    use crate::net::minecraft::client::multiplayer::client_level::{
        ClientLevel, LevelEffects, LevelPlayer, LevelTicking,
    };
    use crate::net::minecraft::client::particle::particle_engine::ParticleEngine;
    use crate::net::minecraft::client::sounds::sound_manager::{SoundInstance, SoundManager};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::server::level::particle_status::ParticleStatus;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::entity::mover_type::MoverType;
    use crate::net::minecraft::world::level::biome::biome::tests::biome_registries;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::entity::beacon_beam_owner::Section;
    use crate::net::minecraft::world::level::block::entity::block_entity::{
        BlockEntity, BlockEntityData,
    };
    use crate::net::minecraft::world::level::block::entity::chest_lid_controller::ChestLidController;
    use crate::net::minecraft::world::level::block::entity::enchanting_table_block_entity::EnchantingTableBlockEntity;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    fn bits_f32(value: f32) -> Value {
        json!(format!("{:08x}", value.to_bits()))
    }

    fn bits_f64(value: f64) -> Value {
        json!(format!("{:016x}", value.to_bits()))
    }

    /// What the particle engine and the player were handed since the last tick's line.
    #[derive(Default)]
    struct Recorder {
        events: RefCell<Vec<Value>>,
    }

    impl ParticleEngine for Recorder {
        fn create_particle(
            &self,
            options: &ParticleOptions,
            x: f64,
            y: f64,
            z: f64,
            xd: f64,
            yd: f64,
            zd: f64,
        ) {
            self.events.borrow_mut().push(json!([
                "particle",
                options.get_type().key().to_string(),
                bits_f64(x),
                bits_f64(y),
                bits_f64(z),
                bits_f64(xd),
                bits_f64(yd),
                bits_f64(zd)
            ]));
        }
    }

    impl SoundManager for Recorder {
        fn play(&self, sound: SoundInstance) {
            let event = match &sound {
                SoundInstance::Simple(instance) => instance.sound,
                SoundInstance::EntityBound(instance) => instance.sound,
            };
            self.events
                .borrow_mut()
                .push(json!(["sound", event.location().to_string()]));
        }

        fn play_delayed(&self, sound: SoundInstance, _delay: i32) {
            self.play(sound);
        }
    }

    /// The suite's player: where it stands, recording the moves it's handed.
    struct Player<'a> {
        position: Vec3,
        recorder: &'a Recorder,
    }

    impl LevelPlayer for Player<'_> {
        fn get_bounding_box(&self) -> AABB {
            let p = self.position;
            AABB::new(p.x - 0.3, p.y, p.z - 0.3, p.x + 0.3, p.y + 1.8, p.z + 0.3)
        }

        fn position(&self) -> Vec3 {
            self.position
        }

        fn is_spectating(&self) -> bool {
            false
        }

        fn is_alive(&self) -> bool {
            true
        }

        fn r#move(&mut self, _level: &ClientLevel, mover_type: MoverType, delta: Vec3) {
            self.recorder.events.borrow_mut().push(json!([
                "move",
                format!("{mover_type:?}"),
                bits_f64(delta.x),
                bits_f64(delta.y),
                bits_f64(delta.z)
            ]));
        }
    }

    fn lid(lid: &ChestLidController) -> Value {
        json!([
            lid.is_should_be_open(),
            bits_f32(lid.get_openness(0.0)),
            bits_f32(lid.get_openness(1.0))
        ])
    }

    fn sections(sections: &[Section]) -> Value {
        sections
            .iter()
            .map(|section| json!([section.get_color(), section.get_height()]))
            .collect()
    }

    fn boxes(shape: &VoxelShape) -> Value {
        shape
            .to_aabbs()
            .iter()
            .map(|b| {
                json!([
                    bits_f64(b.min_x),
                    bits_f64(b.min_y),
                    bits_f64(b.min_z),
                    bits_f64(b.max_x),
                    bits_f64(b.max_y),
                    bits_f64(b.max_z)
                ])
            })
            .collect()
    }

    /// The suite's `state`: `[type, x, y, z, removed, fields]`.
    fn state(level: &ClientLevel, entity: &BlockEntity) -> Value {
        let pos = entity.get_block_pos();
        let fields = match &entity.data {
            BlockEntityData::Chest(chest) => lid(chest.chest_lid_controller()),
            BlockEntityData::EnderChest(chest) => lid(chest.chest_lid_controller()),
            BlockEntityData::ShulkerBox(r#box) => {
                let state = entity.get_block_state();
                json!([
                    format!("{:?}", r#box.get_animation_status()),
                    bits_f32(r#box.get_progress(0.0)),
                    bits_f32(r#box.get_progress(1.0)),
                    r#box.open_count(),
                    boxes(&state.get_shape(level, &pos)),
                    boxes(&state.get_block_support_shape(level, &pos))
                ])
            }
            BlockEntityData::Bell(bell) => json!([
                bell.ticks,
                bell.shaking,
                bell.click_direction
                    .map(|direction| format!("{direction:?}")),
                bell.resonating(),
                bell.resonation_ticks(),
                bell.last_ring_timestamp()
            ]),
            BlockEntityData::EndGateway(gateway) => {
                json!([gateway.age(), gateway.teleport_cooldown()])
            }
            BlockEntityData::DecoratedPot(pot) => json!([
                pot.wobble_started_at_tick,
                pot.last_wobble_style.map(|style| format!("{style:?}"))
            ]),
            BlockEntityData::Spawner(spawner) => {
                let spawner = spawner.get_spawner();
                json!([
                    spawner.spawn_delay(),
                    bits_f64(spawner.get_spin()),
                    bits_f64(spawner.get_o_spin())
                ])
            }
            BlockEntityData::Skull(skull) => {
                json!([skull.is_animating(), skull.animation_tick_count()])
            }
            BlockEntityData::Conduit(conduit) => json!([
                conduit.tick_count,
                bits_f32(conduit.active_rotation()),
                conduit.is_active(),
                conduit.is_hunting(),
                conduit
                    .effect_blocks()
                    .iter()
                    .map(|pos| json!([pos.get_x(), pos.get_y(), pos.get_z()]))
                    .collect::<Vec<_>>()
            ]),
            BlockEntityData::Beacon(beacon) => json!([
                sections(beacon.get_beam_sections()),
                sections(beacon.checking_beam_sections()),
                beacon.levels(),
                beacon.last_check_y()
            ]),
            BlockEntityData::EnchantingTable(table) => json!([
                table.time,
                bits_f32(table.flip),
                bits_f32(table.o_flip),
                bits_f32(table.flip_t),
                bits_f32(table.flip_a),
                bits_f32(table.open),
                bits_f32(table.o_open),
                bits_f32(table.rot),
                bits_f32(table.o_rot),
                bits_f32(table.t_rot)
            ]),
            BlockEntityData::PotentSulfur(sulfur) => json!([sulfur.eruption_tick]),
            _ => Value::Null,
        };
        json!([
            entity.get_type().key(),
            pos.get_x(),
            pos.get_y(),
            pos.get_z(),
            entity.is_removed(),
            fields
        ])
    }

    /// A tick's line: `[events, tickers, blockEntities]`.
    fn line(level: &ClientLevel, events: Vec<Value>) -> Value {
        let tickers: Vec<Value> = level
            .block_entity_tickers()
            .tickers()
            .iter()
            .map(|ticker| {
                let pos = ticker.get_pos();
                json!([
                    ticker.get_type(),
                    pos.get_x(),
                    pos.get_y(),
                    pos.get_z(),
                    ticker.is_removed(level)
                ])
            })
            .collect();
        let mut block_entities = Vec::new();
        for x in -1..=1 {
            for z in -1..=1 {
                let chunk = level
                    .get_chunk_source()
                    .get_chunk(x, z)
                    .expect("the course's chunks");
                for (_, entity) in chunk.get_block_entities().iter() {
                    block_entities.push(state(level, &entity.borrow()));
                }
            }
        }
        json!([events, tickers, block_entities])
    }

    /// The level's block entities ticked over 40 ticks with the server's block events and block
    /// changes between them, like vanilla's `ClientLevel`: what their tickers and events hand the
    /// particle engine and the player, the level's tickers in order, and each block entity's state
    /// (TESTING.md, "Block entities").
    #[test]
    fn ticks_like_vanillas() {
        let chunk_golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = biome_registries(&chunk_golden.input("BIOMES"));
        let mut packets = Vec::new();
        for (args, _) in chunk_golden.cases("captured") {
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            if packet.x().abs() <= 1 && packet.z().abs() <= 1 {
                packets.push(packet);
            }
        }
        assert_eq!(packets.len(), 9);
        let block_colors = BlockColors::create_default();

        let golden = Golden::load("net.minecraft.world.level.block.entity.TickingBlockEntity");
        let programs = golden.cases("programs");
        assert_eq!(programs.len(), 48);
        for (number, (args, expected)) in programs.into_iter().enumerate() {
            let mut level = ClientLevel::new(
                &registries,
                false,
                false,
                ResourceKey::create(
                    Registries::DIMENSION,
                    Identifier::with_default_namespace("overworld"),
                ),
                Holder::Reference(ResourceKey::create(
                    Registries::DIMENSION_TYPE,
                    Identifier::with_default_namespace("overworld"),
                )),
                8,
                8,
                false,
                0,
                63,
            );
            for packet in &packets {
                level.get_chunk_source_mut().replace_with_packet_data(
                    packet.x(),
                    packet.z(),
                    packet.chunk_data(),
                );
            }
            *level.get_random() = LegacyRandomSource::new(i64::decode(&args[0]));
            EnchantingTableBlockEntity::seed_random(i64::decode(&args[1]));
            let particles = match args[2].as_str().expect("a status") {
                "ALL" => ParticleStatus::ALL,
                "DECREASED" => ParticleStatus::DECREASED,
                _ => ParticleStatus::MINIMAL,
            };
            let position = Vec::<f64>::decode(&args[3]);
            let position = Vec3::new(position[0], position[1], position[2]);
            let mut time = i64::decode(&args[4]);
            level.set_time_from_server(time);

            let recorder = Recorder::default();
            let mut player = Player {
                position,
                recorder: &recorder,
            };
            let effects = LevelEffects {
                particle_engine: &recorder,
                sound_manager: &recorder,
                camera_position: position.add_xyz(0.0, 1.62, 0.0),
                particles,
                player: Some(7),
                environment: None,
                block_colors: &block_colors,
            };
            let mut lines = Vec::new();
            for op in args[5].as_array().expect("ops") {
                let op = op.as_array().expect("an op");
                let int = |index: usize| op[index].as_i64().expect("an int") as i32;
                match op[0].as_str().expect("an op's name") {
                    "set" => {
                        level.set_block(
                            &BlockPos::new(int(1), int(2), int(3)),
                            Block::state_by_id(int(4)),
                            18,
                            512,
                        );
                    }
                    "event" => {
                        let mut ticking = LevelTicking {
                            level: &mut level,
                            effects: &effects,
                            player: Some(&mut player),
                        };
                        ClientLevel::block_event(
                            &mut ticking,
                            &BlockPos::new(int(1), int(2), int(3)),
                            int(4),
                            int(5),
                        );
                    }
                    _ => {
                        level.set_time_from_server(time);
                        level.tick_block_entities(&effects, Some(&mut player));
                        time += 1;
                        lines.push(line(&level, recorder.events.take()));
                    }
                }
            }
            lines.push(json!([
                "end",
                level.get_random().next_long(),
                EnchantingTableBlockEntity::next_random_long()
            ]));
            let expected = expected.as_array().expect("lines");
            assert_eq!(lines.len(), expected.len(), "program {number}");
            for (tick, (actual, expected)) in lines.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "program {number}, tick {tick}");
            }
        }
        golden.finish();
    }
}
