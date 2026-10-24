//! The benchmark's measurements of the port (TESTING.md, "Benchmark"). `tools/bench.py` runs the
//! port and vanilla on the same world with the same settings and reads the same points of both:
//! vanilla's through JFR's method tracing (`tools/JfrEvents.java` names the methods), the port's
//! from here, at the same places:
//! - a frame (`frame`): the game loop's turn, from its start (`runTick`'s) to the next;
//! - the end of the frame's GL calls (`submit`): where vanilla submits the frame, after the blit
//!   to the window, the port presents it, both before waiting for the driver to draw it;
//! - each tick (`tick`), each section compile (`compile`), each frame's light updates
//!   (`ClientLevel.update`, `light`) and each chunk the game thread takes from a packet
//!   (`handleLevelChunkWithLight`, `chunk`: its sections and heightmaps read, its light queued).
//!
//! It only measures: nothing here changes what the game does. `rustcraft --bench BENCH.json
//! --benchOut FILE` turns it on, with the bench's options (`BenchOptions`); a thread appends the
//! events to FILE as JSON lines (`{"event":"frame","start":…,"duration":…}`, nanoseconds since
//! the Unix epoch, like JFR's) twice a second, so the runner can stop the client at any time.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use md5::{Digest, Md5};
use serde_json::Value;

use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::io::netty::buffer::unpooled::wrapped_buffer;
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::renderer::block::block_state_model_set::BlockStateModelSet;
use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
use crate::net::minecraft::client::renderer::chunk::render_region_cache::RenderRegionCache;
use crate::net::minecraft::client::renderer::chunk::section_compiler::SectionCompiler;
use crate::net::minecraft::client::renderer::section_buffer_builder_pack::SectionBufferBuilderPack;
use crate::net::minecraft::client::renderer::texture::sprite_loader::SpriteLoader;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::resources::dry_foliage_color_reload_listener::DryFoliageColorReloadListener;
use crate::net::minecraft::client::resources::foliage_color_reload_listener::FoliageColorReloadListener;
use crate::net::minecraft::client::resources::grass_color_reload_listener::GrassColorReloadListener;
use crate::net::minecraft::client::resources::model::model_manager::ModelManager;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use crate::net::minecraft::network::protocol::game::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;
use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
use crate::net::minecraft::network::protocol::game::clientbound_light_update_packet::ClientboundLightUpdatePacket;
use crate::net::minecraft::network::protocol::game::clientbound_login_packet::ClientboundLoginPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket;
use crate::net::minecraft::network::protocol::game::clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::rustcraft::oracle::client::replay::Capture;

/// What an event times.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Frame,
    Submit,
    Tick,
    Compile,
    Light,
    Chunk,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Frame => "frame",
            Kind::Submit => "submit",
            Kind::Tick => "tick",
            Kind::Compile => "compile",
            Kind::Light => "light",
            Kind::Chunk => "chunk",
        }
    }
}

struct Event {
    kind: Kind,
    start: Instant,
    duration: Duration,
}

struct Recorder {
    /// When the recorder started, and that in nanoseconds since the Unix epoch.
    start: Instant,
    start_epoch_nanos: i128,
    events: Mutex<Vec<Event>>,
}

static RECORDER: OnceLock<Recorder> = OnceLock::new();

/// How often the events go to the file.
const WRITE_INTERVAL: Duration = Duration::from_millis(500);

/// Starts recording into `path`, which is created anew.
pub fn start(path: &Path) -> std::io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    let start = Instant::now();
    let start_epoch_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos() as i128);
    let recorder = RECORDER.get_or_init(|| Recorder {
        start,
        start_epoch_nanos,
        events: Mutex::new(Vec::new()),
    });
    std::thread::Builder::new()
        .name("Bench writer".to_string())
        .spawn(move || {
            loop {
                std::thread::sleep(WRITE_INTERVAL);
                let events = std::mem::take(
                    &mut *recorder
                        .events
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()),
                );
                for event in events {
                    let start = recorder.start_epoch_nanos
                        + event.start.duration_since(recorder.start).as_nanos() as i128;
                    let written = writeln!(
                        out,
                        r#"{{"event":"{}","start":{start},"duration":{}}}"#,
                        event.kind.name(),
                        event.duration.as_nanos()
                    );
                    if written.is_err() {
                        return;
                    }
                }
                if out.flush().is_err() {
                    return;
                }
            }
        })?;
    Ok(())
}

