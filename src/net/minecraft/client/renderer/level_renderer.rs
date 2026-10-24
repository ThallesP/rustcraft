//! net/minecraft/client/renderer/LevelRenderer.java
//!
//! The level's frame: the main target cleared to the fog's colour, the sky, then the terrain,
//! solid and cut out, then translucent, drawn from the visible sections' meshes; after drawing,
//! the dirty visible sections compile, the translucent ones the camera moved around are sorted
//! again, they upload, and the occlusion graph moves on. The parts of `LevelExtractor` that pick
//! the visible sections and the ones to compile are here too (`extract_sections`), since the
//! port's level renderer owns its sections. Java's frame graph
//! (`FrameGraphBuilder`) only orders the clear, sky and main passes and makes the targets of the
//! improved transparency and the entity outlines, so the passes run in its order here.
//!
//! Not ported yet: entities, block entities and their features (`FeatureRenderDispatcher`), the
//! block outline and breaking, transient blocks, particles, gizmos, clouds, weather and the world
//! border, the improved transparency (OIT) and the entity outline's post chain, `DrawSeparate`
//! (terrain without multi-draw), wireframe, the chunk fade (`chunkSectionFadeInTime`, which the
//! oracle sets to 0), and the multi-draw's ring buffers (`DynamicGpuData`), whose blocks each
//! frame makes buffers of its own for.
//! The sections' light and tints come through `lighting`, the level itself (`RenderRegionCache`).

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::pipeline::render_target::RenderTarget;
use crate::com::mojang::blaze3d::systems::render_system::{AutoStorageIndexBuffer, RenderSystem};
use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::block::block_state_model_set::BlockStateModelSet;
use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::chunk_sections_to_render::{
    ChunkSectionsToRender, GpuMultiDrawIndexedIndirect, TerrainPipelines,
};
use crate::net::minecraft::client::renderer::chunk::render_region_cache::RenderRegionCache;
use crate::net::minecraft::client::renderer::chunk::section_compiler::SectionCompiler;
use crate::net::minecraft::client::renderer::chunk::section_render_dispatcher::{
    RenderSection, SectionRenderDispatcher,
};
use crate::net::minecraft::client::renderer::chunk::translucency_point_of_view::TranslucencyPointOfView;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::{DynamicGpuData, TerrainTransform};
use crate::net::minecraft::client::renderer::fog::fog_renderer::{FogMode, FogRenderer};
use crate::net::minecraft::client::renderer::rendertype::render_types::RenderTypes;
use crate::net::minecraft::client::renderer::section_occlusion_graph::SectionOcclusionGraph;
use crate::net::minecraft::client::renderer::sky_renderer::{
    SkyPipelines, SkyRenderState, SkyRenderer, SkyResources,
};
use crate::net::minecraft::client::renderer::state::level::block_outline_render_state::BlockOutlineRenderState;
use crate::net::minecraft::client::renderer::state::level::camera_render_state::CameraRenderState;
use crate::net::minecraft::client::renderer::submit_node_storage::SubmitNodeStorage;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::renderer::view_area::ViewArea;
use crate::net::minecraft::client::section_update_tracker::SectionUpdateTracker;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::dimension::dimension_type::Skybox;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::material::fog_type::FogType;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector4f::Vector4f;

/// What the level renderer reads of the options.
#[derive(Clone, Copy, Debug)]
pub struct LevelRenderOptions {
    pub render_distance: i32,
    pub ambient_occlusion: bool,
    pub cutout_leaves: bool,
    /// `chunkSectionFadeInTime`, in seconds.
    pub chunk_section_fade_in_time: f64,
    pub fov: i32,
}

/// What draws the level: the device's pipelines for the terrain and the sky, the block atlas, the
/// lightmap and the shared quad indices.
pub struct LevelResources<'a> {
    pub terrain_pipelines: &'a TerrainPipelines,
    pub sky_pipelines: &'a SkyPipelines,
    pub celestials_atlas: &'a TextureAtlas,
    pub block_atlas: &'a GpuTextureView,
    pub lightmap: &'a GpuTextureView,
    pub lightmap_sampler: &'a Rc<dyn GpuSampler>,
    pub quad_indices: &'a mut AutoStorageIndexBuffer,
}

