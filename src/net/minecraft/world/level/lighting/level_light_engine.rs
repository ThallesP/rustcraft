//! net/minecraft/world/level/lighting/LevelLightEngine.java
//!
//! A level's light: a block light engine and, in a dimension with a sky, a sky light engine, each
//! storing its light by section. The client's keeps the light the server sends
//! (`queueSectionData`), turns each chunk's light on (`setLightEnabled`) and its sections' storage
//! on as they get blocks (`updateSectionStatus`), works out light itself where blocks change
//! (`checkBlock`, `runLightUpdates`), and the renderer and the game read it
//! (`getLayerListener(…).getLightValue`, `getRawBrightness`). Its `levelHeightAccessor` is the
//! level's height, which `new` takes instead of Java's chunk source
//! (`LightChunkGetter.getLevel()`). Not ported yet: `propagateLightSources`, which only the server
//! calls, when it lights a chunk it made.

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::{
    self, Height, LevelHeightAccessor,
};
use crate::net::minecraft::world::level::light_layer::LightLayer;
use crate::net::minecraft::world::level::lighting::block_light_engine::BlockLightEngine;
use crate::net::minecraft::world::level::lighting::layer_light_event_listener::{
    DummyLightLayerEventListener, LayerLightEventListener,
};
use crate::net::minecraft::world::level::lighting::layer_light_section_storage::SectionType;
use crate::net::minecraft::world::level::lighting::light_engine::{
    self, LightEngine, LightEngineClass,
};
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
use crate::net::minecraft::world::level::lighting::sky_light_engine::SkyLightEngine;

/// A level's light engines.
pub struct LevelLightEngine {
    level_height_accessor: Height,
    block_engine: Option<BlockLightEngine>,
    sky_engine: Option<SkyLightEngine>,
}

impl LevelLightEngine {
    pub const LIGHT_SECTION_PADDING: i32 = 1;

    /// For a level of that height, with the engines it asks for.
    pub fn new(
        level_height: Height,
        has_block_light: bool,
        has_sky_light: bool,
    ) -> LevelLightEngine {
        LevelLightEngine {
            level_height_accessor: level_height,
            block_engine: has_block_light.then(BlockLightEngine::new),
            sky_engine: has_sky_light.then(|| SkyLightEngine::new(level_height)),
        }
    }

    /// `EMPTY`: no engines, no height.
    pub fn empty() -> LevelLightEngine {
        LevelLightEngine {
            level_height_accessor: level_height_accessor::create(0, 0),
            block_engine: None,
            sky_engine: None,
        }
    }

    pub fn get_layer_listener(&self, layer: LightLayer) -> &dyn LayerLightEventListener {
        let listener: Option<&dyn LayerLightEventListener> = match layer {
            LightLayer::BLOCK => self
                .block_engine
                .as_ref()
                .map(|engine| engine as &dyn LayerLightEventListener),
            LightLayer::SKY => self
                .sky_engine
                .as_ref()
                .map(|engine| engine as &dyn LayerLightEventListener),
        };
        listener.unwrap_or(&DummyLightLayerEventListener)
    }

    pub fn get_debug_data(&self, layer: LightLayer, pos: &SectionPos) -> String {
        let data = match layer {
            LightLayer::BLOCK => self
                .block_engine
                .as_ref()
                .map(|engine| engine.get_debug_data(pos.as_long())),
            LightLayer::SKY => self
                .sky_engine
                .as_ref()
                .map(|engine| engine.get_debug_data(pos.as_long())),
        };
        data.unwrap_or_else(|| "n/a".to_owned())
    }

    pub fn get_debug_section_type(&self, layer: LightLayer, pos: &SectionPos) -> SectionType {
        let section_type = match layer {
            LightLayer::BLOCK => self
                .block_engine
                .as_ref()
                .map(|engine| engine.get_debug_section_type(pos.as_long())),
            LightLayer::SKY => self
                .sky_engine
                .as_ref()
                .map(|engine| engine.get_debug_section_type(pos.as_long())),
        };
        section_type.unwrap_or(SectionType::EMPTY)
    }

    /// The layer the server sent for a section, or none to drop the one queued. The engine keeps
    /// the layer itself, like Java's.
    pub fn queue_section_data(
        &mut self,
        layer: LightLayer,
        pos: &SectionPos,
        data: Option<DataLayer>,
    ) {
        let data = data.map(|data| Rc::new(RefCell::new(data)));
        match layer {
            LightLayer::BLOCK => {
                if let Some(engine) = &mut self.block_engine {
                    engine.queue_section_data(pos.as_long(), data);
                }
            }
            LightLayer::SKY => {
                if let Some(engine) = &mut self.sky_engine {
                    engine.queue_section_data(pos.as_long(), data);
                }
            }
        }
    }

