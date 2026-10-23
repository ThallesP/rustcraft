//! The client's entry point. With a server to join, it's `net/minecraft/client/main/Main.java`'s
//! `main` (`Main`):
//!
//!   rustcraft --server HOST:PORT --username NAME   joins the server, in a window (--headless: none)
//!   rustcraft --ping HOST:PORT                     pings it like the server list, printing its status
//!
//! The render track's frames on the OpenGL backend, the way vanilla starts it: SDL's video
//! subsystem, the GL library, the device on its hidden window, then the main render target
//! cleared like `GameRenderer.render` clears it at the start of each frame:
//!
//!   rustcraft --device-info                     prints the device as vanilla logs it
//!   rustcraft --compile-pipelines               compiles every pipeline `ShaderManager` does,
//!                                               printing how many linked
//!   rustcraft --dump-atlas DIR                  loads the block atlas from the vanilla pack and
//!                                               dumps it like `TextureAtlas.dumpContents`
//!   rustcraft --screenshot out.png [--size WxH] [--clear AARRGGBB] [--quadrant AARRGGBB]
//!             [--gui-scale N] [--fill X0,Y0,X1,Y1,AARRGGBB]...
//!                                               renders a frame offscreen and writes it as a PNG
//!   rustcraft --window [--size WxH] [--clear AARRGGBB] [--frames N] [--fill ...]...
//!                                               shows frames in a window until it's closed
//!   rustcraft --render-scenario SCENARIO --capture CAPTURE --screenshot out.png [--start-size WxH]
//!                                               renders the scenario's first view of the level a
//!                                               capture's packets fill, offscreen, without a server:
//!                                               the player put where the scenario says and left to
//!                                               land, frames until the sections have compiled, and
//!                                               the HUD unless the scenario hides it; with
//!                                               `--start-size`, the first frame at that size, then
//!                                               the window resized to the scenario's
//!
//! The clear color defaults to `GuiRenderState.clearColorOverride`'s, transparent black; the
//! loading overlay's is its brand background, `FFEF323D`. `--quadrant` clears the lower left
//! quarter in GL's coordinates through a render pass's area, which shows the readback's
//! orientation. Each `--fill` is `GuiGraphicsExtractor.fill` in GUI pixels, drawn in order
//! through `pipeline/gui` the way `GuiRenderer` draws, at the GUI scale (default 1).

use rustcraft::com::mojang::blaze3d::pipeline::main_target::MainTarget;
use rustcraft::com::mojang::blaze3d::platform::window::Window;
use rustcraft::com::mojang::blaze3d::systems::render_system::RenderSystem;
use rustcraft::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use rustcraft::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use rustcraft::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use rustcraft::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use rustcraft::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use rustcraft::com::mojang::renderpearl::api::buffers::gpu_buffer;
use rustcraft::com::mojang::renderpearl::api::commands::render_pass::RenderArea;
use rustcraft::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use rustcraft::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use rustcraft::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use rustcraft::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use rustcraft::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use rustcraft::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use rustcraft::com::mojang::renderpearl::backend::opengl::gl_backend::GlBackend;
use rustcraft::com::mojang::renderpearl::frontend::shaders::pipeline_builder::PipelineBuilder;
use rustcraft::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use rustcraft::net::minecraft::client::renderer::projection::Projection;
use rustcraft::net::minecraft::client::renderer::projection_matrix_buffer::ProjectionMatrixBuffer;
use rustcraft::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use rustcraft::net::minecraft::client::renderer::shader_manager::{Configs, ShaderManager};
use rustcraft::net::minecraft::client::renderer::texture::sprite_loader::SpriteLoader;
use rustcraft::net::minecraft::client::renderer::texture::texture_atlas::{
    AnimationPipelines, TextureAtlas,
};
use rustcraft::net::minecraft::client::screenshot::Screenshot;
use rustcraft::net::minecraft::resources::identifier::Identifier;
use rustcraft::net::minecraft::server::packs::pack_resources::PackResources;
use rustcraft::net::minecraft::server::packs::pack_type::PackType;
use rustcraft::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use rustcraft::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use rustcraft::net::minecraft::util::argb::ARGB;
use rustcraft::org::joml::matrix3x2f::Matrix3x2f;
use rustcraft::org::joml::matrix4f::Matrix4f;
use rustcraft::org::joml::vector4f::Vector4f;
use rustcraft::org::lwjgl::sdl::sdl_events::{SDLEvents, SdlEvent};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