/// The frame's features as the level's passes draw them: Java's `FeatureRenderDispatcher
/// .PreparedFrame` of the level's `SubmitNodeStorage`, which the game renderer holds with what
/// draws them. Each call gets the shared quad indices the level renderer holds.
pub trait LevelFeatures {
    /// `prepareFrame`, once the model view has the camera's rotation.
    fn prepare(&mut self, quad_indices: &mut AutoStorageIndexBuffer);

    fn execute_solid(
        &mut self,
        quad_indices: &mut AutoStorageIndexBuffer,
        render_pass: &mut dyn RenderPass,
    );

    fn execute_translucent(
        &mut self,
        quad_indices: &mut AutoStorageIndexBuffer,
        render_pass: &mut dyn RenderPass,
    );

    fn execute_translucent_after_terrain(
        &mut self,
        quad_indices: &mut AutoStorageIndexBuffer,
        render_pass: &mut dyn RenderPass,
    );

    /// `PreparedFrame.close()`.
    fn close(&mut self, quad_indices: &mut AutoStorageIndexBuffer);
}

/// A draw of the multi-draw's command buffer: `DynamicGpuData.IndexedDraw`.
#[derive(Clone, Copy, Debug)]
struct IndexedDraw {
    index_count: i32,
    instance_count: i32,
    first_index: i32,
    base_vertex: i32,
    base_instance: i32,
}

/// `LevelRenderer.ChunkDrawGroup`: draws sharing a vertex buffer and index buffer.
struct ChunkDrawGroup {
    vertex_buffer: Rc<dyn GpuBuffer>,
    index_buffer: Option<Rc<dyn GpuBuffer>>,
    index_type: Option<IndexType>,
    draws: Vec<IndexedDraw>,
}

pub struct LevelRenderer {
    sky_renderer: Option<SkyRenderer>,
    section_occlusion_graph: SectionOcclusionGraph,
    visible_sections: Vec<usize>,
    nearby_visible_sections: Vec<usize>,
    view_area: Option<ViewArea>,
    section_render_dispatcher: Option<SectionRenderDispatcher>,
    section_update_tracker: Option<SectionUpdateTracker>,
    /// The sections `extract_sections` found dirty, to compile after this frame's draws.
    sections_to_compile: Vec<usize>,
    chunk_layer_sampler: Option<Rc<dyn GpuSampler>>,
    prev_cam_rot_x: f64,
    prev_cam_rot_y: f64,
    last_view_distance: i32,
    last_translucent_sort_block_pos: Option<BlockPos>,
    translucency_resort_iteration_index: usize,
}

impl Default for LevelRenderer {
    fn default() -> LevelRenderer {
        LevelRenderer::new()
    }
}

impl LevelRenderer {
    /// `Math.powExact(2, 3)`.
    pub const OIT_COEFFICIENT_COUNT: i32 = 8;
    pub const OIT_TRANSMITTANCE_TARGET_COUNT: i32 = LevelRenderer::OIT_COEFFICIENT_COUNT / 4;

    pub fn new() -> LevelRenderer {
        LevelRenderer {
            sky_renderer: None,
            section_occlusion_graph: SectionOcclusionGraph::new(),
            visible_sections: Vec::new(),
            nearby_visible_sections: Vec::new(),
            view_area: None,
            section_render_dispatcher: None,
            section_update_tracker: None,
            sections_to_compile: Vec::new(),
            chunk_layer_sampler: None,
            prev_cam_rot_x: f64::MIN,
            prev_cam_rot_y: f64::MIN,
            last_view_distance: -1,
            last_translucent_sort_block_pos: None,
            translucency_resort_iteration_index: 0,
        }
    }