    pub fn retain_data(&mut self, pos: ChunkPos, retain: bool) {
        if let Some(engine) = &mut self.block_engine {
            engine.retain_data(pos, retain);
        }

        if let Some(engine) = &mut self.sky_engine {
            engine.retain_data(pos, retain);
        }
    }

    /// The brighter of the block's light and the sky's, the sky's dimmed by `sky_dampen`.
    pub fn get_raw_brightness(&self, pos: &BlockPos, sky_dampen: i32) -> i32 {
        let sky_light = match &self.sky_engine {
            None => 0,
            Some(engine) => engine.get_light_value(pos) - sky_dampen,
        };
        let block_light = match &self.block_engine {
            None => 0,
            Some(engine) => engine.get_light_value(pos),
        };
        block_light.max(sky_light)
    }

    pub fn light_on_in_column(&self, section_zero_node: i64) -> bool {
        match &self.block_engine {
            None => true,
            Some(block_engine) => {
                block_engine.storage.light_on_in_column(section_zero_node)
                    && self.sky_engine.as_ref().is_none_or(|sky_engine| {
                        sky_engine.storage.light_on_in_column(section_zero_node)
                    })
            }
        }
    }

    /// The level's sections, and one more above and below.
    pub fn get_light_section_count(&self) -> i32 {
        self.level_height_accessor.get_sections_count() + 2
    }

    pub fn get_min_light_section(&self) -> i32 {
        self.level_height_accessor.get_min_section_y() - 1
    }

    pub fn get_max_light_section(&self) -> i32 {
        self.get_min_light_section() + self.get_light_section_count()
    }
}

impl LightEventListener for LevelLightEngine {
    fn check_block(&mut self, pos: &BlockPos) {
        if let Some(engine) = &mut self.block_engine {
            engine.check_block(pos);
        }

        if let Some(engine) = &mut self.sky_engine {
            engine.check_block(pos);
        }
    }

    fn has_light_work(&self) -> bool {
        self.sky_engine
            .as_ref()
            .is_some_and(|engine| engine.has_light_work())
            || self
                .block_engine
                .as_ref()
                .is_some_and(|engine| engine.has_light_work())
    }

    fn run_light_updates(&mut self, chunk_source: &dyn LightChunkGetter) -> i32 {
        let mut count = 0;
        if let Some(engine) = &mut self.block_engine {
            count += light_engine::run_light_updates(engine, chunk_source);
        }

        if let Some(engine) = &mut self.sky_engine {
            count += light_engine::run_light_updates(engine, chunk_source);
        }

        count
    }

    fn update_section_status(&mut self, pos: SectionPos, section_empty: bool) {
        if let Some(engine) = &mut self.block_engine {
            engine.update_section_status(pos, section_empty);
        }

        if let Some(engine) = &mut self.sky_engine {
            engine.update_section_status(pos, section_empty);
        }
    }

    fn set_light_enabled(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        pos: ChunkPos,
        enable: bool,
    ) {
        if let Some(engine) = &mut self.block_engine {
            engine.set_light_enabled_base(pos, enable);
        }

        if let Some(engine) = &mut self.sky_engine {
            engine.set_light_enabled(chunk_source, pos, enable);
        }
    }
}

/// An engine lent out to read its layer.
impl<E: LightEngineClass> LayerLightEventListener for E {
    fn get_data_layer_data(&self, pos: &SectionPos) -> Option<DataLayer> {
        LightEngine::get_data_layer_data(self, pos)
    }