enum Mode {
    DeviceInfo,
    CompilePipelines,
    DumpAtlas(PathBuf),
    Screenshot(PathBuf),
    Window,
    RenderScenario(PathBuf),
}

struct Arguments {
    mode: Mode,
    capture: Option<PathBuf>,
    screenshot: Option<PathBuf>,
    start_size: Option<(i32, i32)>,
    width: i32,
    height: i32,
    clear_color: Vector4f,
    quadrant_color: Option<Vector4f>,
    frames: Option<u32>,
    gui_scale: i32,
    fills: Vec<Fill>,
}

/// `GuiGraphicsExtractor.fill`'s arguments.
#[derive(Clone, Copy)]
struct Fill {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: i32,
}

fn parse_fill(text: &str) -> Result<Fill, String> {
    let parts: Vec<&str> = text.split(',').collect();
    let [x0, y0, x1, y1, color] = parts[..] else {
        return Err(format!("--fill takes X0,Y0,X1,Y1,AARRGGBB, not {text}"));
    };
    let coordinate = |part: &str| part.parse().map_err(|_| format!("bad coordinate {part}"));
    Ok(Fill {
        x0: coordinate(x0)?,
        y0: coordinate(y0)?,
        x1: coordinate(x1)?,
        y1: coordinate(y1)?,
        color: u32::from_str_radix(color, 16).map_err(|_| format!("bad color {color}"))? as i32,
    })
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut mode = None;
    let (mut width, mut height) = (854, 480);
    let mut clear_color = Vector4f::from_scalar(0.0);
    let mut quadrant_color = None;
    let mut frames = None;
    let mut gui_scale = 1;
    let mut fills = Vec::new();
    let mut capture = None;
    let mut screenshot = None;
    let mut start_size = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        let mut value = || arguments.next().ok_or(format!("{argument} needs a value"));
        match argument.as_str() {
            "--device-info" => mode = Some(Mode::DeviceInfo),
            "--compile-pipelines" => mode = Some(Mode::CompilePipelines),
            "--screenshot" => {
                let path = PathBuf::from(value()?);
                if !matches!(mode, Some(Mode::RenderScenario(_))) {
                    mode = Some(Mode::Screenshot(path.clone()));
                }
                screenshot = Some(path);
            }
            "--render-scenario" => mode = Some(Mode::RenderScenario(PathBuf::from(value()?))),
            "--capture" => capture = Some(PathBuf::from(value()?)),
            "--window" => mode = Some(Mode::Window),
            "--dump-atlas" => mode = Some(Mode::DumpAtlas(PathBuf::from(value()?))),
            "--size" => {
                let size = value()?;
                let (w, h) = size
                    .split_once('x')
                    .ok_or(format!("--size takes WxH, not {size}"))?;
                width = w.parse().map_err(|_| format!("bad width {w}"))?;
                height = h.parse().map_err(|_| format!("bad height {h}"))?;
            }
            "--start-size" => {
                let size = value()?;
                let (w, h) = size
                    .split_once('x')
                    .ok_or(format!("--start-size takes WxH, not {size}"))?;
                start_size = Some((
                    w.parse().map_err(|_| format!("bad width {w}"))?,
                    h.parse().map_err(|_| format!("bad height {h}"))?,
                ));
            }
            "--clear" => {
                let hex = value()?;
                let argb = u32::from_str_radix(&hex, 16)
                    .map_err(|_| format!("--clear takes AARRGGBB, not {hex}"))?;
                clear_color = ARGB::vector4f_from_argb32(argb as i32);
            }
            "--quadrant" => {
                let hex = value()?;
                let argb = u32::from_str_radix(&hex, 16)
                    .map_err(|_| format!("--quadrant takes AARRGGBB, not {hex}"))?;
                quadrant_color = Some(ARGB::vector4f_from_argb32(argb as i32));
            }
            "--gui-scale" => {
                gui_scale = value()?
                    .parse()
                    .map_err(|_| "--gui-scale takes a scale".to_owned())?
            }
            "--fill" => fills.push(parse_fill(&value()?)?),
            "--frames" => {
                frames = Some(
                    value()?
                        .parse()
                        .map_err(|_| "--frames takes a count".to_owned())?,
                )
            }
            _ => return Err(format!("unknown argument {argument}")),
        }
    }
    Ok(Arguments {
        mode: mode
            .ok_or("pass --device-info, --compile-pipelines, --screenshot PATH or --window")?,
        capture,
        screenshot,
        start_size,
        width,
        height,
        clear_color,
        quadrant_color,
        frames,
        gui_scale,
        fills,
    })
}

