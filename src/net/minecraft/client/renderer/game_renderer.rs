//! net/minecraft/client/renderer/GameRenderer.java
//!
//! A frame: the main target cleared, the global settings, the lightmap, then the level (the
//! camera's projection, the fog, and `LevelRenderer`'s passes), then the GUI. What the frame reads
//! of the game is extracted first (`extract`: the window, the options, the camera, the fog, the
//! lightmap's and the sky's states), the way Java's `GameRenderState` is. The renderer holds the
//! device it was made with (Java's `RenderSystem.getDevice()`), the block atlas, the baked block
//! models and the block colours (Java's `ModelManager`, `AtlasManager` and
//! `Minecraft.blockColors`, which the resource reload fills, with the colormaps), the level
//! renderer (Java's `Minecraft.levelRenderer`), and the GUI's atlas, the texture manager it's
//! registered in, the GUI's render state (`GameRenderState.guiRenderState`) and its renderer.
//!
//! The 3D HUD (`render3dHud`) draws the first-person hands (`renderItemInHand`) after the level,
//! with the HUD's own projection over a cleared depth, through the feature renderers
//! (`FeatureRenderDispatcher`, with the hands' own `SubmitNodeStorage`), lit by `Lighting`'s
//! level entry, which the level's main pass sets up in Java.
//!
//! Not ported yet: the screen effects (`ScreenEffectRenderer`), the view's bobbing (the oracle
//! turns it off), the nausea and portal warp, post effects (`end_of_frame` has no chain in the
//! vanilla pack, so a frame applies none), the panorama, the 3D HUD's depth target
//! (`hud3DTarget`, only when post effects need a consistent depth), the debug crosshair, and the
//! boss overlay's darkening.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::pipeline::main_target::MainTarget;
use crate::com::mojang::blaze3d::platform::lighting::{Entry, Lighting};
use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::platform::window::Window;
use crate::com::mojang::blaze3d::systems::render_system::{AutoStorageIndexBuffer, RenderSystem};
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::com::mojang::math::axis::Axis;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::frontend::shaders::pipeline_builder::PipelineBuilder;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::camera::{Camera, CameraOptions};
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::font::font_manager::FontManager;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::render::gui_renderer::{
    GlyphTextures, GuiRenderer, GuiWindow,
};
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::block::block_state_model_set::BlockStateModelSet;
use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::chunk_sections_to_render::TerrainPipelines;
use crate::net::minecraft::client::renderer::entity::entity_render_dispatcher::EntityRenderDispatcher;
use crate::net::minecraft::client::renderer::entity::player::avatar_renderer::AvatarEntity;
use crate::net::minecraft::client::renderer::entity::state::avatar_render_state::AvatarRenderState;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::feature::feature_render_dispatcher::FeatureRenderDispatcher;
use crate::net::minecraft::client::renderer::first_person_hands_and_items_renderer::FirstPersonHandsAndItemsRenderer;
use crate::net::minecraft::client::renderer::fog::environment::atmospheric_fog_environment::AtmosphericFogEnvironment;
use crate::net::minecraft::client::renderer::fog::environment::lava_fog_environment::LavaFogEnvironment;
use crate::net::minecraft::client::renderer::fog::environment::powdered_snow_fog_environment::PowderedSnowFogEnvironment;
use crate::net::minecraft::client::renderer::fog::environment::water_fog_environment::WaterFogEnvironment;
use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::client::renderer::fog::fog_renderer::{FogMode, FogRenderer};
use crate::net::minecraft::client::renderer::global_settings_uniform::GlobalSettingsUniform;
use crate::net::minecraft::client::renderer::level_renderer::{
    LevelRenderOptions, LevelRenderer, LevelResources,
};
use crate::net::minecraft::client::renderer::lightmap::Lightmap;
use crate::net::minecraft::client::renderer::projection::Projection;
use crate::net::minecraft::client::renderer::projection_matrix_buffer::ProjectionMatrixBuffer;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::shader_manager::{Configs, ShaderManager};
use crate::net::minecraft::client::renderer::sky_renderer::{SkyPipelines, SkyRenderState};
use crate::net::minecraft::client::renderer::staged_vertex_buffer::StagedVertexBuffer;
use crate::net::minecraft::client::renderer::state::gui::gui_render_state::GuiRenderState;
use crate::net::minecraft::client::renderer::state::level::camera_render_state::CameraRenderState;
use crate::net::minecraft::client::renderer::state::level::player_render_state::PlayerRenderState;
use crate::net::minecraft::client::renderer::state::lightmap_render_state::LightmapRenderState;
use crate::net::minecraft::client::renderer::submit_node_storage::SubmitNodeStorage;
use crate::net::minecraft::client::renderer::texture::overlay_texture::OverlayTexture;
use crate::net::minecraft::client::renderer::texture::sprite_contents::AdditionalMetadataType;
use crate::net::minecraft::client::renderer::texture::sprite_loader::SpriteLoader;
use crate::net::minecraft::client::renderer::texture::texture_atlas::{
    AnimationPipelines, TextureAtlas,
};
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;
use crate::net::minecraft::client::renderer::ui_lightmap::UiLightmap;
use crate::net::minecraft::client::resources::dry_foliage_color_reload_listener::DryFoliageColorReloadListener;
use crate::net::minecraft::client::resources::foliage_color_reload_listener::FoliageColorReloadListener;
use crate::net::minecraft::client::resources::grass_color_reload_listener::GrassColorReloadListener;
use crate::net::minecraft::client::resources::model::model_manager::ModelManager;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::client::screenshot::Screenshot;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::living_entity::LivingEntityClass;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::cardinal_lighting;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::material::fog_type::FogType;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector4f::Vector4f;

