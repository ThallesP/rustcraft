//! com/mojang/renderpearl/api/commands/RenderPass.java
//!
//! Drawing with one pipeline at a time, its uniforms bound by name, and vertex and index buffers,
//! directly or from indirect commands. Debug groups are labels a GL debugger shows and change no
//! pixel, so `pushDebugGroup` and `popDebugGroup` do nothing yet. `drawMultipleIndexed` is generic
//! in its uniform argument, which a trait object's method can't be, so it's a function of the
//! module over any pass (`draw_multiple_indexed`), doing what `FrontendRenderPass`'s does through
//! the pass's methods; its draws' uniform uploader is the pass itself, `setUniform` of a buffer.
//! Not ported yet: timestamps, push constants, the direct multi-draws, which come with the
//! pipelines that use them, and `drawIndirect`.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
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

    /// `drawCount` indexed draws whose `VkDrawIndexedIndirectCommand`s are in `commands`.
    fn draw_indexed_indirect(&mut self, commands: &GpuBufferSlice, draw_count: i32);

    fn push_debug_group(&mut self, _label: &str) {}

    fn pop_debug_group(&mut self) {}
}

/// `RenderPass.Draw`: one draw of `draw_multiple_indexed`, its vertex buffer at `slot`, the
/// default index buffer and type where it has none, and what it sets from the draws' uniform
/// argument first.
pub struct Draw<T: ?Sized> {
    pub slot: i32,
    pub vertex_buffer: Rc<dyn GpuBuffer>,
    pub index_buffer: Option<Rc<dyn GpuBuffer>>,
    pub index_type: Option<IndexType>,
    pub first_index: i32,
    pub index_count: i32,
    pub base_vertex: i32,
    #[allow(clippy::type_complexity)]
    pub uniform_uploader_consumer: Option<Box<dyn Fn(&T, &mut dyn RenderPass)>>,
}

/// `drawMultipleIndexed`, as `FrontendRenderPass` draws it: each draw sets its uniforms, binds its
/// vertex buffer whole and its index buffer when it isn't the last draw's, and draws one instance
/// from its first index and base vertex. The dynamic uniforms Java names are the ones the draws
/// set, which only its validation reads.
pub fn draw_multiple_indexed<T: ?Sized>(
    render_pass: &mut dyn RenderPass,
    draws: &[Draw<T>],
    default_index_buffer: Option<&Rc<dyn GpuBuffer>>,
    default_index_type: Option<IndexType>,
    uniform_argument: &T,
) {
    let mut last_index_buffer: Option<(&Rc<dyn GpuBuffer>, IndexType)> = None;
    for draw in draws {
        if let Some(uniform_uploader_consumer) = &draw.uniform_uploader_consumer {
            uniform_uploader_consumer(uniform_argument, render_pass);
        }

        let index_type = draw
            .index_type
            .or(default_index_type)
            .expect("an index type");
        let index_buffer = draw
            .index_buffer
            .as_ref()
            .or(default_index_buffer)
            .expect("an index buffer");
        render_pass.set_vertex_buffer(draw.slot, Some(&gpu_buffer::slice(&draw.vertex_buffer)));
        let same_index_buffer = last_index_buffer.is_some_and(|(last, last_type)| {
            Rc::ptr_eq(last, index_buffer) && last_type == index_type
        });
        if !same_index_buffer {
            render_pass.set_index_buffer(index_buffer, index_type);
            last_index_buffer = Some((index_buffer, index_type));
        }
        render_pass.draw_indexed(draw.index_count, 1, draw.first_index, draw.base_vertex, 0);
    }
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