/// What the GUI draws with: `pipeline/gui` compiled like `ShaderManager` compiles it, its
/// projection's buffer and the shared quad indices.
struct Gui {
    pipeline: Rc<dyn CompiledRenderPipeline>,
    projection: Projection,
    projection_matrix_buffer: ProjectionMatrixBuffer,
    quad_indices: rustcraft::com::mojang::blaze3d::systems::render_system::AutoStorageIndexBuffer,
}

/// The vanilla pack's shaders, and a pipeline builder for the device.
fn shaders(device: &dyn GpuDevice) -> Result<(Configs, PipelineBuilder), String> {
    let pack = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla")
        .join(include_str!("../MINECRAFT_VERSION").trim())
        .join("resources");
    let configs = ShaderManager::load_configs(&pack)
        .map_err(|error| format!("can't load the shaders from {}: {error}", pack.display()))?;
    let info = device.get_device_info();
    let builder = PipelineBuilder::new(
        info.is_z_zero_to_one,
        info.features.shader_draw_parameters,
        info.hints_and_workarounds.is_explicit_depth_required,
    );
    Ok((configs, builder))
}

/// The block atlas as the client loads it at startup (`AtlasManager`'s blocks entry, with
/// `Options`' default mipmap levels, 4, and no anisotropic filtering), dumped into `dir`.
fn dump_block_atlas(device: &dyn GpuDevice, dir: &Path) -> Result<(), String> {
    let (configs, builder) = shaders(device)?;
    let compile = |pipeline| {
        builder
            .compile_pipeline(pipeline, &configs, device)
            .map(|pipeline| pipeline as Rc<dyn CompiledRenderPipeline>)
            .ok_or("an animation pipeline didn't compile")
    };
    let pipelines = AnimationPipelines {
        blit: compile(&RenderPipelines::get().ANIMATE_SPRITE_BLIT)?,
        interpolate: compile(&RenderPipelines::get().ANIMATE_SPRITE_INTERPOLATE)?,
    };
    let sampler_cache = SamplerCache::new(device);
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
    let mut atlas = TextureAtlas::new(
        device,
        Identifier::with_default_namespace(TextureAtlas::LOCATION_BLOCKS),
    );
    let preparations =
        SpriteLoader::new(atlas.location().clone(), atlas.max_supported_texture_size())
            .load_and_stitch(
                &manager,
                &Identifier::with_default_namespace("blocks"),
                4,
                0,
                &[],
            )
            .map_err(|error| format!("couldn't stitch {}", error.sprite))?;
    atlas.upload(device, &sampler_cache, &pipelines, preparations);
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let location = atlas.location().clone();
    atlas
        .dump_contents(device, &location, dir)
        .map_err(|error| format!("can't dump the atlas into {}: {error}", dir.display()))?;
    println!(
        "dumped {} ({}x{}, {} mip levels)",
        location,
        atlas.get_width(),
        atlas.get_height(),
        atlas.max_mip_level() + 1
    );
    Ok(())
}

impl Gui {
    fn new(device: &dyn GpuDevice) -> Result<Gui, String> {
        let (configs, builder) = shaders(device)?;
        let pipeline: Rc<dyn CompiledRenderPipeline> = builder
            .compile_pipeline(&RenderPipelines::get().GUI, &configs, device)
            .ok_or("pipeline/gui didn't compile")?;
        Ok(Gui {
            pipeline,
            projection: Projection::new(),
            projection_matrix_buffer: ProjectionMatrixBuffer::new(device, "gui"),
            quad_indices: RenderSystem::sequential_buffer(PrimitiveTopology::QUADS),
        })
    }

