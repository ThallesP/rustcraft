//! com/mojang/renderpearl/api/commands/RenderPass.java
//!
//! Drawing with one pipeline at a time, its uniforms bound by name, and vertex and index buffers.
//! Not ported yet: debug groups, timestamps, push constants, and the multi-draws, indirect draws
//! and `drawMultipleIndexed`, which come with the pipelines that use them.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use std::rc::Rc;

pub const MAX_VERTEX_BUFFERS: usize = 16;
pub const INDIRECT_DRAW_SIZE: i32 = 16;
pub const INDIRECT_INDEXED_DRAW_SIZE: i32 = 20;

/// A pass over a set of attachments, which ends when it's dropped (Java's `close()`).
pub trait RenderPass {
    fn set_pipeline(&mut self, pipeline: &Rc<dyn CompiledRenderPipeline>);

    /// `setUniform(String, GpuTextureView, GpuSampler)`: both or neither.
    fn set_uniform_sampler(
        &mut self,
        name: &str,
        texture_view: Option<&GpuTextureView>,
        sampler: Option<&Rc<dyn GpuSampler>>,
    );

    /// `setUniform(String, GpuBuffer)`: the whole buffer.
    fn set_uniform_buffer(&mut self, name: &str, value: &Rc<dyn GpuBuffer>);

    /// `setUniform(String, GpuBufferSlice)`.
    fn set_uniform_slice(&mut self, name: &str, value: &GpuBufferSlice);

    fn enable_scissor(&mut self, x: i32, y: i32, width: i32, height: i32);

    fn disable_scissor(&mut self);

    fn set_vertex_buffer(&mut self, slot: i32, vertex_buffer: Option<&GpuBufferSlice>);

    fn set_index_buffer(&mut self, index_buffer: &Rc<dyn GpuBuffer>, index_type: IndexType);

    fn draw_indexed(
        &mut self,
        index_count: i32,
        instance_count: i32,
        first_index: i32,
        vertex_offset: i32,
        first_instance: i32,
    );

    fn draw(
        &mut self,
        vertex_count: i32,
        instance_count: i32,
        first_vertex: i32,
        first_instance: i32,
    );
}

/// `RenderPass.RenderArea`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RenderArea {
    pub fn fills_texture(&self, texture: &GpuTextureView) -> bool {
        self.x == 0
            && self.y == 0
            && self.width == texture.get_width(0)
            && self.height == texture.get_height(0)
    }
}