/// Now, if the benchmark is recording: what `record` takes as the event's start.
#[inline]
pub fn now() -> Option<Instant> {
    RECORDER.get().map(|_| Instant::now())
}

/// Records an event from `start` (`now()`'s) until now.
#[inline]
pub fn record(kind: Kind, start: Option<Instant>) {
    if let (Some(recorder), Some(start)) = (RECORDER.get(), start) {
        let duration = start.elapsed();
        recorder
            .events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(Event {
                kind,
                start,
                duration,
            });
    }
}

/// Records an instant event, now.
#[inline]
pub fn mark(kind: Kind) {
    record(kind, now());
}

/// The settings a bench runs both clients with: its `options`, the keys of a scenario's (window,
/// field of view, GUI scale, render distance, F1) and `max_fps`, vanilla's `maxFps` (260 for
/// unlimited). The rest are the oracle client's (`oracle/client/options.txt`), which `bench.py`
/// writes into vanilla's `options.txt` too.
#[derive(Clone, Debug)]
pub struct BenchOptions {
    pub width: i32,
    pub height: i32,
    pub fov: i32,
    pub gui_scale: i32,
    pub render_distance: i32,
    pub hide_gui: bool,
    pub max_fps: i32,
}

impl BenchOptions {
    pub fn load(path: &Path) -> Result<BenchOptions, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let root: Value =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let options = root
            .get("options")
            .ok_or_else(|| format!("{}: missing options", path.display()))?;
        let int = |key: &str| {
            options
                .get(key)
                .and_then(Value::as_i64)
                .map(|value| value as i32)
                .ok_or_else(|| format!("{}: options.{key} isn't an int", path.display()))
        };
        let window = options
            .get("window")
            .and_then(Value::as_array)
            .filter(|window| window.len() == 2)
            .ok_or_else(|| format!("{}: options.window isn't [w, h]", path.display()))?;
        Ok(BenchOptions {
            width: window[0].as_i64().unwrap_or(854) as i32,
            height: window[1].as_i64().unwrap_or(480) as i32,
            fov: int("fov")?,
            gui_scale: int("gui_scale")?,
            render_distance: int("render_distance")?,
            hide_gui: options
                .get("hide_gui")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            max_fps: int("max_fps")?,
        })
    }

    /// Sets the game's options to the bench's, and the oracle client's where it has none, like
    /// `OracleGameTest::new` sets a scenario's.
    pub fn apply(&self, minecraft: &Minecraft) {
        let mut options = minecraft.options.borrow_mut();
        options.set_render_distance(self.render_distance);
        options.set_particles(ParticleStatus::MINIMAL);
        options.joined_first_server = true;
        options.fov = self.fov;
        options.chunk_section_fade_in_time = 0.0;
        options.cloud_range = 64;
        options.texture_filtering_rgss = true;
        options.vignette = false;
        options.enable_vsync = false;
        options.gui_scale = self.gui_scale;
        options.framerate_limit = self.max_fps;
        drop(options);
        minecraft.hud_hidden.set(self.hide_gui);
    }
}

/// The thread's CPU time: what the offline benchmark times, which a busy machine's other
/// processes don't add to.
fn thread_cpu_time() -> Duration {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `time` is a valid timespec for the call to fill.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) };
    Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
}