    /// `GuiRenderer.draw` for fills: their quads in one mesh, drawn in one render pass over the
    /// main target with the GUI's orthographic projection and model view.
    fn draw(
        &mut self,
        device: &dyn GpuDevice,
        main_target: &MainTarget,
        gui_scale: i32,
        fills: &[Fill],
    ) {
        if fills.is_empty() {
            return;
        }
        let mut byte_buffer = ByteBufferBuilder::new(786432);
        let mut builder = BufferBuilder::new(
            &mut byte_buffer,
            PrimitiveTopology::QUADS,
            DefaultVertexFormat::POSITION_COLOR,
        );
        let pose = Matrix3x2f::new();
        for fill in fills {
            // GuiGraphicsExtractor.fill puts the larger coordinates first.
            let (x0, x1) = if fill.x0 < fill.x1 {
                (fill.x1, fill.x0)
            } else {
                (fill.x0, fill.x1)
            };
            let (y0, y1) = if fill.y0 < fill.y1 {
                (fill.y1, fill.y0)
            } else {
                (fill.y0, fill.y1)
            };
            // ColoredRectangleRenderState.buildVertices.
            builder
                .add_vertex_with_2d_pose(&pose, x0 as f32, y0 as f32)
                .set_color(fill.color);
            builder
                .add_vertex_with_2d_pose(&pose, x0 as f32, y1 as f32)
                .set_color(fill.color);
            builder
                .add_vertex_with_2d_pose(&pose, x1 as f32, y1 as f32)
                .set_color(fill.color);
            builder
                .add_vertex_with_2d_pose(&pose, x1 as f32, y0 as f32)
                .set_color(fill.color);
        }
        let mesh = builder.build_or_throw();
        let draw_state = *mesh.draw_state();
        let vertex_buffer = device.create_buffer_with_data(
            Some("GUI vertices"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        );
        let index_buffer = self.quad_indices.get_buffer(device, draw_state.index_count);

        self.projection.setup_ortho(
            1000.0,
            11000.0,
            main_target.width as f32 / gui_scale as f32,
            main_target.height as f32 / gui_scale as f32,
            true,
        );
        let projection = self
            .projection_matrix_buffer
            .get_buffer(device, &mut self.projection);
        let mut model_view = Matrix4f::new();
        model_view.set_translation(0.0, 0.0, -11000.0);
        let transform = Transform::of_model_view(model_view).write();
        let dynamic_transforms = device.create_buffer_with_data(
            Some("Dynamic Transforms UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &transform,
        );

        let encoder = device.create_command_encoder();
        let mut pass = encoder.create_render_pass_with(
            "GUI",
            main_target
                .get_color_texture_view()
                .expect("the main target has color"),
            None,
            main_target.get_depth_texture_view(),
            None,
        );
        // RenderSystem.bindDefaultUniforms: only the projection is set so far.
        pass.set_uniform_slice("Projection", &projection);
        pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&vertex_buffer)));
        pass.set_index_buffer(&index_buffer, self.quad_indices.index_type());
        pass.draw_indexed(draw_state.index_count, 1, 0, 0, 0);
    }
}