/// The options a frame reads (`OptionsRenderState` and what `Camera` asks).
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub render_distance: i32,
    pub fov: i32,
    pub fov_effect_scale: f32,
    pub cloud_range: i32,
    pub ambient_occlusion: bool,
    pub cutout_leaves: bool,
    pub chunk_section_fade_in_time: f64,
    pub gamma: f32,
    pub glint_strength: f64,
    /// `textureFiltering() == RGSS`.
    pub use_rgss: bool,
    pub hide_gui: bool,
}

/// What the frame reads of the environment at the camera: Java asks the camera's
/// `EnvironmentAttributeProbe` for each, in `FogRenderer`, `SkyRenderer` and
/// `LightmapRenderStateExtractor`.
#[derive(Clone, Debug, Default)]
pub struct FrameEnvironment {
    pub fog: FogData,
    pub sky: SkyRenderState,
    pub lightmap: LightmapRenderState,
}

/// What the fog and the sky read of the camera's entity: Java asks the entity itself in
/// `FogRenderer` and `SkyRenderer`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraEntityState {
    /// `LocalPlayer.getWaterVision()`, for a camera on the local player.
    pub water_vision: Option<f32>,
    pub is_under_water: bool,
    pub is_spectator: bool,
}

impl CameraEntityState {
    pub fn of(player: Option<&LocalPlayer>) -> CameraEntityState {
        player.map_or_else(CameraEntityState::default, |player| CameraEntityState {
            water_vision: Some(player.get_water_vision()),
            is_under_water: player.is_under_water(),
            is_spectator: player.is_spectator(),
        })
    }
}

/// `ShaderManager`'s compiled pipelines, compiled when first asked for
/// (`RenderSystem.getCompiledPipeline`).
pub struct Pipelines {
    configs: Configs,
    builder: PipelineBuilder,
    compiled: Vec<(Identifier, Rc<dyn CompiledRenderPipeline>)>,
}

impl Pipelines {
    pub fn new(device: &dyn GpuDevice) -> Pipelines {
        let configs = ShaderManager::load_configs(&VanillaPackResources::vanilla_jar_root())
            .expect("the vanilla pack's shaders");
        let info = device.get_device_info();
        Pipelines {
            configs,
            builder: PipelineBuilder::new(
                info.is_z_zero_to_one,
                info.features.shader_draw_parameters,
                info.hints_and_workarounds.is_explicit_depth_required,
            ),
            compiled: Vec::new(),
        }
    }

    /// Panics for a pipeline that doesn't compile, which Java logs before it crashes.
    pub fn get(
        &mut self,
        device: &dyn GpuDevice,
        pipeline: &RenderPipeline,
    ) -> Rc<dyn CompiledRenderPipeline> {
        if let Some((_, compiled)) = self
            .compiled
            .iter()
            .find(|(location, _)| location == pipeline.get_location())
        {
            return Rc::clone(compiled);
        }
        let compiled: Rc<dyn CompiledRenderPipeline> = self
            .builder
            .compile_pipeline(pipeline, &self.configs, device)
            .unwrap_or_else(|| panic!("{} didn't compile", pipeline.get_location()));
        self.compiled
            .push((pipeline.get_location().clone(), Rc::clone(&compiled)));
        compiled
    }
}

pub struct GameRenderer {
    device: &'static dyn GpuDevice,
    main_render_target: MainTarget,
    main_camera: Camera,
    camera_render_state: CameraRenderState,
    lightmap: Lightmap,
    lightmap_render_state: LightmapRenderState,
    lightmap_needs_update: bool,
    block_light_flicker: f32,
    random_source: Box<dyn RandomSource>,
    fog_renderer: FogRenderer,
    global_settings_uniform: GlobalSettingsUniform,
    level_projection_matrix_buffer: ProjectionMatrixBuffer,
    level_renderer: LevelRenderer,
    pipelines: Pipelines,
    block_atlas: TextureAtlas,
    models: Arc<BlockStateModelSet>,
    fluid_models: Arc<FluidStateModelSet>,
    block_colors: Arc<BlockColors>,
    sampler_cache: SamplerCache,
    quad_indices: AutoStorageIndexBuffer,
    /// The level's environment attributes: Java's `level.environmentAttributes()`, which the
    /// port's level doesn't hold yet.
    environment_attributes: Option<EnvironmentAttributeSystem>,
    /// `FogRenderer.FOG_ENVIRONMENTS`' open air, the one the port's camera is ever in.
    atmospheric_fog: AtmosphericFogEnvironment,
    /// `AtlasManager`'s GUI atlas (`AtlasIds.GUI`, `Sheets.GUI_SHEET`).
    gui_atlas: TextureAtlas,
    texture_manager: TextureManager,
    gui_render_state: GuiRenderState,
    gui_renderer: GuiRenderer,
    ui_lightmap: UiLightmap,
    /// `Minecraft.font`, from the resource reload's `FontManager`.
    font: Rc<Font>,
    lighting: Lighting,
    /// The level's cardinal lighting the lighting's level entry was last updated for
    /// (`setLevel`'s `updateLevel`).
    lighting_level_type: Option<cardinal_lighting::Type>,
    overlay_texture: OverlayTexture,
    hud_projection: Projection,
    hud3d_projection_matrix_buffer: ProjectionMatrixBuffer,
    /// Java's `LevelRenderer.entityRenderDispatcher`.
    entity_render_dispatcher: EntityRenderDispatcher,
    feature_render_dispatcher: FeatureRenderDispatcher,
    hand_and_screen_submit_node_storage: SubmitNodeStorage,
    /// `RenderBuffers.stagedVertexBuffer()`.
    staged_vertex_buffer: StagedVertexBuffer,
    /// `levelRenderState.playerRenderState`.
    player_render_state: PlayerRenderState,
    /// `minecraft.gameMode.getPlayerMode() == SPECTATOR`, as of the frame's extract.
    player_is_spectator: bool,
}

