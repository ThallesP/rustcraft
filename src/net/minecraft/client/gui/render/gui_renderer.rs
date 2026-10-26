//! net/minecraft/client/gui/render/GuiRenderer.java
//!
//! Draws the frame's `GuiRenderState`: each layer's elements sorted by scissor, pipeline and
//! texture (`ELEMENT_SORT_COMPARATOR`, a stable sort), then a draw for each run of elements that
//! share them, in the strata's order, over the main target with the GUI's orthographic
//! projection. Java stages the vertices in a `StagedVertexBuffer`, whose pooled GPU buffers get
//! the draws' vertices one after another (each at a multiple of its vertex size); here each frame
//! makes a buffer of the same bytes, and the draws read it at the same offsets, which changes no
//! pixel.
//!
//! The strata after the blur a screen asks for draw after the depth is cleared and the level and
//! the strata before are blurred (`processBlurEffect`, which the game renderer hands in). The
//! panorama a screen asks for is drawn first (`CubeMap`), its texture the game renderer's texture
//! manager's.
//!
//! Not ported yet: items (`prepareItemElements` and the GUI's item atlas, `GuiItemAtlas`) and
//! pictures in pictures.

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::pipeline::main_target::MainTarget;
use crate::com::mojang::blaze3d::systems::render_system::{AutoStorageIndexBuffer, RenderSystem};
use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
use crate::net::minecraft::client::renderer::cube_map::CubeMap;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use crate::net::minecraft::client::renderer::game_renderer::Pipelines;
use crate::net::minecraft::client::renderer::projection::Projection;
use crate::net::minecraft::client::renderer::projection_matrix_buffer::ProjectionMatrixBuffer;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::state::gui::glyph_render_state::GlyphRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_render_state::{
    GuiRenderState, TraverseRange,
};
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::matrix4f::Matrix4f;

/// `StagedVertexBuffer`'s initial capacity for the GUI.
const VERTEX_BUFFER_CAPACITY: i32 = 786432;

pub struct GuiRenderer {
    /// `StagedVertexBuffer.stagingBuffer`: the builder every frame's vertices are written into.
    staging_buffer: ByteBufferBuilder,
    draws: Vec<Draw>,
    first_draw_index_after_blur: usize,
    gui_projection: Projection,
    gui_projection_matrix_buffer: ProjectionMatrixBuffer,
    quad_indices: AutoStorageIndexBuffer,
    /// `cubeMap`: the title screen's panorama.
    cube_map: CubeMap,
}

/// `GuiRenderer.Draw` with its `StagedVertexBuffer.Draw`: the vertices its elements built.
struct Draw {
    pipeline: Arc<RenderPipeline>,
    texture_setup: TextureSetup,
    scissor_area: Option<ScreenRectangle>,
    format: &'static VertexFormat,
    primitive_topology: PrimitiveTopology,
    vertices: Vec<u8>,
    vertex_count: i32,
    index_count: i32,
    vertex_offset: usize,
}

/// What a text's glyphs bind (`GlyphRenderState.textureSetup()`): the sampler cache's
/// `getClampToEdge(NEAREST)` for the font page, and the game renderer's lightmap, the UI's while
/// the GUI renders, with `getClampToEdge(LINEAR)`.
pub struct GlyphTextures {
    pub sampler: Rc<dyn GpuSampler>,
    pub lightmap: GpuTextureView,
    pub lightmap_sampler: Rc<dyn GpuSampler>,
}

/// The window the GUI draws over: its size and GUI scale (`WindowRenderState`).
#[derive(Clone, Copy, Debug)]
pub struct GuiWindow {
    pub width: i32,
    pub height: i32,
    pub gui_scale: i32,
}

/// `ELEMENT_SORT_COMPARATOR`: by scissor area (none first, then top, bottom, left, right), the
/// pipeline's sort key, then the texture setup's.
pub fn element_sort_comparator(
    a: &dyn GuiElementRenderState,
    b: &dyn GuiElementRenderState,
) -> Ordering {
    let scissor = |area: Option<ScreenRectangle>| {
        area.map(|area| (area.top(), area.bottom(), area.left(), area.right()))
    };
    scissor(a.scissor_area())
        .cmp(&scissor(b.scissor_area()))
        .then_with(|| {
            a.pipeline()
                .get_sort_key()
                .cmp(&b.pipeline().get_sort_key())
        })
        .then_with(|| {
            a.texture_setup()
                .get_sort_key()
                .cmp(&b.texture_setup().get_sort_key())
        })
}

impl GuiRenderer {
    pub fn new(device: &dyn GpuDevice) -> GuiRenderer {
        GuiRenderer {
            staging_buffer: ByteBufferBuilder::new(VERTEX_BUFFER_CAPACITY),
            draws: Vec::new(),
            first_draw_index_after_blur: usize::MAX,
            gui_projection: Projection::new(),
            gui_projection_matrix_buffer: ProjectionMatrixBuffer::new(device, "gui"),
            quad_indices: RenderSystem::sequential_buffer(PrimitiveTopology::QUADS),
            cube_map: CubeMap::new(
                device,
                Identifier::with_default_namespace("textures/gui/title/background/panorama"),
            ),
        }
    }