/// `GameRenderer.render`'s first command: the main target cleared to the clear color, depth to 0
/// (reversed depth), then a render pass over it, like the GUI's, which draws nothing yet.
fn render_frame(
    device: &dyn GpuDevice,
    main_target: &MainTarget,
    arguments: &Arguments,
    gui: &mut Option<Gui>,
) {
    let clear_color = &arguments.clear_color;
    let encoder = device.create_command_encoder();
    encoder.clear_color_and_depth_textures(
        &**main_target
            .get_color_texture()
            .expect("the main target has color"),
        clear_color,
        &**main_target
            .get_depth_texture()
            .expect("the main target has depth"),
        0.0,
    );
    let pass = encoder.create_render_pass_with(
        "GUI",
        main_target
            .get_color_texture_view()
            .expect("the main target has color"),
        None,
        None,
        None,
    );
    drop(pass);

    if let Some(gui) = gui {
        gui.draw(device, main_target, arguments.gui_scale, &arguments.fills);
    }

    if let Some(quadrant_color) = arguments.quadrant_color {
        let view = main_target
            .get_color_texture_view()
            .expect("the main target has color");
        let descriptor = RenderPassDescriptor::builder("Quadrant")
            .with_color_attachment(view.clone(), Some(quadrant_color))
            .with_render_area(RenderArea {
                x: 0,
                y: 0,
                width: main_target.width / 2,
                height: main_target.height / 2,
            })
            .build();
        drop(encoder.create_render_pass(descriptor));
    }
}