    /// `LevelExtractor.allChanged` and `LevelRenderer.invalidateCompiledGeometry`: new sections
    /// around the camera, all to compile, with a compiler for the options.
    pub fn all_changed(
        &mut self,
        level: &ClientLevel,
        options: &LevelRenderOptions,
        camera_section_pos: SectionPos,
        models: &Arc<BlockStateModelSet>,
        fluid_models: &Arc<FluidStateModelSet>,
        block_colors: &Arc<BlockColors>,
    ) {
        self.last_view_distance = options.render_distance;
        // The new tracker has every section dirty, so what the level marked before is moot.
        level.level_extractor().listen();
        let mut tracker = SectionUpdateTracker::new(level, options.render_distance);
        tracker.reposition_camera(camera_section_pos);
        self.section_update_tracker = Some(tracker);

        let section_compiler = SectionCompiler::new(
            options.ambient_occlusion,
            options.cutout_leaves,
            Arc::clone(models),
            Arc::clone(fluid_models),
            Arc::clone(block_colors),
        );
        match &mut self.section_render_dispatcher {
            Some(dispatcher) => dispatcher.set_compiler(section_compiler),
            None => {
                self.section_render_dispatcher =
                    Some(SectionRenderDispatcher::new(section_compiler))
            }
        }
        LeavesBlock::set_cutout_leaves(options.cutout_leaves);
        let mut view_area = ViewArea::new(
            level.get_min_y(),
            level.get_max_y(),
            level.get_min_section_y(),
            level.get_max_section_y(),
            options.render_distance,
        );
        self.section_occlusion_graph
            .wait_and_reset(Some(&view_area));
        self.clear_visible_sections();
        view_area.reposition_camera(camera_section_pos);
        self.view_area = Some(view_area);
    }

    /// `resize`: the visible sections are worked out again (the entity outline's target isn't
    /// ported).
    pub fn resize(&mut self, _width: i32, _height: i32) {
        self.section_occlusion_graph.invalidate();
    }

    pub fn reset_level_render_data(&mut self) {
        self.view_area = None;
        self.section_render_dispatcher = None;
        self.section_update_tracker = None;
        self.section_occlusion_graph.wait_and_reset(None);
        self.clear_visible_sections();
    }

    pub fn clear_visible_sections(&mut self) {
        self.visible_sections.clear();
        self.nearby_visible_sections.clear();
    }

    /// `LevelExtractor.extract`'s sections: the camera moves the sections, the frustum picks the
    /// visible ones when the graph or the camera's rotation changed, and the visible dirty ones
    /// whose neighbours the level has are marked to compile after the frame.
    pub fn extract_sections(
        &mut self,
        level: &ClientLevel,
        camera_state: &CameraRenderState,
        options: &LevelRenderOptions,
        models: &Arc<BlockStateModelSet>,
        fluid_models: &Arc<FluidStateModelSet>,
        block_colors: &Arc<BlockColors>,
    ) {
        let camera_section_pos = SectionPos::of_block_pos(&camera_state.block_pos);
        if options.render_distance != self.last_view_distance || self.view_area.is_none() {
            self.all_changed(
                level,
                options,
                camera_section_pos,
                models,
                fluid_models,
                block_colors,
            );
        }

        // What the level's changes marked since the last frame, which Java's tracker got as they
        // came.
        let dirty_sections = level.level_extractor().take_dirty_sections();
        if let Some(tracker) = &mut self.section_update_tracker {
            for (x, y, z, player_changed) in dirty_sections {
                tracker.set_dirty(x, y, z, player_changed);
            }
            tracker.reposition_camera(camera_section_pos);
        }

        let cam_rot_x = (camera_state.x_rot / 2.0).floor() as f64;
        let cam_rot_y = (camera_state.y_rot / 2.0).floor() as f64;
        if self.section_occlusion_graph.consume_frustum_update()
            || cam_rot_x != self.prev_cam_rot_x
            || cam_rot_y != self.prev_cam_rot_y
        {
            self.apply_frustum(camera_state);
            self.prev_cam_rot_x = cam_rot_x;
            self.prev_cam_rot_y = cam_rot_y;
        }

        let (Some(tracker), Some(view_area)) = (&mut self.section_update_tracker, &self.view_area)
        else {
            return;
        };
        self.sections_to_compile.clear();
        for &section in &self.visible_sections {
            let render_section = view_area.section(section);
            let section_node = render_section.section_node();
            let is_uncompiled = render_section.get_section_mesh().is_uncompiled();
            let has_all_neighbors = tracker.has_all_neighbors(level, section_node);
            if let Some(dirty_state) = tracker.get_dirty_state(section_node)
                && dirty_state.is_dirty()
                && (!is_uncompiled || has_all_neighbors)
            {
                self.sections_to_compile.push(section);
                dirty_state.set_not_dirty();
            }
        }
    }

    fn apply_frustum(&mut self, camera_state: &CameraRenderState) {
        self.clear_visible_sections();
        self.section_occlusion_graph.add_sections_in_frustum(
            &camera_state.cull_frustum,
            &mut self.visible_sections,
            &mut self.nearby_visible_sections,
        );
    }

