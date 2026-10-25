//! net/minecraft/client/renderer/chunk/ChunkSectionsToRender.java
//!
//! A frame's terrain draws, ready for the render passes, and the terrain's transform. Its two
//! subclasses are a `Kind`: `DrawIndirect`, each layer's draw groups (sections that share a vertex
//! buffer, drawn by indirect multi-draws) with the sections' positions as instance data, which a
//! device with indirect draws and base instances takes (`LevelRenderer.multiDrawIndirectAvailable`,
//! llvmpipe); and `DrawSeparate`, a draw for each section's layer with its section's uniform
//! block, which the others take (Apple's OpenGL 4.1). The wireframe pipelines and the
//! order-independent transparency's passes (`renderOit`) aren't ported.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::AutoStorageIndexBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::{
    self, Draw, INDIRECT_INDEXED_DRAW_SIZE, RenderPass,
};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;

/// `ChunkSectionsToRender.GpuMultiDrawIndexedIndirect`.
pub struct GpuMultiDrawIndexedIndirect {
    pub vertex_buffer: GpuBufferSlice,
    pub index_buffer: Option<GpuBufferSlice>,
    pub index_type: Option<IndexType>,
    pub indirect_command_buffer: GpuBufferSlice,
    pub draw_count: i32,
}

pub struct ChunkSectionsToRender {
    max_indices_required: i32,
    terrain_transform_ubo: GpuBufferSlice,
    kind: Kind,
}

/// The subclass and its fields, each layer's by the layer's ordinal.
enum Kind {
    /// `DrawIndirect`.
    DrawIndirect {
        draw_groups_per_layer: [Vec<GpuMultiDrawIndexedIndirect>; 3],
        chunk_section_infos: GpuBufferSlice,
    },
    /// `DrawSeparate`: each draw sets its section's block of `chunk_section_infos`.
    DrawSeparate {
        draws_per_layer: [Vec<Draw<[GpuBufferSlice]>>; 3],
        chunk_section_infos: Vec<GpuBufferSlice>,
    },
}

/// What the terrain draws with: each layer's pipelines, compiled, for multi-draws and for draws
/// one at a time (`ChunkSectionLayer.pipeline`).
pub struct TerrainPipelines {
    pub multidraw: [Rc<dyn CompiledRenderPipeline>; 3],
    pub separate: [Rc<dyn CompiledRenderPipeline>; 3],
}

impl ChunkSectionsToRender {
    /// `new DrawIndirect(…)`.
    pub fn draw_indirect(
        terrain_transform_ubo: GpuBufferSlice,
        draw_groups_per_layer: [Vec<GpuMultiDrawIndexedIndirect>; 3],
        max_indices_required: i32,
        chunk_section_infos: GpuBufferSlice,
    ) -> ChunkSectionsToRender {
        ChunkSectionsToRender {
            max_indices_required,
            terrain_transform_ubo,
            kind: Kind::DrawIndirect {
                draw_groups_per_layer,
                chunk_section_infos,
            },
        }
    }

    /// `new DrawSeparate(…)`.
    pub fn draw_separate(
        terrain_transform_ubo: GpuBufferSlice,
        draws_per_layer: [Vec<Draw<[GpuBufferSlice]>>; 3],
        max_indices_required: i32,
        chunk_section_infos: Vec<GpuBufferSlice>,
    ) -> ChunkSectionsToRender {
        ChunkSectionsToRender {
            max_indices_required,
            terrain_transform_ubo,
            kind: Kind::DrawSeparate {
                draws_per_layer,
                chunk_section_infos,
            },
        }
    }

    /// `renderGroup` for a group's layers, with the block atlas, its sampler and the lightmap.
    #[allow(clippy::too_many_arguments)]
    pub fn render_group(
        &self,
        layers: &[ChunkSectionLayer],
        render_pass: &mut dyn RenderPass,
        device: &dyn GpuDevice,
        quad_indices: &mut AutoStorageIndexBuffer,
        pipelines: &TerrainPipelines,
        sampler: &Rc<dyn GpuSampler>,
        atlas: &GpuTextureView,
        lightmap: &GpuTextureView,
        lightmap_sampler: &Rc<dyn GpuSampler>,
    ) {
        let default_index_buffer = if self.max_indices_required == 0 {
            None
        } else {
            Some(quad_indices.get_buffer(device, 0))
        };
        let default_index_type = quad_indices.index_type();
        render_pass.set_uniform_slice("TerrainUniform", &self.terrain_transform_ubo);
        render_pass.set_uniform_sampler("Sampler0", Some(atlas), Some(sampler));
        render_pass.set_uniform_sampler("Sampler2", Some(lightmap), Some(lightmap_sampler));
        for &layer in layers {
            render_pass.push_debug_group(&format!("Terrain layer: {}", layer.label()));
            self.render(
                layer,
                render_pass,
                device,
                default_index_buffer.as_ref(),
                default_index_type,
                pipelines,
            );
            render_pass.pop_debug_group();
        }
    }

    fn render(
        &self,
        layer: ChunkSectionLayer,
        render_pass: &mut dyn RenderPass,
        device: &dyn GpuDevice,
        default_index_buffer: Option<&Rc<dyn GpuBuffer>>,
        default_index_type: IndexType,
        pipelines: &TerrainPipelines,
    ) {
        match &self.kind {
            Kind::DrawIndirect {
                draw_groups_per_layer,
                chunk_section_infos,
            } => {
                render_pass.set_pipeline(&pipelines.multidraw[layer as usize]);
                let max_draw_indirect_draw_count =
                    device.get_device_info().limits.max_draw_indirect_draw_count;
                let draw_groups = &draw_groups_per_layer[layer as usize];
                if !draw_groups.is_empty() {
                    render_pass.set_vertex_buffer(1, Some(chunk_section_infos));
                }

                for indirect_draw in draw_groups {
                    if indirect_draw.draw_count > 0 {
                        render_pass.set_vertex_buffer(0, Some(&indirect_draw.vertex_buffer));
                        let index_type = indirect_draw.index_type.unwrap_or(default_index_type);
                        let index_buffer = match &indirect_draw.index_buffer {
                            Some(index_buffer) => &index_buffer.buffer,
                            None => default_index_buffer.expect("the shared quad indices"),
                        };
                        render_pass.set_index_buffer(index_buffer, index_type);
                        let buffer = &indirect_draw.indirect_command_buffer.buffer;
                        let mut start_offset = indirect_draw.indirect_command_buffer.offset;
                        let mut remaining_draw_count = indirect_draw.draw_count;
                        while remaining_draw_count > 0 {
                            let pass_draw_count =
                                remaining_draw_count.min(max_draw_indirect_draw_count);
                            let length =
                                i64::from(pass_draw_count) * i64::from(INDIRECT_INDEXED_DRAW_SIZE);
                            let pass_slice = GpuBufferSlice {
                                buffer: Rc::clone(buffer),
                                offset: start_offset,
                                length,
                            };
                            render_pass.draw_indexed_indirect(&pass_slice, pass_draw_count);
                            remaining_draw_count -= pass_draw_count;
                            start_offset += length;
                        }
                    }
                }
            }
            Kind::DrawSeparate {
                draws_per_layer,
                chunk_section_infos,
            } => {
                render_pass.set_pipeline(&pipelines.separate[layer as usize]);
                let draws = &draws_per_layer[layer as usize];
                if !draws.is_empty() {
                    render_pass::draw_multiple_indexed(
                        render_pass,
                        draws,
                        default_index_buffer,
                        default_index_buffer.map(|_| default_index_type),
                        chunk_section_infos.as_slice(),
                    );
                }
            }
        }
    }
}