/// `--render-scenario`: the level a capture fills, its clocks set by the capture's `set_time`s,
/// the player where the scenario puts it, settled like the oracle client's settle, a frame each
/// tick until it has landed and the sections have compiled, and a screenshot of the last.
fn render_scenario(
    device: &'static dyn GpuDevice,
    scenario: &Path,
    capture: &Path,
    out: &Path,
    start_size: Option<(i32, i32)>,
) -> Result<(), String> {
    use rustcraft::io::netty::buffer::unpooled::wrapped_buffer;
    use rustcraft::net::minecraft::client::client_clock_manager::ClientClockManager;
    use rustcraft::net::minecraft::client::gui::gui::Gui;
    use rustcraft::net::minecraft::client::gui::hud::HudContext;
    use rustcraft::net::minecraft::client::options::Options;
    use rustcraft::net::minecraft::client::player::local_player::LocalPlayer;
    use rustcraft::net::minecraft::client::renderer::chunk::render_region_cache::LevelEnvironment;
    use rustcraft::net::minecraft::client::renderer::game_renderer::{
        CameraEntityState, GameRenderer,
    };
    use rustcraft::net::minecraft::core::registries::registries::Registries;
    use rustcraft::net::minecraft::network::codec::stream_codec::StreamCodec;
    use rustcraft::net::minecraft::network::protocol::game::clientbound_set_time_packet::ClientboundSetTimePacket;
    use rustcraft::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
    use rustcraft::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
    use rustcraft::net::minecraft::world::level::game_type::GameType;
    use rustcraft::net::minecraft::world::phys::vec3::Vec3;
    use rustcraft::rustcraft::oracle::client::oracle_game_test::OracleGameTest;
    use rustcraft::rustcraft::oracle::client::replay::{Capture, CapturedLevel};
    use rustcraft::rustcraft::oracle::client::scenario::Scenario;

    let scenario = Scenario::load(scenario)?;
    let capture = Capture::load(capture).ok_or(format!("can't read {}", capture.display()))?;
    let captured = CapturedLevel::from_capture_with_light(&capture);
    let level = captured.level;
    let mut clocks = ClientClockManager::new();
    for packet in &capture.packets {
        if !packet.serverbound && packet.name == "minecraft:set_time" {
            let mut input = RegistryFriendlyByteBuf::new(
                wrapped_buffer(packet.payload.clone()),
                std::sync::Arc::clone(&captured.registries),
            );
            let set_time = ClientboundSetTimePacket::STREAM_CODEC.decode(&mut input);
            clocks.handle_updates(set_time.game_time(), &set_time.clock_updates());
        }
    }
    let biomes = captured.registries.lookup_or_throw(Registries::BIOME);
    let biome_keys: Vec<_> = (0..biomes.size())
        .filter_map(|id| biomes.holder_by_id_or_throw(id).unwrap_key().cloned())
        .collect();
    let dimension_type = level
        .dimension_type_registration()
        .unwrap_key()
        .ok_or("the login's dimension type has a key")?
        .identifier()
        .clone();

    let mut options = Options::new();
    options.gui_scale = scenario.options.gui_scale;
    let mut gui = Gui::new();
    let player_mode = match scenario.world.game_mode.as_str() {
        "creative" => GameType::CREATIVE,
        "spectator" => GameType::SPECTATOR,
        _ => GameType::SURVIVAL,
    };
    let (width, height) = start_size.unwrap_or((scenario.options.width, scenario.options.height));
    let mut game_renderer = GameRenderer::new(device, width, height);
    game_renderer.set_environment_attributes(EnvironmentAttributeSystem::for_client_level(
        &dimension_type,
        &biome_keys,
        true,
    )?);
    let render_options = OracleGameTest::render_options(&scenario);
    let mut player = LocalPlayer::new(None);
    // The scenario's player is the offline player `rustcraft`, whose id picks its skin.
    {
        use rustcraft::net::minecraft::core::uuid_util::UUIDUtil;
        use rustcraft::net::minecraft::world::entity::entity::EntityClass;
        player
            .entity_mut()
            .set_uuid(UUIDUtil::create_offline_player_uuid("rustcraft"));
    }
    player.set_pos(Vec3::new(
        scenario.player.x,
        scenario.player.y,
        scenario.player.z,
    ));
    player.set_y_rot(scenario.player.y_rot);
    player.set_x_rot(scenario.player.x_rot);
    // The server echoes the main arm and the skin's parts the client sent (`set_entity_data`).
    let information = options.build_player_information();
    player.main_arm = information.main_hand();
    player.model_customisation = information.model_customisation() as i8;
    game_renderer.set_level(true);
    let environment = LevelEnvironment::new(&level, clocks);
    // The texture animations at their first frames, drawn like the scenario start draws them: the
    // frames rendered here don't tick them, like those before the marker don't count.
    game_renderer.restart_texture_animations();
    let mut ticks = 0;
    while ticks < OracleGameTest::SETTLE_TICKS * 3 || !player.on_ground() {
        // The window resized to the scenario's size after the first frame, like
        // `Minecraft.render_level`'s.
        if ticks == 1 {
            game_renderer.follow_window_size(scenario.options.width, scenario.options.height);
        }
        gui.tick(false);
        game_renderer.tick(true, Some(&player), Some(&environment), &render_options);
        player.common_tick();
        player.tick(&level, &options);
        game_renderer.update(Some(&player), Some(&level), 1.0, &render_options);
        let eye_y = player.get_y() + f64::from(player.get_eye_height());
        game_renderer.render_level_frame(
            &level,
            &level,
            &environment,
            Some(&player),
            &render_options,
            1.0,
            1.0,
            level.get_game_time(),
            eye_y,
            &CameraEntityState::of(Some(&player)),
        );
        let font = Rc::clone(game_renderer.font());
        let hud_context = HudContext {
            font: &font,
            is_hidden: scenario.options.hide_gui,
            player: &player,
            player_mode,
            can_hurt_player: player_mode.is_survival(),
            has_experience: player_mode.is_survival(),
            is_hardcore: level.is_hardcore(),
        };
        game_renderer.extract_gui(
            options.gui_scale,
            options.get_background_color(0.0),
            0,
            0,
            |graphics| {
                gui.extract_render_state(graphics, Some(&hud_context));
            },
        );
        game_renderer.render_gui(options.gui_scale);
        ticks += 1;
        if ticks > OracleGameTest::LOAD_TIMEOUT_TICKS {
            return Err("the player never landed".to_string());
        }
    }
    let image = game_renderer.take_screenshot();
    image
        .write_to_file(out)
        .map_err(|error| format!("can't write {}: {error}", out.display()))?;
    println!(
        "wrote {} ({}x{})",
        out.display(),
        image.get_width(),
        image.get_height()
    );
    Ok(())
}