    fn get_light_value(&self, pos: &BlockPos) -> i32 {
        LightEngine::get_light_value(self, pos)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::{Value, json};

    use crate::golden::{Golden, unhex};
    use crate::net::minecraft::client::multiplayer::client_chunk_cache::tests::overworld;
    use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
    use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::section_pos::SectionPos;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet::ClientboundLightUpdatePacket;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::tests::registries;
    use crate::net::minecraft::world::level::chunk::light_chunk::LightChunk;
    use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
    use crate::net::minecraft::world::level::light_layer::LightLayer;
    use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;

    /// Values in order as runs, `[[value, count], …]`.
    fn runs_of(values: impl Iterator<Item = i32>) -> Value {
        let mut runs: Vec<(i32, i32)> = Vec::new();
        for value in values {
            match runs.last_mut() {
                Some((last, count)) if *last == value => *count += 1,
                _ => runs.push((value, 1)),
            }
        }
        Value::from(
            runs.into_iter()
                .map(|(value, count)| json!([value, count]))
                .collect::<Vec<_>>(),
        )
    }

    /// A section's values in index order (y, then z, then x) as runs.
    fn section_runs(lookup: impl Fn(i32, i32, i32) -> i32) -> Value {
        runs_of(
            (0..16)
                .flat_map(|y| (0..16).flat_map(move |z| (0..16).map(move |x| (x, y, z))))
                .map(|(x, y, z)| lookup(x, y, z)),
        )
    }

    fn int(event: &[Value], index: usize) -> i32 {
        event[index].as_i64().expect("an int") as i32
    }

    fn layer_name(section_type: super::SectionType) -> String {
        format!("{section_type:?}")
    }

    /// `["box", x, y, z, r]`: the sky's and the blocks' light of the cube, as runs.
    fn read_box(level: &ClientLevel, x: i32, y: i32, z: i32, r: i32) -> Value {
        let layers: Vec<Value> = [LightLayer::SKY, LightLayer::BLOCK]
            .into_iter()
            .map(|layer| {
                let mut values = Vec::new();
                for dy in -r..=r {
                    for dz in -r..=r {
                        for dx in -r..=r {
                            values.push(
                                level.get_brightness(layer, &BlockPos::new(x + dx, y + dy, z + dz)),
                            );
                        }
                    }
                }
                runs_of(values.into_iter())
            })
            .collect();
        Value::from(layers)
    }

    /// `["read", minX, maxX, minZ, maxZ]`: each column's light, as the suite reads it back.
    fn read(level: &ClientLevel, min_x: i32, max_x: i32, min_z: i32, max_z: i32) -> Value {
        let engine = level.get_light_engine();
        let mut columns = Vec::new();
        for chunk_x in min_x..=max_x {
            for chunk_z in min_z..=max_z {
                let sources = match level.get_chunk_source().get_chunk(chunk_x, chunk_z) {
                    None => Value::Null,
                    Some(chunk) => {
                        let sources = chunk.get_sky_light_sources();
                        let lowest = (0..16)
                            .flat_map(|z| (0..16).map(move |x| sources.get_lowest_source_y(x, z)));
                        json!([runs_of(lowest), sources.get_highest_lowest_source_y()])
                    }
                };
                let mut sections = Vec::new();
                for section_y in engine.get_min_light_section()..engine.get_max_light_section() {
                    let section = SectionPos::of(chunk_x, section_y, chunk_z);
                    let light = |layer| {
                        section_runs(|x, y, z| {
                            level.get_brightness(
                                layer,
                                &BlockPos::new(
                                    section.min_block_x() + x,
                                    section.min_block_y() + y,
                                    section.min_block_z() + z,
                                ),
                            )
                        })
                    };
                    sections.push(json!([
                        layer_name(engine.get_debug_section_type(LightLayer::SKY, &section)),
                        layer_name(engine.get_debug_section_type(LightLayer::BLOCK, &section)),
                        light(LightLayer::SKY),
                        light(LightLayer::BLOCK),
                    ]));
                }
                columns.push(json!([
                    chunk_x,
                    chunk_z,
                    engine.light_on_in_column(SectionPos::get_zero_node_xz(chunk_x, chunk_z)),
                    sources,
                    sections,
                ]));
            }
        }
        Value::from(columns)
    }

    /// The program's events on a level, as the client's handlers and frames run them.
    fn run(
        level: &mut ClientLevel,
        packets: &HashMap<(i32, i32), ClientboundLevelChunkWithLightPacket>,
        program: &[Value],
        mut check: impl FnMut(usize, &Value),
    ) {
        for (index, event) in program.iter().enumerate() {
            let event = event.as_array().expect("an event");
            let output = match event[0].as_str().expect("its name") {
                "chunk" => {
                    let packet = &packets[&(int(event, 1), int(event, 2))];
                    ClientPacketListener::handle_level_chunk_with_light_in(level, packet);
                    Value::Null
                }
                "light" => {
                    let (x, z) = (int(event, 1), int(event, 2));
                    let packet = ClientboundLightUpdatePacket::new(
                        x,
                        z,
                        packets[&(x, z)].light_data().clone(),
                    );
                    ClientPacketListener::handle_light_update_packet_in(level, &packet);
                    Value::Null
                }
                "forget" => {
                    let packet = ClientboundForgetLevelChunkPacket::new(ChunkPos::new(
                        int(event, 1),
                        int(event, 2),
                    ));
                    ClientPacketListener::handle_forget_level_chunk_in(level, &packet);
                    Value::Null
                }
                "frame" => {
                    // `ClientLevel.update`, with what the suite records of it.
                    level.poll_light_updates();
                    let (engine, chunks) = level.get_light_engine_mut();
                    let count = engine.run_light_updates(chunks);
                    json!([
                        count,
                        level.get_light_engine().has_light_work(),
                        level.light_updates_queued()
                    ])
                }
                "set" => {
                    let pos = BlockPos::new(int(event, 1), int(event, 2), int(event, 3));
                    let state = Block::state_by_id(int(event, 4));
                    level.set_server_verified_block_state(&pos, state, 19);
                    Value::Null
                }
                "fill" => {
                    let state = Block::state_by_id(int(event, 7));
                    for x in int(event, 1)..=int(event, 4) {
                        for y in int(event, 2)..=int(event, 5) {
                            for z in int(event, 3)..=int(event, 6) {
                                level.set_server_verified_block_state(
                                    &BlockPos::new(x, y, z),
                                    state,
                                    19,
                                );
                            }
                        }
                    }
                    Value::Null
                }
                "box" => read_box(
                    level,
                    int(event, 1),
                    int(event, 2),
                    int(event, 3),
                    int(event, 4),
                ),
                "read" => read(
                    level,
                    int(event, 1),
                    int(event, 2),
                    int(event, 3),
                    int(event, 4),
                ),
                other => panic!("no event {other}"),
            };
            check(index, &output);
        }
    }

    /// The first place two outputs differ, for the failure message.
    fn first_difference(path: String, expected: &Value, actual: &Value) -> Option<String> {
        match (expected, actual) {
            (Value::Array(expected), Value::Array(actual)) => {
                for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                    if let Some(difference) =
                        first_difference(format!("{path}[{index}]"), expected, actual)
                    {
                        return Some(difference);
                    }
                }
                (expected.len() != actual.len())
                    .then(|| format!("{path}: {} elements, not {}", expected.len(), actual.len()))
            }
            _ => (expected != actual).then(|| format!("{path}: expected {expected}, got {actual}")),
        }
    }