/// `--benchLevel CAPTURE`, the offline benchmark (TESTING.md, "Benchmark"): a capture of a bench's
/// join (`bench.py --capture`) put in a level the way the listener puts it, and every section of
/// its chunks compiled, each part timed on its own in the thread's CPU time, without a server or
/// a window. It prints the times and a digest of every mesh and visibility set, which an
/// optimization must leave as it was.
pub fn level(capture_path: &Path) -> Result<(), String> {
    let capture = Capture::load(capture_path)
        .ok_or_else(|| format!("can't read {}", capture_path.display()))?;
    let friendly = |payload: &[i8]| FriendlyByteBuf::new(wrapped_buffer(payload.to_vec()));
    let millis = |duration: Duration| duration.as_secs_f64() * 1000.0;

    // What the configuration's registry data makes, then the login's level.
    let mut collector = RegistryDataCollector::new();
    for packet in &capture.packets {
        if !packet.serverbound
            && packet.state == "configuration"
            && packet.name == "minecraft:registry_data"
        {
            let packet =
                ClientboundRegistryDataPacket::STREAM_CODEC.decode(&mut friendly(&packet.payload));
            collector.append_contents(packet.registry(), packet.entries());
        }
    }
    let registries = Arc::new(collector.collect_game_registries());
    let registry_buffer = |payload: &[i8]| {
        RegistryFriendlyByteBuf::new(wrapped_buffer(payload.to_vec()), Arc::clone(&registries))
    };

    // The chunks the way the listener takes them: decoded (Netty's thread), handled (the game
    // thread's), then the light their handling queued, worked out.
    let mut level: Option<ClientLevel> = None;
    let mut chunks = BTreeSet::new();
    let (mut decode, mut handle) = (Duration::ZERO, Duration::ZERO);
    let mut chunk_count = 0;
    for packet in &capture.packets {
        if packet.serverbound || packet.state != "play" {
            continue;
        }
        if packet.name == "minecraft:login" {
            let login =
                ClientboundLoginPacket::STREAM_CODEC.decode(&mut registry_buffer(&packet.payload));
            let spawn_info = login.common_player_spawn_info();
            let mut new_level = ClientLevel::new(
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
            );
            new_level.set_biome_blend_radius(Options::new().biome_blend_radius());
            level = Some(new_level);
            continue;
        }
        let Some(level) = level.as_mut() else {
            continue;
        };
        match packet.name.as_str() {
            "minecraft:level_chunk_with_light" => {
                let start = thread_cpu_time();
                let chunk = ClientboundLevelChunkWithLightPacket::STREAM_CODEC
                    .decode(&mut registry_buffer(&packet.payload));
                let decoded = thread_cpu_time();
                ClientPacketListener::handle_level_chunk_with_light_in(level, &chunk);
                decode += decoded - start;
                handle += thread_cpu_time() - decoded;
                chunk_count += 1;
                chunks.insert((chunk.x(), chunk.z()));
            }
            "minecraft:light_update" => {
                let update = ClientboundLightUpdatePacket::STREAM_CODEC
                    .decode(&mut friendly(&packet.payload));
                ClientPacketListener::handle_light_update_packet_in(level, &update);
            }
            "minecraft:forget_level_chunk" => {
                let forget = ClientboundForgetLevelChunkPacket::STREAM_CODEC
                    .decode(&mut friendly(&packet.payload));
                ClientPacketListener::handle_forget_level_chunk_in(level, &forget);
                chunks.remove(&(forget.pos().x(), forget.pos().z()));
            }
            "minecraft:set_chunk_cache_center" => {
                let center = ClientboundSetChunkCacheCenterPacket::STREAM_CODEC
                    .decode(&mut friendly(&packet.payload));
                level
                    .get_chunk_source_mut()
                    .update_view_center(center.get_x(), center.get_z());
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
    let mut level = level.ok_or("the capture has no login")?;
    let start = thread_cpu_time();
    while level.has_light_updates() {
        level.update();
    }
    level.update();
    let light = thread_cpu_time() - start;

    // The renderer's models and tints, as the game renderer loads them (the atlases laid out
    // without a device), and a compiler with the oracle's options (ambient occlusion, cutout
    // leaves).
    let vanilla = VanillaPackResources::create_client_pack(
        &VanillaPackResources::vanilla_jar_root(),
        Path::new("no-asset-index"),
    );
    let layers = vanilla
        .resource_layers()
        .iter()
        .map(|layer| layer.clone() as Rc<dyn PackResources>)
        .collect();
    let manager = MultiPackResourceManager::new(PackType::CLIENT_RESOURCES, layers);
    GrassColorReloadListener::apply(GrassColorReloadListener::prepare(&manager));
    FoliageColorReloadListener::apply(FoliageColorReloadListener::prepare(&manager));
    DryFoliageColorReloadListener::apply(DryFoliageColorReloadListener::prepare(&manager));
    let stitch = |location: &str, definition: &str, max_mipmap_levels: i32| {
        SpriteLoader::new(Identifier::with_default_namespace(location), 16384)
            .stitch_layout(
                SpriteLoader::load_sprites(
                    &manager,
                    &Identifier::with_default_namespace(definition),
                    &[],
                ),
                max_mipmap_levels,
                0,
            )
            .map_err(|error| format!("couldn't stitch {}", error.sprite))
    };
    let blocks = stitch(TextureAtlas::LOCATION_BLOCKS, "blocks", 4)?;
    let items = stitch(TextureAtlas::LOCATION_ITEMS, "items", 0)?;
    let materials = MaterialBaker::new(&blocks, &items);
    let baked = ModelManager::bake_block_state_models(&manager, &materials);
    let models = Arc::new(BlockStateModelSet::new(
        baked.block_state_models,
        baked.missing_models.block,
    ));
    let fluid_models = Arc::new(FluidStateModelSet::new(
        FluidStateModelSet::bake(&materials),
        baked.missing_models.fluid,
    ));
    LeavesBlock::set_cutout_leaves(true);
    let compiler = SectionCompiler::new(
        true,
        true,
        models,
        fluid_models,
        Arc::new(BlockColors::create_default()),
    );

    // Every section of each chunk whose neighbours the level has, as the renderer compiles a
    // section once they're there, seen from above the middle of the chunks.
    let (Some(&(min_x, _)), Some(&(max_x, _))) = (chunks.first(), chunks.last()) else {
        return Err("the capture has no chunks".to_string());
    };
    let (min_z, max_z) = chunks
        .iter()
        .fold((i32::MAX, i32::MIN), |(min, max), &(_, z)| {
            (min.min(z), max.max(z))
        });
    let camera = [
        f64::from(min_x + max_x + 1) * 8.0,
        100.0,
        f64::from(min_z + max_z + 1) * 8.0,
    ];
    let loaded = |x: i32, z: i32| level.get_chunk_source().get_chunk(x, z).is_some();
    let mut buffers = SectionBufferBuilderPack::new();
    let mut cache = RenderRegionCache::new();
    let mut digest = Md5::new();
    let mut compiles = Vec::new();
    let mut meshed = 0;
    for &(x, z) in &chunks {
        if !(-1..=1).all(|dx| (-1..=1).all(|dz| loaded(x + dx, z + dz))) {
            continue;
        }
        for y in level.get_min_section_y()..=level.get_max_section_y() {
            let section_pos = SectionPos::of(x, y, z);
            let region = cache.create_region(&level, &level, section_pos.as_long());
            let sorting = VertexSorting::by_distance_xyz(
                (camera[0] - f64::from(section_pos.min_block_x())) as f32,
                (camera[1] - f64::from(section_pos.min_block_y())) as f32,
                (camera[2] - f64::from(section_pos.min_block_z())) as f32,
            );
            let start = thread_cpu_time();
            let results = compiler.compile(section_pos, &region, &sorting, &mut buffers);
            compiles.push(thread_cpu_time() - start);

            digest.update(section_pos.as_long().to_le_bytes());
            for (layer, mesh) in &results.rendered_layers {
                digest.update([*layer as u8]);
                digest.update(mesh.vertex_buffer());
                if let Some(indices) = mesh.index_buffer() {
                    digest.update(indices);
                }
            }
            let mut visibility = 0u64;
            for from in Direction::VALUES {
                for to in Direction::VALUES {
                    if results.visibility_set.visibility_between(from, to) {
                        visibility |= 1 << (from as u32 + 6 * to as u32);
                    }
                }
            }
            digest.update(visibility.to_le_bytes());
            meshed += usize::from(!results.rendered_layers.is_empty());
        }
    }
    let total: Duration = compiles.iter().sum();
    compiles.sort();
    let digest: [u8; 16] = digest.finalize().into();
    println!(
        "chunks: {chunk_count}, decoded in {:.1} ms, handled in {:.1} ms",
        millis(decode),
        millis(handle)
    );
    println!(
        "light: {:.1} ms ({:.1} us a chunk)",
        millis(light),
        millis(light) * 1000.0 / f64::from(chunk_count)
    );
    println!(
        "sections: {} compiled ({meshed} with a mesh) in {:.1} ms, median {:.1} us, mean {:.1} us",
        compiles.len(),
        millis(total),
        compiles[compiles.len() / 2].as_secs_f64() * 1e6,
        total.as_secs_f64() * 1e6 / compiles.len() as f64
    );
    println!(
        "digest: {}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    Ok(())
}