    /// `render`: the clear, sky and main passes, then the sections' compiles and uploads and the
    /// occlusion graph's update. The frame's features (`submitFeatures`, which the game renderer
    /// submits) are prepared once the model view has the camera's rotation, and drawn in the main
    /// pass.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &dyn GpuDevice,
        main_target: &RenderTarget,
        resources: &mut LevelResources<'_>,
        fog_renderer: &FogRenderer,
        camera_state: &CameraRenderState,
        sky_state: &SkyRenderState,
        options: &LevelRenderOptions,
        level: &ClientLevel,
        lighting: &dyn BlockAndTintGetter,
        features: &mut dyn LevelFeatures,
    ) {
        let terrain_fog = fog_renderer.get_buffer(FogMode::WORLD);
        let fog_color = camera_state.fog_data.color;
        self.reposition_camera(camera_state);
        RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.mul(&camera_state.view_rotation_matrix);
        });
        features.prepare(resources.quad_indices);

        let color = main_target
            .get_color_texture_view()
            .expect("the main target has color");
        let depth = main_target
            .get_depth_texture_view()
            .expect("the main target has depth");

        // The frame graph's "clear" pass.
        device
            .create_command_encoder()
            .clear_color_and_depth_textures(
                &**main_target.get_color_texture().expect("color"),
                &Vector4f::from_xyzw(fog_color.x, fog_color.y, fog_color.z, 0.0),
                &**main_target.get_depth_texture().expect("depth"),
                0.0,
            );
        // addSkyPass: none in powder snow or lava, or for a dimension without a sky (mob effects
        // that hide it aren't ported).
        let fog_type = camera_state.fog_type;
        if fog_type != FogType::POWDER_SNOW && fog_type != FogType::LAVA {
            let celestials_atlas = resources.celestials_atlas;
            let sky_renderer = self
                .sky_renderer
                .get_or_insert_with(|| SkyRenderer::new(device, celestials_atlas));
            if sky_state.skybox != Skybox::NONE {
                let mut sky_resources = SkyResources {
                    pipelines: resources.sky_pipelines,
                    celestials: celestials_atlas
                        .get_texture_view()
                        .expect("the atlas is uploaded"),
                    celestials_sampler: celestials_atlas
                        .get_sampler()
                        .expect("the atlas is uploaded"),
                    quad_indices: resources.quad_indices,
                };
                sky_renderer.render(
                    device,
                    &mut sky_resources,
                    &terrain_fog,
                    color,
                    depth,
                    sky_state,
                );
            }
        }
        // "main".
        let terrain_matrix = Matrix4f::from_matrix4f(&camera_state.view_rotation_matrix);
        let chunk_sections_to_render =
            self.prepare_chunk_renders_indirect(device, &terrain_matrix, true, resources, options);
        self.add_main_pass(
            device,
            &terrain_fog,
            &chunk_sections_to_render,
            color,
            depth,
            resources,
            features,
        );

        RenderSystem::with_model_view_stack(|stack| {
            stack.pop_matrix();
        });
        features.close(resources.quad_indices);
        self.compile_sections(level, lighting);
        self.schedule_translucent_section_resort(&camera_state.pos);
        self.upload_terrain_buffers(device);
        if let Some(view_area) = &mut self.view_area {
            let (loaded_chunks, empty_sections) =
                LevelRenderer::chunk_loading_state(level, view_area);
            self.section_occlusion_graph.update(
                camera_state,
                options.fov,
                view_area,
                loaded_chunks,
                empty_sections,
            );
        }
    }

    fn reposition_camera(&mut self, camera_state: &CameraRenderState) {
        let camera_pos = camera_state.pos;
        let camera_section_pos = SectionPos::of_block_pos(&camera_state.block_pos);
        if let Some(view_area) = &mut self.view_area
            && view_area.reposition_camera(camera_section_pos)
        {
            self.section_occlusion_graph.invalidate();
        }

        if let Some(dispatcher) = &mut self.section_render_dispatcher {
            dispatcher.set_camera_position(camera_pos);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_main_pass(
        &mut self,
        device: &dyn GpuDevice,
        terrain_fog: &GpuBufferSlice,
        chunk_sections_to_render: &ChunkSectionsToRender,
        color: &GpuTextureView,
        depth: &GpuTextureView,
        resources: &mut LevelResources<'_>,
        features: &mut dyn LevelFeatures,
    ) {
        RenderSystem::set_shader_fog(terrain_fog.clone());
        let sampler = self
            .chunk_layer_sampler
            .get_or_insert_with(|| {
                device.create_sampler(
                    AddressMode::CLAMP_TO_EDGE,
                    AddressMode::CLAMP_TO_EDGE,
                    FilterMode::LINEAR,
                    FilterMode::LINEAR,
                    1,
                    None,
                )
            })
            .clone();
        // prepareTranslucents: no clouds, border or weather yet, then the shared indices' growth.
        resources
            .quad_indices
            .resize_to_requested_index_count(device);
        let encoder = device.create_command_encoder();
        let mut render_pass =
            encoder.create_render_pass_with("Main", color, None, Some(depth), None);
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        chunk_sections_to_render.render_group(
            &[ChunkSectionLayer::SOLID, ChunkSectionLayer::CUTOUT],
            &mut *render_pass,
            device,
            resources.quad_indices,
            resources.terrain_pipelines,
            &sampler,
            resources.block_atlas,
            resources.lightmap,
            resources.lightmap_sampler,
        );
        // executeSolid's features, then executeClassicTransparency: the translucent features, the
        // translucent terrain, then the features after it.
        features.execute_solid(resources.quad_indices, &mut *render_pass);
        features.execute_translucent(resources.quad_indices, &mut *render_pass);
        chunk_sections_to_render.render_group(
            &[ChunkSectionLayer::TRANSLUCENT],
            &mut *render_pass,
            device,
            resources.quad_indices,
            resources.terrain_pipelines,
            &sampler,
            resources.block_atlas,
            resources.lightmap,
            resources.lightmap_sampler,
        );
        features.execute_translucent_after_terrain(resources.quad_indices, &mut *render_pass);
    }

    /// `submitBlockOutline`: the outline of the block the player looks at, from the camera, in
    /// the window's line width (the high contrast option's extra outline and colour, and improved
    /// transparency's render type, aren't ported: the options are off). `submitHitOutline` without
    /// the debug shapes.
    pub fn submit_block_outline(
        submit_node_collector: &mut SubmitNodeStorage,
        state: &BlockOutlineRenderState,
        camera_pos: &Vec3,
        appropriate_line_width: f32,
    ) {
        let mut pose_stack = PoseStack::new();
        let pos = state.pos;
        pose_stack.push_pose();
        pose_stack.translate_f64(
            f64::from(pos.get_x()) - camera_pos.x,
            f64::from(pos.get_y()) - camera_pos.y,
            f64::from(pos.get_z()) - camera_pos.z,
        );
        assert!(
            !state.high_contrast,
            "the high contrast block outline isn't ported"
        );
        let outline_color = ARGB::black(102);
        submit_node_collector.submit_shape_outline(
            &pose_stack,
            state.shape.clone(),
            &RenderTypes::lines_translucent(),
            outline_color,
            appropriate_line_width,
            state.is_translucent,
        );
        pose_stack.pop_pose();
    }

    /// `extractSectionDrawGroups`: each visible section's layers' draws, grouped by the buffers
    /// they read (translucent ones in order, the rest by the buffers' identity), and the
    /// sections' positions; returns the most indices a draw of the shared quad indices needs.
    fn extract_section_draw_groups(
        &self,
        respect_translucent_order: bool,
        section_infos: &mut Vec<(i32, i32, i32, f32)>,
        draw_groups: &mut [Vec<ChunkDrawGroup>; 3],
        options: &LevelRenderOptions,
    ) -> i32 {
        let mut largest_index_count = 0;
        let (Some(dispatcher), Some(view_area)) =
            (&self.section_render_dispatcher, &self.view_area)
        else {
            return largest_index_count;
        };
        let fade_duration = (options.chunk_section_fade_in_time * 1000.0) as i64;
        let now = Util::get_millis();
        let mut last_transparent_group: Option<(usize, usize)> = None;
        // Java hashes the buffers' identities into an int, `combinedHash`; their pair is the
        // same key.
        // The draw group cache, by the buffers' identities.
        let mut draw_group_cache: Vec<((usize, usize), usize, usize)> = Vec::new();
        for &section in &self.visible_sections {
            let render_section = view_area.section(section);
            let section_mesh = render_section.get_section_mesh();
            let render_offset = render_section.get_render_origin();
            let mut section_info_data_index = -1;
            for layer in ChunkSectionLayer::VALUES {
                let (Some(draw), Some(slice)) = (
                    section_mesh.get_section_draw(layer),
                    dispatcher.get_render_section_slice(section_mesh, layer),
                ) else {
                    continue;
                };
                if draw.has_custom_index_buffer && slice.index_buffer.is_none() {
                    continue;
                }
                if section_info_data_index == -1 {
                    section_info_data_index = section_infos.len() as i32;
                    section_infos.push((
                        render_offset.get_x(),
                        render_offset.get_y(),
                        render_offset.get_z(),
                        render_section.get_visibility(now, fade_duration),
                    ));
                }

                let vertex_size = layer.vertex_format().get_vertex_size();
                let identity =
                    |buffer: &Rc<dyn GpuBuffer>| Rc::as_ptr(buffer) as *const () as usize;
                let mut key = (identity(&slice.vertex_buffer), 0);
                let mut first_index = 0;
                let (index_buffer, index_type) = if !draw.has_custom_index_buffer {
                    largest_index_count = largest_index_count.max(draw.index_count);
                    (None, None)
                } else {
                    let index_buffer = slice.index_buffer.clone().expect("checked above");
                    key.1 = identity(&index_buffer);
                    first_index =
                        (slice.index_buffer_offset / i64::from(draw.index_type.bytes())) as i32;
                    (Some(index_buffer), Some(draw.index_type))
                };
                let base_vertex = (slice.vertex_buffer_offset / i64::from(vertex_size)) as i32;
                let layer_groups = &mut draw_groups[layer as usize];
                let group = if layer.translucent() && respect_translucent_order {
                    let same =
                        (last_transparent_group == Some(key)).then(|| layer_groups.len() - 1);
                    last_transparent_group = Some(key);
                    same
                } else {
                    draw_group_cache
                        .iter()
                        .find(|(cached_key, cached_layer, _)| {
                            *cached_key == key && *cached_layer == layer as usize
                        })
                        .map(|(_, _, index)| *index)
                };
                let group = match group {
                    Some(group) => group,
                    None => {
                        layer_groups.push(ChunkDrawGroup {
                            vertex_buffer: Rc::clone(&slice.vertex_buffer),
                            index_buffer,
                            index_type,
                            draws: Vec::new(),
                        });
                        let index = layer_groups.len() - 1;
                        draw_group_cache.push((key, layer as usize, index));
                        index
                    }
                };
                layer_groups[group].draws.push(IndexedDraw {
                    index_count: draw.index_count,
                    instance_count: 1,
                    first_index,
                    base_vertex,
                    base_instance: section_info_data_index,
                });
            }
        }

        largest_index_count
    }

    /// `prepareChunkRendersIndirect`: the draw groups' commands in one buffer, the sections'
    /// positions in another, and the terrain's transform.
    fn prepare_chunk_renders_indirect(
        &self,
        device: &dyn GpuDevice,
        model_view_matrix: &Matrix4f,
        respect_translucent_order: bool,
        resources: &mut LevelResources<'_>,
        options: &LevelRenderOptions,
    ) -> ChunkSectionsToRender {
        let mut draw_groups: [Vec<ChunkDrawGroup>; 3] = Default::default();
        let mut section_infos = Vec::new();
        let texture_atlas_width = resources.block_atlas.get_width(0);
        let texture_atlas_height = resources.block_atlas.get_height(0);
        let largest_index_count = self.extract_section_draw_groups(
            respect_translucent_order,
            &mut section_infos,
            &mut draw_groups,
            options,
        );

        // writeChunkSectionCommands: each group's draws, one after another.
        let mut commands = Vec::new();
        let mut group_offsets: [Vec<(i64, i32)>; 3] = Default::default();
        for layer in ChunkSectionLayer::VALUES {
            let mut sorted_draw_groups: Vec<&ChunkDrawGroup> =
                draw_groups[layer as usize].iter().collect();
            if layer.translucent() && respect_translucent_order {
                sorted_draw_groups.reverse();
            }

            for draw_group in sorted_draw_groups {
                let mut sorted_draws = draw_group.draws.clone();
                if layer.translucent() && respect_translucent_order {
                    sorted_draws.reverse();
                }

                group_offsets[layer as usize]
                    .push((commands.len() as i64, sorted_draws.len() as i32));
                for draw in sorted_draws {
                    for value in [
                        draw.index_count,
                        draw.instance_count,
                        draw.first_index,
                        draw.base_vertex,
                        draw.base_instance,
                    ] {
                        commands.extend_from_slice(&value.to_le_bytes());
                    }
                }
            }
        }
        let command_buffer = (!commands.is_empty()).then(|| {
            device.create_buffer_with_data(
                Some("Chunk Sections Command Buffer"),
                gpu_buffer::USAGE_INDIRECT_PARAMETERS,
                &commands,
            )
        });

        let mut indirect_draws: [Vec<GpuMultiDrawIndexedIndirect>; 3] = Default::default();
        for layer in ChunkSectionLayer::VALUES {
            let mut sorted_draw_groups: Vec<&ChunkDrawGroup> =
                draw_groups[layer as usize].iter().collect();
            if layer.translucent() && respect_translucent_order {
                sorted_draw_groups.reverse();
            }

            for (draw_group, (offset, count)) in sorted_draw_groups
                .into_iter()
                .zip(&group_offsets[layer as usize])
            {
                indirect_draws[layer as usize].push(GpuMultiDrawIndexedIndirect {
                    vertex_buffer: gpu_buffer::slice(&draw_group.vertex_buffer),
                    index_buffer: draw_group.index_buffer.as_ref().map(gpu_buffer::slice),
                    index_type: draw_group.index_type,
                    indirect_command_buffer: gpu_buffer::slice_range(
                        command_buffer.as_ref().expect("a group has draws"),
                        *offset,
                        i64::from(*count) * 20,
                    ),
                    draw_count: *count,
                });
            }
        }

        let terrain_transform = TerrainTransform {
            model_view: Matrix4f::from_matrix4f(model_view_matrix),
            texture_atlas_width,
            texture_atlas_height,
        };
        let terrain_transform_ubo = gpu_buffer::slice(&device.create_buffer_with_data(
            Some("Terrain Transform UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &terrain_transform.write(),
        ));
        // writeChunkSectionsInstanced: each section's position and visibility, std140.
        let mut infos = Vec::new();
        for (x, y, z, visibility) in &section_infos {
            let mut builder = Std140Builder::on_stack(DynamicGpuData::CHUNK_SECTION_UBO_SIZE);
            builder.put_ivec3(*x, *y, *z).put_float(*visibility);
            infos.extend_from_slice(&builder.get());
        }
        if infos.is_empty() {
            infos.resize(DynamicGpuData::CHUNK_SECTION_UBO_SIZE as usize, 0);
        }
        let chunk_section_infos = gpu_buffer::slice(&device.create_buffer_with_data(
            Some("Chunk Sections Instanced"),
            gpu_buffer::USAGE_VERTEX,
            &infos,
        ));
        if largest_index_count != 0 {
            resources
                .quad_indices
                .request_index_count(largest_index_count);
        }

        ChunkSectionsToRender::new(
            terrain_transform_ubo,
            indirect_draws,
            largest_index_count,
            chunk_section_infos,
        )
    }

    /// `compileSections`: the dirty visible sections, compiled now over regions of the level.
    fn compile_sections(&mut self, level: &ClientLevel, lighting: &dyn BlockAndTintGetter) {
        let (Some(dispatcher), Some(view_area)) =
            (&mut self.section_render_dispatcher, &mut self.view_area)
        else {
            return;
        };
        let mut cache = RenderRegionCache::new();
        for section in std::mem::take(&mut self.sections_to_compile) {
            let render_section = view_area.sections.get_mut(section);
            let region = cache.create_region(level, lighting, render_section.section_node());
            if dispatcher.compile_sync(render_section, &region) {
                self.section_occlusion_graph
                    .schedule_propagation_from(section);
            }
        }
    }

    /// `scheduleTranslucentSectionResort`, after the compiles: the nearby visible sections, and the
    /// next few of the others in turn, sorted again when the camera moved to another block (for
    /// the nearby ones, or one the camera is in a slab of) or to another side of them.
    fn schedule_translucent_section_resort(&mut self, camera_pos: &Vec3) {
        let (Some(dispatcher), Some(view_area)) =
            (&mut self.section_render_dispatcher, &self.view_area)
        else {
            return;
        };
        if self.visible_sections.is_empty() {
            return;
        }

        let camera_block_pos = BlockPos::containing_xyz(camera_pos.x, camera_pos.y, camera_pos.z);
        let block_pos_changed = Some(camera_block_pos) != self.last_translucent_sort_block_pos;
        let mut point_of_view = TranslucencyPointOfView::default();
        for &section in &self.nearby_visible_sections {
            LevelRenderer::schedule_resort(
                dispatcher,
                view_area.section(section),
                &mut point_of_view,
                camera_pos,
                block_pos_changed,
                true,
            );
        }

        self.translucency_resort_iteration_index %= self.visible_sections.len();
        let mut resorts_left = (self.visible_sections.len() / 8).max(15);
        while resorts_left > 0 {
            resorts_left -= 1;
            let index = self.translucency_resort_iteration_index % self.visible_sections.len();
            self.translucency_resort_iteration_index += 1;
            LevelRenderer::schedule_resort(
                dispatcher,
                view_area.section(self.visible_sections[index]),
                &mut point_of_view,
                camera_pos,
                block_pos_changed,
                false,
            );
        }

        self.last_translucent_sort_block_pos = Some(camera_block_pos);
    }

    /// `scheduleResort`. A resort runs at once, so none is ever still scheduled; improved
    /// transparency isn't ported, so it's always off.
    fn schedule_resort(
        dispatcher: &mut SectionRenderDispatcher,
        section: &RenderSection,
        point_of_view: &mut TranslucencyPointOfView,
        camera_pos: &Vec3,
        block_pos_changed: bool,
        is_nearby: bool,
    ) {
        point_of_view.set(camera_pos, section.section_node());
        let point_of_view_changed = section
            .get_section_mesh()
            .is_different_point_of_view(point_of_view);
        let resort_because_block_pos_changed =
            block_pos_changed && (point_of_view.is_axis_aligned() || is_nearby);
        if (resort_because_block_pos_changed || point_of_view_changed)
            && section.has_translucent_geometry()
        {
            dispatcher.resort_transparency(section);
        }
    }

    fn upload_terrain_buffers(&mut self, device: &dyn GpuDevice) {
        let (Some(dispatcher), Some(view_area)) =
            (&mut self.section_render_dispatcher, &mut self.view_area)
        else {
            return;
        };
        for section in dispatcher.upload_terrain_buffers_to_gpu(device, &mut view_area.sections) {
            self.section_occlusion_graph
                .schedule_propagation_from(section);
        }
    }

    /// The chunk cache's loaded chunks and empty sections, which Java's cache tracks as they
    /// change: here, the chunks the view area covers that the level has, and their sections of
    /// only air.
    fn chunk_loading_state(
        level: &ClientLevel,
        view_area: &ViewArea,
    ) -> (HashSet<i64>, HashSet<i64>) {
        let mut loaded_chunks = HashSet::new();
        let mut empty_sections = HashSet::new();
        let center = view_area.get_camera_section_pos();
        let radius = view_area.get_view_distance();
        for x in center.x() - radius..=center.x() + radius {
            for z in center.z() - radius..=center.z() + radius {
                let Some(chunk) = level.get_chunk_source().get_chunk(x, z) else {
                    continue;
                };
                loaded_chunks.insert(i64::from(x) & 0xffffffff | (i64::from(z) & 0xffffffff) << 32);
                for (index, section) in chunk.get_sections().iter().enumerate() {
                    if section.has_only_air() {
                        let section_y = level.get_min_section_y() + index as i32;
                        empty_sections.insert(SectionPos::as_long_xyz(x, section_y, z));
                    }
                }
            }
        }
        (loaded_chunks, empty_sections)
    }

    /// Whether every visible section has compiled: Java's `hasRenderedAllSections` (an empty
    /// compile queue), with no visible section left dirty.
    pub fn has_rendered_all_sections(&self) -> bool {
        self.sections_to_compile.is_empty()
            && self
                .section_render_dispatcher
                .as_ref()
                .is_none_or(SectionRenderDispatcher::is_queue_empty)
    }

    pub fn visible_sections(&self) -> &[usize] {
        &self.visible_sections
    }
}