impl GameRenderer {
    /// The renderer of a window `width` by `height`, with the vanilla pack's block atlas
    /// (`options.mipmapLevels()` of 4, filtering that isn't anisotropic) and block models.
    pub fn new(device: &'static dyn GpuDevice, width: i32, height: i32) -> GameRenderer {
        let mut pipelines = Pipelines::new(device);
        let vanilla = VanillaPackResources::create_client_pack(
            &VanillaPackResources::vanilla_jar_root(),
            std::path::Path::new("no-asset-index"),
        );
        let layers = vanilla
            .resource_layers()
            .iter()
            .map(|layer| layer.clone() as Rc<dyn PackResources>)
            .collect();
        let manager = Rc::new(MultiPackResourceManager::new(
            PackType::CLIENT_RESOURCES,
            layers,
        ));
        // The resource reload's colormaps, which the biomes' tints read.
        GrassColorReloadListener::apply(GrassColorReloadListener::prepare(&*manager));
        FoliageColorReloadListener::apply(FoliageColorReloadListener::prepare(&*manager));
        DryFoliageColorReloadListener::apply(DryFoliageColorReloadListener::prepare(&*manager));
        let mut block_atlas = TextureAtlas::new(
            device,
            Identifier::with_default_namespace(TextureAtlas::LOCATION_BLOCKS),
        );
        let blocks = SpriteLoader::new(
            block_atlas.location().clone(),
            block_atlas.max_supported_texture_size(),
        )
        .load_and_stitch(
            &*manager,
            &Identifier::with_default_namespace("blocks"),
            4,
            0,
            &[],
        )
        .unwrap_or_else(|error| panic!("couldn't stitch {}", error.sprite));
        let items = SpriteLoader::new(
            Identifier::with_default_namespace(TextureAtlas::LOCATION_ITEMS),
            block_atlas.max_supported_texture_size(),
        )
        .load_and_stitch(
            &*manager,
            &Identifier::with_default_namespace("items"),
            0,
            0,
            &[],
        )
        .unwrap_or_else(|error| panic!("couldn't stitch {}", error.sprite));
        let materials = MaterialBaker::new(&blocks, &items);
        let baked = ModelManager::bake_block_state_models(&*manager, &materials);
        let models = Arc::new(BlockStateModelSet::new(
            baked.block_state_models,
            baked.missing_models.block,
        ));
        let fluid_models = Arc::new(FluidStateModelSet::new(
            FluidStateModelSet::bake(&materials),
            baked.missing_models.fluid,
        ));
        let sampler_cache = SamplerCache::new(device);
        let animation_pipelines = AnimationPipelines {
            blit: pipelines.get(device, &RenderPipelines::get().ANIMATE_SPRITE_BLIT),
            interpolate: pipelines.get(device, &RenderPipelines::get().ANIMATE_SPRITE_INTERPOLATE),
        };
        block_atlas.upload(device, &sampler_cache, &animation_pipelines, blocks);
        // The GUI's atlas: no mipmaps, and its sprites' `gui` sections.
        let mut gui_atlas = TextureAtlas::new(device, GameRenderer::gui_sheet());
        let gui = SpriteLoader::new(
            gui_atlas.location().clone(),
            gui_atlas.max_supported_texture_size(),
        )
        .load_and_stitch(
            &*manager,
            &Identifier::with_default_namespace("gui"),
            0,
            0,
            &[AdditionalMetadataType::Gui],
        )
        .unwrap_or_else(|error| panic!("couldn't stitch {}", error.sprite));
        gui_atlas.upload(device, &sampler_cache, &animation_pipelines, gui);
        // FontManager's reload, with no font options (Unicode and Japanese variants off).
        let font = Rc::new(Font::new(
            Rc::new(FontManager::load(Some(device), &*manager, &[])),
            false,
        ));
        let mut texture_manager =
            TextureManager::with_resource_manager(Rc::clone(&manager) as Rc<dyn ResourceManager>);
        texture_manager.register(
            gui_atlas.location().clone(),
            gui_atlas
                .get_texture_view()
                .expect("the atlas is uploaded")
                .clone(),
            Rc::clone(gui_atlas.get_sampler().expect("the atlas is uploaded")),
        );
        let overlay_texture = OverlayTexture::new(device, &sampler_cache);

        GameRenderer {
            device,
            main_render_target: MainTarget::new(device, width, height),
            main_camera: Camera::new(),
            camera_render_state: CameraRenderState::default(),
            lightmap: Lightmap::new(device),
            lightmap_render_state: LightmapRenderState::default(),
            lightmap_needs_update: false,
            block_light_flicker: 0.0,
            random_source: random_source::create(),
            fog_renderer: FogRenderer::new(device),
            global_settings_uniform: GlobalSettingsUniform::new(device),
            level_projection_matrix_buffer: ProjectionMatrixBuffer::new(device, "level"),
            level_renderer: LevelRenderer::new(),
            pipelines,
            block_atlas,
            models,
            fluid_models,
            block_colors: Arc::new(BlockColors::create_default()),
            sampler_cache,
            quad_indices: RenderSystem::sequential_buffer(PrimitiveTopology::QUADS),
            environment_attributes: None,
            atmospheric_fog: AtmosphericFogEnvironment::default(),
            gui_atlas,
            texture_manager,
            gui_render_state: GuiRenderState::new(),
            gui_renderer: GuiRenderer::new(device),
            ui_lightmap: UiLightmap::new(device),
            font,
            lighting: Lighting::new(device),
            lighting_level_type: None,
            overlay_texture,
            hud_projection: Projection::new(),
            hud3d_projection_matrix_buffer: ProjectionMatrixBuffer::new(device, "hud3d"),
            entity_render_dispatcher: EntityRenderDispatcher::new(),
            feature_render_dispatcher: FeatureRenderDispatcher::new(),
            hand_and_screen_submit_node_storage: SubmitNodeStorage::new(),
            staged_vertex_buffer: StagedVertexBuffer::new(),
            player_render_state: PlayerRenderState::default(),
            player_is_spectator: false,
        }
    }

