//! com/mojang/renderpearl/backend/api/RenderPassBackend.java
//!
//! What a backend's render pass does once the frontend's has checked the call. A uniform's value
//! is a Java `Object`, a buffer slice or a texture view with its sampler, so it's `UniformValue`.
//! The same parts as `RenderPass` aren't ported yet.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::BackendRenderPipeline;
use crate::com::mojang::renderpearl::util::texture_view_and_sampler::TextureViewAndSampler;
use std::rc::Rc;

/// A uniform's value: a uniform or texel buffer's slice, or a sampled texture.
#[derive(Clone)]
pub enum UniformValue {
    Buffer(GpuBufferSlice),
    Sampler(TextureViewAndSampler),
}

pub trait RenderPassBackend {
    fn set_pipeline(&mut self, pipeline: &Rc<dyn BackendRenderPipeline>);

    fn set_uniform(&mut self, index: i32, value: Option<&UniformValue>);

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

    fn draw_indexed_indirect(&mut self, commands: &GpuBufferSlice, draw_count: i32);
}