    /// The walk's chunks and their light, fed to the client's level by its handlers in vanilla's
    /// programs (chunks, light packets, forgetting chunks, frames, blocks set), and read back:
    /// every light value and section type of the columns, the sky light sources, and the light
    /// around each block set, as vanilla's light engine has them. A quarter of the programs, by
    /// their number: reading the light back in an unoptimized build takes a while, which the
    /// four tests share.
    fn check_programs(shard: usize) {
        let chunk_golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&chunk_golden.input("BIOMES"));
        let mut packets = HashMap::new();
        for (args, _) in chunk_golden.cases("captured") {
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            packets.insert((packet.x(), packet.z()), packet);
        }

        let golden = Golden::load("net.minecraft.world.level.lighting.LevelLightEngine");
        let programs = golden.cases("programs");
        assert!(programs.len() >= 50, "the suite's programs");
        let mut failures = Vec::new();
        for (number, (args, expected)) in programs.iter().enumerate() {
            if number % SHARDS != shard {
                continue;
            }
            let program = args[0].as_array().expect("events");
            let expected = expected.as_array().expect("outputs");
            let mut level = overworld(&registries, 8);
            let mut outputs = 0;
            run(&mut level, &packets, program, |index, output| {
                outputs += 1;
                if failures.len() < 8
                    && let Some(difference) =
                        first_difference(String::new(), &expected[index], output)
                {
                    failures.push(format!(
                        "program {number}, event {index} {}: {difference}",
                        program[index]
                    ));
                }
            });

            // Then the raw brightness of blocks around the spawn.
            for case in expected[outputs].as_array().expect("raw brightness") {
                let case: Vec<i32> = case
                    .as_array()
                    .expect("a case")
                    .iter()
                    .map(|value| value.as_i64().expect("an int") as i32)
                    .collect();
                let pos = BlockPos::new(case[0], case[1], case[2]);
                let actual = level.get_raw_brightness(&pos, case[3]);
                if actual != case[4] && failures.len() < 8 {
                    failures.push(format!(
                        "program {number}: getRawBrightness({pos:?}, {}) is {}, not {actual}",
                        case[3], case[4]
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    const SHARDS: usize = 4;

    #[test]
    fn the_clients_light_matches_vanillas_0() {
        check_programs(0);
    }

    #[test]
    fn the_clients_light_matches_vanillas_1() {
        check_programs(1);
    }

    #[test]
    fn the_clients_light_matches_vanillas_2() {
        check_programs(2);
    }

    #[test]
    fn the_clients_light_matches_vanillas_3() {
        check_programs(3);
    }
}