    pub fn font(&self) -> &Rc<Font> {
        &self.font
    }

    /// `Sheets.GUI_SHEET`, the GUI atlas's texture.
    pub fn gui_sheet() -> Identifier {
        Identifier::with_default_namespace("textures/atlas/gui.png")
    }

    /// The window's GUI scale and size for the `guiScale` option (`Minecraft.resizeGui`): the
    /// Unicode font isn't forced.
    pub fn gui_window(&self, gui_scale_option: i32) -> GuiWindow {
        let (width, height) = (
            self.main_render_target.width,
            self.main_render_target.height,
        );
        GuiWindow {
            width,
            height,
            gui_scale: Window::calculate_scale(width, height, gui_scale_option, false),
        }
    }

    /// `Gui.extractRenderState`'s start: the GUI's render state reset, and a `GuiGraphicsExtractor`
    /// over it for `extract`, at the mouse's scaled position.
    pub fn extract_gui(
        &mut self,
        gui_scale_option: i32,
        text_background_color: i32,
        mouse_x: i32,
        mouse_y: i32,
        extract: impl FnOnce(&mut GuiGraphicsExtractor),
    ) {
        let window = self.gui_window(gui_scale_option);
        let (gui_width, gui_height) =
            Window::gui_scaled_size(window.width, window.height, window.gui_scale);
        self.gui_render_state.reset();
        let mut graphics = GuiGraphicsExtractor::new(
            gui_width,
            gui_height,
            text_background_color,
            &self.gui_atlas,
            &self.texture_manager,
            &mut self.gui_render_state,
            mouse_x,
            mouse_y,
        );
        extract(&mut graphics);
    }

    /// `render`'s GUI, after the level: `guiRenderer.render()` over the main target.
    pub fn render_gui(&mut self, gui_scale_option: i32) {
        let window = self.gui_window(gui_scale_option);
        let glyph_textures = GlyphTextures {
            sampler: Rc::clone(self.sampler_cache.get_clamp_to_edge(FilterMode::NEAREST)),
            lightmap: self.ui_lightmap.get_texture_view().clone(),
            lightmap_sampler: Rc::clone(self.sampler_cache.get_clamp_to_edge(FilterMode::LINEAR)),
        };
        self.gui_renderer.render(
            self.device,
            &mut self.gui_render_state,
            &mut self.pipelines,
            &self.main_render_target,
            window,
            &glyph_textures,
        );
    }

    /// `extractOptions`: what the frame reads of the options, and whether the HUD is hidden.
    pub fn extract_options(options: &Options, hud_hidden: bool) -> RenderOptions {
        RenderOptions {
            render_distance: options.get_effective_render_distance(),
            fov: options.fov,
            fov_effect_scale: options.fov_effect_scale as f32,
            cloud_range: options.cloud_range,
            ambient_occlusion: options.ambient_occlusion,
            cutout_leaves: options.cutout_leaves,
            chunk_section_fade_in_time: options.chunk_section_fade_in_time,
            gamma: options.gamma as f32,
            glint_strength: options.glint_strength,
            use_rgss: options.texture_filtering_rgss,
            hide_gui: hud_hidden,
        }
    }