    /// The panorama's cube, whose texture the game renderer registers
    /// (`registerPanoramaTextures`).
    pub fn cube_map(&self) -> &CubeMap {
        &self.cube_map
    }

    /// `render`: prepare, draw, and the state reset for the next frame.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &dyn GpuDevice,
        render_state: &mut GuiRenderState,
        pipelines: &mut Pipelines,
        main_render_target: &MainTarget,
        window: GuiWindow,
        glyph_textures: &GlyphTextures,
        texture_manager: &TextureManager,
        process_blur_effect: &mut dyn FnMut(&mut Pipelines),
    ) {
        if let Some(panorama_render_state) = render_state.panorama_render_state {
            let pipeline = pipelines.get(device, &RenderPipelines::get().PANORAMA);
            let texture = texture_manager.get_texture(self.cube_map.location());
            self.cube_map.render(
                device,
                &pipeline,
                main_render_target,
                &mut self.quad_indices,
                texture,
                (window.width, window.height),
                10.0,
                panorama_render_state.spin,
            );
        }
        self.prepare(render_state, glyph_textures);
        self.draw(
            device,
            pipelines,
            main_render_target,
            window,
            process_blur_effect,
        );
        self.draws.clear();
        render_state.reset();
        self.first_draw_index_after_blur = usize::MAX;
    }

    fn prepare(&mut self, render_state: &mut GuiRenderState, glyph_textures: &GlyphTextures) {
        // prepareText: each text's glyphs, in its layer.
        render_state.prepare_text(|text, renderable| {
            let texture_view = renderable
                .texture_view()
                .expect("a font page on the device")
                .clone();
            Box::new(GlyphRenderState::new(
                text.pose,
                renderable.clone(),
                text.scissor,
                TextureSetup::single_texture_with_lightmap(
                    texture_view,
                    Rc::clone(&glyph_textures.sampler),
                    glyph_textures.lightmap.clone(),
                    Rc::clone(&glyph_textures.lightmap_sampler),
                ),
            ))
        });
        render_state.sort_elements(element_sort_comparator);
        self.add_elements_to_meshes(render_state, TraverseRange::BEFORE_BLUR);
        self.first_draw_index_after_blur = self.draws.len();
        self.add_elements_to_meshes(render_state, TraverseRange::AFTER_BLUR);
    }

    fn add_elements_to_meshes(&mut self, render_state: &GuiRenderState, range: TraverseRange) {
        // previousDraw, previousPipeline, previousTextureSetup and previousScissorArea: the last
        // draw this range appended, which an element joins when it shares them.
        let mut previous_draw: Option<usize> = None;
        let GuiRenderer {
            staging_buffer,
            draws,
            ..
        } = self;
        render_state.for_each_element(
            |element_state| {
                let pipeline = element_state.pipeline();
                let texture_setup = element_state.texture_setup();
                let scissor_area = element_state.scissor_area();
                let joins = previous_draw.is_some_and(|index| {
                    let previous = &draws[index];
                    Arc::ptr_eq(pipeline, &previous.pipeline)
                        && scissor_area == previous.scissor_area
                        && *texture_setup == previous.texture_setup
                });
                if !joins {
                    draws.push(Draw {
                        pipeline: Arc::clone(pipeline),
                        texture_setup: texture_setup.clone(),
                        scissor_area,
                        format: pipeline
                            .get_vertex_format_binding(0)
                            .expect("a GUI pipeline has a vertex format"),
                        primitive_topology: pipeline.get_primitive_topology(),
                        vertices: Vec::new(),
                        vertex_count: 0,
                        index_count: 0,
                        vertex_offset: 0,
                    });
                    previous_draw = Some(draws.len() - 1);
                }
                let draw = &mut draws[previous_draw.expect("a draw was appended")];
                // getVertexBuilder: the elements' vertices in the staging buffer, which the
                // draw's mesh copies out.
                let mut builder =
                    BufferBuilder::new(staging_buffer, draw.primitive_topology, draw.format);
                element_state.build_vertices(&mut builder);
                if let Some(mesh) = builder.build() {
                    draw.vertices.extend_from_slice(mesh.vertex_buffer());
                    draw.vertex_count += mesh.draw_state().vertex_count;
                    draw.index_count += mesh.draw_state().index_count;
                }
            },
            range,
        );
    }

    fn draw(
        &mut self,
        device: &dyn GpuDevice,
        pipelines: &mut Pipelines,
        main_render_target: &MainTarget,
        window: GuiWindow,
        process_blur_effect: &mut dyn FnMut(&mut Pipelines),
    ) {
        if self.draws.is_empty() {
            return;
        }
        // StagedVertexBuffer.upload: each draw's vertices at the next multiple of its vertex size.
        let mut vertices = Vec::new();
        for draw in &mut self.draws {
            if draw.vertex_count == 0 {
                continue;
            }
            let vertex_size = draw.format.get_vertex_size() as usize;
            draw.vertex_offset = vertices.len().div_ceil(vertex_size) * vertex_size;
            vertices.resize(draw.vertex_offset, 0);
            vertices.extend_from_slice(&draw.vertices);
        }
        if vertices.is_empty() {
            return;
        }
        let vertex_buffer = device.create_buffer_with_data(
            Some("GUI Vertex Buffer - Vertex"),
            gpu_buffer::USAGE_VERTEX,
            &vertices,
        );
        let max_index_count = self
            .draws
            .iter()
            .map(|draw| draw.index_count)
            .max()
            .unwrap_or(0);
        let index_buffer = self.quad_indices.get_buffer(device, max_index_count);

        self.gui_projection.setup_ortho(
            1000.0,
            11000.0,
            window.width as f32 / window.gui_scale as f32,
            window.height as f32 / window.gui_scale as f32,
            true,
        );
        RenderSystem::set_projection_matrix(
            self.gui_projection_matrix_buffer
                .get_buffer(device, &mut self.gui_projection),
        );
        let mut model_view = Matrix4f::new();
        model_view.set_translation(0.0, 0.0, -11000.0);
        let dynamic_transforms = device.create_buffer_with_data(
            Some("Dynamic Transforms UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &Transform::of_model_view(model_view).write(),
        );

        let before_blur = self.first_draw_index_after_blur.min(self.draws.len());
        if self.first_draw_index_after_blur > 0 {
            self.execute_draw_range(
                device,
                "GUI before blur",
                pipelines,
                main_render_target,
                window,
                &dynamic_transforms,
                &vertex_buffer,
                &index_buffer,
                0..before_blur,
            );
        }
        if self.draws.len() > self.first_draw_index_after_blur {
            device.create_command_encoder().clear_depth_texture(
                &**main_render_target
                    .get_depth_texture()
                    .expect("the main target has depth"),
                0.0,
            );
            process_blur_effect(pipelines);
            self.execute_draw_range(
                device,
                "GUI after blur",
                pipelines,
                main_render_target,
                window,
                &dynamic_transforms,
                &vertex_buffer,
                &index_buffer,
                self.first_draw_index_after_blur..self.draws.len(),
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_draw_range(
        &self,
        device: &dyn GpuDevice,
        label: &str,
        pipelines: &mut Pipelines,
        main_render_target: &MainTarget,
        window: GuiWindow,
        dynamic_transforms: &Rc<dyn GpuBuffer>,
        vertex_buffer: &Rc<dyn GpuBuffer>,
        index_buffer: &Rc<dyn GpuBuffer>,
        range: std::ops::Range<usize>,
    ) {
        // The pipelines compile before the pass starts, which only borrows them.
        let compiled: Vec<_> = self.draws[range.clone()]
            .iter()
            .map(|draw| pipelines.get(device, &draw.pipeline))
            .collect();
        let encoder = device.create_command_encoder();
        let mut render_pass = encoder.create_render_pass_with(
            label,
            main_render_target
                .get_color_texture_view()
                .expect("the main target has color"),
            None,
            main_render_target.get_depth_texture_view(),
            None,
        );
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", dynamic_transforms);
        for (draw, compiled) in self.draws[range].iter().zip(&compiled) {
            if draw.vertex_count == 0 {
                continue;
            }
            render_pass.set_pipeline(compiled);
            render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(vertex_buffer)));
            match draw.scissor_area {
                Some(scissor_area) => enable_scissor(&scissor_area, &mut *render_pass, window),
                None => render_pass.disable_scissor(),
            }
            let setup = &draw.texture_setup;
            if setup.texure0.is_some() {
                render_pass.set_uniform_sampler(
                    "Sampler0",
                    setup.texure0.as_ref(),
                    setup.sampler0.as_ref(),
                );
            }
            if setup.texure1.is_some() {
                render_pass.set_uniform_sampler(
                    "Sampler1",
                    setup.texure1.as_ref(),
                    setup.sampler1.as_ref(),
                );
            }
            if setup.texure2.is_some() {
                render_pass.set_uniform_sampler(
                    "Sampler2",
                    setup.texure2.as_ref(),
                    setup.sampler2.as_ref(),
                );
            }
            render_pass.set_index_buffer(index_buffer, self.quad_indices.index_type());
            let base_vertex = (draw.vertex_offset / draw.format.get_vertex_size() as usize) as i32;
            render_pass.draw_indexed(draw.index_count, 1, 0, base_vertex, 0);
        }
    }
}

/// `enableScissor`: the rectangle in the window's pixels, from its bottom.
fn enable_scissor(
    rectangle: &ScreenRectangle,
    render_pass: &mut dyn RenderPass,
    window: GuiWindow,
) {
    let gui_scale = window.gui_scale;
    let left = f64::from(rectangle.left() * gui_scale);
    let top = f64::from(rectangle.top() * gui_scale);
    let right = f64::from((rectangle.right() * gui_scale).min(window.width));
    let bottom = f64::from((rectangle.bottom() * gui_scale).min(window.height));
    render_pass.enable_scissor(
        left as i32,
        window.height - bottom as i32,
        0.max((right - left) as i32),
        0.max((bottom - top) as i32),
    );
}