fn run(arguments: Arguments) -> Result<(), String> {
    RenderSystem::init_backend_system();
    let mut backend = GlBackend::new();
    backend.load_library().map_err(|error| error.to_string())?;
    let debug_options = GpuDebugOptions {
        log_level: 1,
        ..GpuDebugOptions::default()
    };
    let device = backend
        .create_device(&debug_options)
        .map_err(|error| error.to_string())?;
    let info = device.get_device_info();
    let mut gui = if arguments.fills.is_empty() {
        None
    } else {
        Some(Gui::new(&*device)?)
    };

    match arguments.mode {
        Mode::DeviceInfo => {
            println!(
                "Using graphics backend {}, using drivers: {}",
                info.backend_name, info.driver_info
            );
            println!(
                "Using graphics device: {} ({})",
                info.name, info.vendor_name
            );
            println!(
                "Using graphics device extensions: {}",
                info.underlying_extensions.join(", ")
            );
            println!(
                "isZZeroToOne: {}, shaderDrawParameters: {}, isExplicitDepthRequired: {}",
                info.is_z_zero_to_one,
                info.features.shader_draw_parameters,
                info.hints_and_workarounds.is_explicit_depth_required
            );
        }
        Mode::CompilePipelines => {
            // ShaderManager.reload compiles the required pipelines, then the optional ones.
            let (configs, builder) = shaders(&*device)?;
            let pipelines: Vec<_> = RenderPipelines::required_pipelines()
                .into_iter()
                .chain(RenderPipelines::optional_pipelines())
                .collect();
            let mut compiled = 0;
            for pipeline in &pipelines {
                match builder.compile_pipeline(pipeline, &configs, &*device) {
                    Some(_) => compiled += 1,
                    None => println!("failed: {}", pipeline.get_location()),
                }
            }
            println!("compiled {compiled} of {} pipelines", pipelines.len());
        }
        Mode::DumpAtlas(ref dir) => dump_block_atlas(&*device, dir)?,
        Mode::RenderScenario(ref scenario) => {
            let capture = arguments
                .capture
                .as_ref()
                .ok_or("--render-scenario needs --capture")?;
            let path = arguments
                .screenshot
                .as_ref()
                .ok_or("--render-scenario needs --screenshot")?;
            render_scenario(
                Box::leak(device),
                scenario,
                capture,
                path,
                arguments.start_size,
            )?;
            return Ok(());
        }
        Mode::Screenshot(ref path) => {
            let main_target = MainTarget::new(&*device, arguments.width, arguments.height);
            render_frame(&*device, &main_target, &arguments, &mut gui);
            let image = Screenshot::take_screenshot(&*device, &main_target, 1);
            image
                .write_to_file(path)
                .map_err(|error| format!("can't write {}: {error}", path.display()))?;
            println!(
                "wrote {} ({}x{})",
                path.display(),
                image.get_width(),
                image.get_height()
            );
        }
        Mode::Window => {
            let window = Window::new(
                &backend,
                arguments.width,
                arguments.height,
                "Minecraft* 26.3 (rustcraft)",
            );
            let surface = device.create_surface(window.handle());
            let main_target = MainTarget::new(&*device, window.get_width(), window.get_height());
            let mut frame = 0;
            'frames: while arguments.frames.is_none_or(|frames| frame < frames) {
                let mut event = SdlEvent::default();
                while SDLEvents::SDL_PollEvent(&mut event) {
                    if event.event_type == SDLEvents::SDL_EVENT_QUIT
                        || event.event_type == SDLEvents::SDL_EVENT_WINDOW_CLOSE_REQUESTED
                    {
                        break 'frames;
                    }
                }
                render_frame(&*device, &main_target, &arguments, &mut gui);
                surface.blit_from_texture(
                    main_target
                        .get_color_texture_view()
                        .expect("the main target has color"),
                );
                surface.present();
                frame += 1;
            }
        }
    }
    Ok(())
}

/// The render track's modes, which `parse_arguments` reads; anything else goes to `Main`.
const RENDER_MODES: [&str; 6] = [
    "--device-info",
    "--compile-pipelines",
    "--dump-atlas",
    "--screenshot",
    "--window",
    "--render-scenario",
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let joins = args
        .iter()
        .any(|arg| ["--server", "--quickPlayMultiplayer"].contains(&arg.as_str()));
    if joins || !args.iter().any(|arg| RENDER_MODES.contains(&arg.as_str())) {
        // An exit status's low byte, like Java's `System.exit(-5)`.
        let status = rustcraft::net::minecraft::client::main::main::Main::main(&args);
        return ExitCode::from(status as u8);
    }

    match parse_arguments().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rustcraft: {error}");
            ExitCode::FAILURE
        }
    }
}