    pub fn device(&self) -> &'static dyn GpuDevice {
        self.device
    }

    /// `tick()`, each client tick: the lightmap's flicker, the camera's eye, attributes and field
    /// of view; then the attributes' tick cache goes, which `ClientLevel.tick` does at its end.
    pub fn tick(
        &mut self,
        has_level: bool,
        player: Option<&LocalPlayer>,
        level: Option<&dyn LayerLevel>,
        options: &RenderOptions,
    ) {
        // LightmapRenderStateExtractor.tick.
        let random = &mut self.random_source;
        self.block_light_flicker += (random.next_float() - random.next_float())
            * random.next_float()
            * random.next_float()
            * 0.1;
        self.block_light_flicker *= 0.9;
        self.lightmap_needs_update = true;
        self.main_camera.set_level(has_level);
        self.main_camera
            .tick(player, level, options.fov_effect_scale);
        if let Some(environment_attributes) = &self.environment_attributes {
            environment_attributes.invalidate_tick_cache();
        }
    }

    fn animation_pipelines(&mut self) -> AnimationPipelines {
        AnimationPipelines {
            blit: self
                .pipelines
                .get(self.device, &RenderPipelines::get().ANIMATE_SPRITE_BLIT),
            interpolate: self.pipelines.get(
                self.device,
                &RenderPipelines::get().ANIMATE_SPRITE_INTERPOLATE,
            ),
        }
    }

    /// `TextureManager.tick()`, before a frame's ticks: the block atlas's animations, the only
    /// ticking texture the port has.
    pub fn tick_textures(&mut self) {
        let pipelines = self.animation_pipelines();
        self.block_atlas
            .cycle_animation_frames(self.device, &self.sampler_cache, &pipelines);
    }

    /// The block atlas's animations started over, at a scenario's start (TESTING.md).
    pub fn restart_texture_animations(&mut self) {
        let pipelines = self.animation_pipelines();
        self.block_atlas
            .restart_animations(self.device, &self.sampler_cache, &pipelines);
    }

    /// Whether the level's environment attributes are yet to be made.
    pub fn needs_environment_attributes(&self) -> bool {
        self.environment_attributes.is_none()
    }

    pub fn set_environment_attributes(
        &mut self,
        environment_attributes: EnvironmentAttributeSystem,
    ) {
        self.environment_attributes = Some(environment_attributes);
    }

    /// What the frame reads of the environment at the camera (`FogRenderer.setupFog`,
    /// `SkyRenderer.extractRenderState`, `LightmapRenderStateExtractor.extract`), at the partial
    /// tick, the eye at `eye_y` over a level whose horizon is at `horizon_height`.
    #[allow(clippy::too_many_arguments)]
    fn extract_environment(
        &mut self,
        level: &dyn LayerLevel,
        options: &RenderOptions,
        partial_ticks: f32,
        delta_ticks: f32,
        eye_y: f64,
        min_y: i32,
        is_flat: bool,
        fog_type: FogType,
        camera_entity: &CameraEntityState,
    ) -> FrameEnvironment {
        // ClientLevelData.getHorizonHeight.
        let horizon_height = if is_flat { f64::from(min_y) } else { 63.0 };
        let system = self
            .environment_attributes
            .as_ref()
            .expect("the level's environment attributes");
        let camera = &mut self.main_camera;
        let mut environment = FrameEnvironment::default();

        // FogRenderer.setupFog: the colour and distances of the environment the camera is in
        // (FOG_ENVIRONMENTS' first that applies; blindness and darkness need mob effects, which
        // aren't ported), then the render distance's.
        let fog_type = if fog_type == FogType::NONE {
            FogType::ATMOSPHERIC
        } else {
            fog_type
        };
        let fog = &mut environment.fog;
        let render_distance_in_blocks = (options.render_distance * 16) as f32;
        let color = match fog_type {
            FogType::LAVA => LavaFogEnvironment::get_base_color(),
            FogType::POWDER_SNOW => PowderedSnowFogEnvironment::get_base_color(),
            FogType::WATER => {
                WaterFogEnvironment::get_base_color(camera, system, level, partial_ticks)
            }
            _ => self.atmospheric_fog.get_base_color(
                camera,
                system,
                level,
                options.render_distance,
                partial_ticks,
            ),
        };
        fog.color = FogRenderer::compute_fog_color(
            &color,
            camera.position().y,
            min_y,
            is_flat,
            fog_type,
            camera_entity.water_vision,
        );
        match fog_type {
            FogType::LAVA => LavaFogEnvironment::setup_fog(
                fog,
                render_distance_in_blocks,
                camera_entity.is_spectator,
            ),
            FogType::POWDER_SNOW => PowderedSnowFogEnvironment::setup_fog(
                fog,
                render_distance_in_blocks,
                camera_entity.is_spectator,
            ),
            FogType::WATER => WaterFogEnvironment::setup_fog(
                fog,
                camera,
                system,
                level,
                partial_ticks,
                camera_entity.water_vision,
            ),
            _ => self.atmospheric_fog.setup_fog(
                fog,
                camera,
                system,
                level,
                render_distance_in_blocks,
                delta_ticks,
                partial_ticks,
                options.cloud_range,
            ),
        }
        let render_distance_fog_span = Mth::clamp_f32(render_distance_in_blocks / 10.0, 4.0, 64.0);
        fog.render_distance_start = render_distance_in_blocks - render_distance_fog_span;
        fog.render_distance_end = render_distance_in_blocks;

        // SkyRenderer.extractRenderState, for the overworld's sky.
        let probe = camera.attribute_probe_mut();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sky = &mut environment.sky;
        sky.sun_angle = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SUN_ANGLE,
            partial_ticks,
        ) * degrees;
        sky.moon_angle = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::MOON_ANGLE,
            partial_ticks,
        ) * degrees;
        sky.star_angle = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::STAR_ANGLE,
            partial_ticks,
        ) * degrees;
        sky.rain_brightness = 1.0 - level.rain_level();
        sky.star_brightness = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::STAR_BRIGHTNESS,
            partial_ticks,
        );
        sky.sunrise_and_sunset_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SUNRISE_SUNSET_COLOR,
            partial_ticks,
        );
        sky.sky_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SKY_COLOR,
            partial_ticks,
        );
        // shouldRenderDarkDisc.
        sky.should_render_dark_disc = eye_y - horizon_height < 0.0 && !camera_entity.is_under_water;

        // LightmapRenderStateExtractor.extract, without the end's flashes, the darkness effect or
        // night vision (mob effects aren't ported); the flicker and the gamma are the frame's.
        let lightmap = &mut environment.lightmap;
        lightmap.block_light_tint = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::BLOCK_LIGHT_TINT,
            partial_ticks,
        );
        lightmap.sky_factor = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SKY_LIGHT_FACTOR,
            partial_ticks,
        );
        lightmap.sky_light_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::SKY_LIGHT_COLOR,
            partial_ticks,
        );
        lightmap.ambient_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::AMBIENT_LIGHT_COLOR,
            partial_ticks,
        );
        lightmap.darkness_effect_scale = 0.0;
        lightmap.night_vision_effect_intensity = 0.0;
        lightmap.night_vision_color = probe.get_value(
            system,
            level,
            &EnvironmentAttributes::NIGHT_VISION_COLOR,
            partial_ticks,
        );
        lightmap.boss_overlay_world_darkening = 0.0;
        environment
    }

    /// `setLevel`: the camera's level, and a new one's sections from scratch.
    pub fn set_level(&mut self, has_level: bool) {
        self.main_camera.set_level(has_level);
        if !has_level {
            self.main_camera.reset();
        }
        self.level_renderer.reset_level_render_data();
    }

    pub fn main_camera(&self) -> &Camera {
        &self.main_camera
    }

    pub fn main_camera_mut(&mut self) -> &mut Camera {
        &mut self.main_camera
    }

    pub fn camera_options(&self, options: &RenderOptions) -> CameraOptions {
        CameraOptions {
            render_distance: options.render_distance,
            cloud_range: options.cloud_range,
            fov: options.fov,
            fov_effect_scale: options.fov_effect_scale,
            window_width: self.main_render_target.width,
            window_height: self.main_render_target.height,
            z_zero_to_one: self.device.get_device_info().is_z_zero_to_one,
        }
    }

    /// `update(DeltaTracker)`: the camera at the player's eye, in the level.
    pub fn update(
        &mut self,
        player: Option<&LocalPlayer>,
        level: Option<&dyn BlockGetter>,
        partial_ticks: f32,
        options: &RenderOptions,
    ) {
        let camera_options = self.camera_options(options);
        self.main_camera
            .update(player, level, partial_ticks, &camera_options);
    }

    /// A frame of the level, at `partial_ticks` (`delta_ticks` since the last): `extract`, then
    /// `render`. `lighting` is the level's light and tints, and `environment` what its
    /// environment attributes read of it; the player's eye is at `eye_y`, and its hands are drawn
    /// in front of the level.
    #[allow(clippy::too_many_arguments)]
    pub fn render_level_frame(
        &mut self,
        level: &ClientLevel,
        lighting: &dyn BlockAndTintGetter,
        environment: &dyn LayerLevel,
        player: Option<&LocalPlayer>,
        options: &RenderOptions,
        partial_ticks: f32,
        delta_ticks: f32,
        game_time: i64,
        eye_y: f64,
        camera_entity: &CameraEntityState,
    ) {
        let device = self.device;
        let z_zero_to_one = device.get_device_info().is_z_zero_to_one;
        // extractCamera.
        self.main_camera.extract_render_state(
            &mut self.camera_render_state,
            partial_ticks,
            z_zero_to_one,
        );
        self.camera_render_state.fog_type =
            self.main_camera.get_fluid_in_camera(level, options.fov);
        let environment = self.extract_environment(
            environment,
            options,
            partial_ticks,
            delta_ticks,
            eye_y,
            level.get_min_y(),
            level.is_flat(),
            self.camera_render_state.fog_type,
            camera_entity,
        );
        self.camera_render_state.fog_data = environment.fog.clone();
        let level_options = LevelRenderOptions {
            render_distance: options.render_distance,
            ambient_occlusion: options.ambient_occlusion,
            cutout_leaves: options.cutout_leaves,
            chunk_section_fade_in_time: options.chunk_section_fade_in_time,
            fov: options.fov,
        };
        self.level_renderer.extract_sections(
            level,
            &self.camera_render_state,
            &level_options,
            &self.models,
            &self.fluid_models,
            &self.block_colors,
        );
        self.lightmap_render_state = environment.lightmap.clone();
        self.lightmap_render_state.needs_update = self.lightmap_needs_update;
        self.lightmap_render_state.block_factor = self.block_light_flicker + 1.4;
        self.lightmap_render_state.brightness = options.gamma;
        self.lightmap_needs_update = false;
        self.extract_player_state(level, player, partial_ticks);
        // setLevel's Lighting.updateLevel, for the level's cardinal lighting.
        let cardinal_light_type =
            DimensionType::cardinal_light_type_of(level.dimension_type_registration());
        if self.lighting_level_type != Some(cardinal_light_type) {
            self.lighting.update_level(device, cardinal_light_type);
            self.lighting_level_type = Some(cardinal_light_type);
        }

        // render.
        device
            .create_command_encoder()
            .clear_color_and_depth_textures(
                &**self.main_render_target.get_color_texture().expect("color"),
                &Vector4f::from_scalar(0.0),
                &**self.main_render_target.get_depth_texture().expect("depth"),
                0.0,
            );
        self.global_settings_uniform.update(
            device,
            self.main_render_target.width,
            self.main_render_target.height,
            options.glint_strength,
            game_time,
            partial_ticks,
            0,
            &self.camera_render_state.pos,
            options.use_rgss,
        );
        let lightmap_pipeline = self.pipelines.get(device, &RenderPipelines::get().LIGHTMAP);
        self.lightmap
            .render(device, &lightmap_pipeline, &self.lightmap_render_state);
        self.render_level(level, lighting, &level_options, &environment.sky, options);
        self.fog_renderer_end_frame();
        device.create_command_encoder().clear_depth_texture(
            &**self.main_render_target.get_depth_texture().expect("depth"),
            0.0,
        );
    }

    fn fog_renderer_end_frame(&mut self) {
        RenderSystem::set_shader_fog(self.fog_renderer.get_buffer(FogMode::NONE));
    }

    /// `renderLevel`: the projection (no bobbing or warp), the fog, the level renderer, then the
    /// 3D HUD.
    fn render_level(
        &mut self,
        level: &ClientLevel,
        lighting: &dyn BlockAndTintGetter,
        level_options: &LevelRenderOptions,
        sky_state: &SkyRenderState,
        options: &RenderOptions,
    ) {
        let device = self.device;
        let mut projection_matrix =
            Matrix4f::from_matrix4f(&self.camera_render_state.projection_matrix);
        projection_matrix.mul(&Matrix4f::new());
        RenderSystem::set_projection_matrix(
            self.level_projection_matrix_buffer
                .get_buffer_matrix(device, &projection_matrix),
        );
        self.fog_renderer
            .update_buffer(device, &self.camera_render_state.fog_data);
        let terrain_pipelines = TerrainPipelines {
            multidraw: ChunkSectionLayer::VALUES
                .map(|layer| self.pipelines.get(device, layer.pipeline(true))),
        };
        let sky_pipelines = SkyPipelines {
            sky: self.pipelines.get(device, &RenderPipelines::get().SKY),
        };
        let lightmap_sampler = Rc::clone(self.sampler_cache.get_clamp_to_edge(FilterMode::LINEAR));
        let mut resources = LevelResources {
            terrain_pipelines: &terrain_pipelines,
            sky_pipelines: &sky_pipelines,
            block_atlas: self
                .block_atlas
                .get_texture_view()
                .expect("the atlas is uploaded"),
            lightmap: self.lightmap.get_texture_view(),
            lightmap_sampler: &lightmap_sampler,
            quad_indices: &mut self.quad_indices,
        };
        // The main pass's lighting (LevelRenderer.addMainPass), which nothing the level draws
        // reads yet, and the hands do.
        self.lighting.setup_for(Entry::LEVEL);
        self.level_renderer.render(
            device,
            &self.main_render_target,
            &mut resources,
            &self.fog_renderer,
            &self.camera_render_state,
            sky_state,
            level_options,
            level,
            lighting,
        );
        self.render_3d_hud(options);
    }

    /// `render3dHud`: the HUD's projection over a cleared depth, the hands, and the fog off after.
    /// The screen effects and the debug crosshair aren't ported.
    fn render_3d_hud(&mut self, options: &RenderOptions) {
        let device = self.device;
        self.hud_projection.setup_perspective(
            0.05,
            self.camera_render_state.depth_far,
            self.camera_render_state.hud_fov,
            self.main_render_target.width as f32,
            self.main_render_target.height as f32,
        );
        RenderSystem::set_projection_matrix(
            self.hud3d_projection_matrix_buffer
                .get_buffer(device, &mut self.hud_projection),
        );
        device.create_command_encoder().clear_depth_texture(
            &**self.main_render_target.get_depth_texture().expect("depth"),
            0.0,
        );
        self.render_item_in_hand(options);
        RenderSystem::set_shader_fog(self.fog_renderer.get_buffer(FogMode::NONE));
    }

    /// `OptionsRenderState.damageTiltStrength`: the option's default (the options screen isn't
    /// ported).
    const DAMAGE_TILT_STRENGTH: f64 = 1.0;

    /// `bobHurt`: the view tilted on dying and towards a hit while the camera's entity is hurt.
    fn bob_hurt(camera_state: &CameraRenderState, pose_stack: &mut PoseStack) {
        let entity = &camera_state.entity_render_state;
        if !entity.is_living {
            return;
        }
        let mut hurt = entity.hurt_time;
        if entity.is_dead_or_dying {
            let duration = Math::min_f32(entity.death_time, 20.0);
            pose_stack.rotate_degrees(Axis::ZP, 40.0 - 8000.0 / (duration + 200.0));
        }

        if hurt < 0.0 {
            return;
        }

        hurt /= entity.hurt_duration as f32;
        hurt = Mth::sin(f64::from(
            hurt * hurt * hurt * hurt * std::f64::consts::PI as f32,
        ));
        let rr = entity.hurt_dir;
        pose_stack.rotate_degrees(Axis::YP, -rr);
        let tilt_amount = (f64::from(-hurt) * 14.0 * GameRenderer::DAMAGE_TILT_STRENGTH) as f32;
        pose_stack.rotate_degrees(Axis::ZP, tilt_amount);
        pose_stack.rotate_degrees(Axis::YP, rr);
    }

    /// `renderItemInHand`: in the first person, unless the HUD is hidden or the player is a
    /// spectator, the hands submitted in the camera's frame and drawn in their own pass over the
    /// level's color, with the HUD's depth.
    fn render_item_in_hand(&mut self, options: &RenderOptions) {
        let player_state = &self.player_render_state;
        if !player_state.has_player
            || !self.camera_render_state.is_first_person
            || self.camera_render_state.entity_render_state.is_sleeping
            || options.hide_gui
            || self.player_is_spectator
        {
            return;
        }
        let device = self.device;
        let camera_state = &self.camera_render_state;
        let mut pose_stack = PoseStack::new();
        pose_stack.push_pose();
        let mut inverted = Matrix4f::new();
        camera_state.view_rotation_matrix.invert_into(&mut inverted);
        pose_stack.mul_pose(&mut inverted);
        RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.mul(&camera_state.view_rotation_matrix);
        });
        GameRenderer::bob_hurt(camera_state, &mut pose_stack);
        FirstPersonHandsAndItemsRenderer::submit_hands_with_items(
            camera_state.camera_entity_partial_ticks,
            &mut pose_stack,
            &mut self.hand_and_screen_submit_node_storage,
            player_state,
            &mut self.entity_render_dispatcher,
        );

        let mut context = FeatureFrameContext {
            device,
            texture_manager: &mut self.texture_manager,
            sampler_cache: &self.sampler_cache,
            overlay_texture: self.overlay_texture.get_texture_view(),
            lightmap_texture: self.lightmap.get_texture_view(),
            pipelines: &mut self.pipelines,
            staged_vertex_buffer: &mut self.staged_vertex_buffer,
            quad_indices: &mut self.quad_indices,
            // ProjectionType.PERSPECTIVE's.
            vertex_sorting: VertexSorting::distance_to_origin,
            use_improved_transparency: false,
            last_transform: None,
        };
        let frame = self
            .feature_render_dispatcher
            .prepare_frame(&mut context, &mut self.hand_and_screen_submit_node_storage);
        {
            let encoder = device.create_command_encoder();
            let mut render_pass = encoder.create_render_pass_with(
                "Item in hand",
                self.main_render_target
                    .get_color_texture_view()
                    .expect("the main target has color"),
                None,
                self.main_render_target.get_depth_texture_view(),
                None,
            );
            RenderSystem::bind_default_uniforms(&mut *render_pass);
            self.feature_render_dispatcher.render_all_features(
                &mut context,
                &mut *render_pass,
                &frame,
            );
        }
        self.feature_render_dispatcher
            .close_frame(&mut context, frame);

        RenderSystem::with_model_view_stack(|stack| {
            stack.pop_matrix();
        });
        pose_stack.pop_pose();
    }

    /// `LevelExtractor.extractPlayerState`, as far as the hands read it: the player's avatar (its
    /// renderer's `extractRenderState`) and its hands. The screen effects' parts aren't ported.
    fn extract_player_state(
        &mut self,
        level: &ClientLevel,
        player: Option<&LocalPlayer>,
        partial_ticks: f32,
    ) {
        let state = &mut self.player_render_state;
        state.reset();
        let Some(player) = player else {
            return;
        };
        state.has_player = true;
        let entity = player.entity();
        self.player_is_spectator = player.is_spectator();
        // Camera.extractRenderState's part about its entity, the player: the hurt and death
        // that tilt the view. The player's hurt direction and death time aren't ported (0).
        let partial = self.camera_render_state.camera_entity_partial_ticks;
        let living = player.living_entity();
        let camera_entity = &mut self.camera_render_state.entity_render_state;
        camera_entity.is_living = true;
        camera_entity.is_player = true;
        camera_entity.is_sleeping = false;
        camera_entity.is_dead_or_dying = living.is_dead_or_dying();
        camera_entity.hurt_dir = 0.0;
        camera_entity.hurt_time = living.hurt_time as f32 - partial;
        camera_entity.death_time = partial;
        camera_entity.hurt_duration = living.hurt_duration;
        let avatar_entity = AvatarEntity {
            uuid: entity.get_uuid(),
            light_probe_position: entity.get_eye_position_partial(partial_ticks),
            x_rot: entity.get_x_rot_partial(partial_ticks),
            main_arm: player.get_main_arm(),
            model_customisation: player.model_customisation,
            // Entity.FLAG_INVISIBLE and FLAG_ONFIRE: a fire-immune entity's own fire isn't
            // ported, and a client's fire is the server's flag.
            is_invisible: entity.get_shared_flag(5),
            is_on_fire: entity.get_shared_flag(0),
            level,
        };
        let mut avatar_render_state = AvatarRenderState::default();
        self.entity_render_dispatcher
            .get_renderer(&avatar_render_state)
            .extract_render_state(&avatar_entity, &mut avatar_render_state);
        state.avatar_render_state = Some(avatar_render_state);
        player.first_person_hands_and_items().extract_render_state(
            entity.get_x_rot_partial(partial_ticks),
            entity.get_y_rot_partial(partial_ticks),
            Mth::lerp_f32(partial_ticks, player.x_bob_o, player.x_bob),
            Mth::lerp_f32(partial_ticks, player.y_bob_o, player.y_bob),
            &mut state.first_person_hands_and_items,
        );
        state.is_under_water = player.is_under_water();
        state.eye_position_y = entity.get_eye_position_partial(partial_ticks).y;
    }

    /// `Screenshot.takeScreenshot` of the main target.
    pub fn take_screenshot(&self) -> NativeImage {
        Screenshot::take_screenshot(self.device, &self.main_render_target, 1)
    }

    pub fn main_render_target(&self) -> &MainTarget {
        &self.main_render_target
    }

    pub fn level_renderer(&self) -> &LevelRenderer {
        &self.level_renderer
    }
}
