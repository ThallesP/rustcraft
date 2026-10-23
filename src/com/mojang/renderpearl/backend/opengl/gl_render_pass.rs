//! com/mojang/renderpearl/backend/opengl/GlRenderPass.java
//!
//! What the frontend's pass hands the backend, kept until a draw needs it: the encoder's
//! `setupDraw` binds only what changed since the last draw (the dirty flags). The same parts as
//! `RenderPassBackend` aren't ported yet.

use crate::com::mojang::blaze3d::systems::scissor_state::ScissorState;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::MAX_VERTEX_BUFFERS;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::BackendRenderPipeline;
use crate::com::mojang::renderpearl::backend::api::render_pass_backend::{
    RenderPassBackend, UniformValue,
};
use crate::com::mojang::renderpearl::backend::opengl::gl_command_encoder::GlCommandEncoder;
use crate::com::mojang::renderpearl::backend::opengl::gl_render_pipeline::GlRenderPipeline;
use std::any::Any;
use std::rc::Rc;

pub struct GlRenderPass<'a> {
    encoder: &'a GlCommandEncoder,
    default_scissor_state: ScissorState,
    pub(super) pipeline: Option<Rc<GlRenderPipeline>>,
    pub(super) vertex_buffers: [Option<GpuBufferSlice>; MAX_VERTEX_BUFFERS],
    pub(super) vertex_buffer_dirty: bool,
    pub(super) index_buffer: Option<Rc<dyn GpuBuffer>>,
    pub(super) index_type: IndexType,
    pub(super) index_buffer_dirty: bool,
    pub(super) scissor_state: ScissorState,
    pub(super) scissor_state_dirty: bool,
    pub(super) uniforms: Vec<Option<UniformValue>>,
    pub(super) dirty_uniforms: Vec<bool>,
    pub(super) any_uniform_dirty: bool,
    pub(super) color_attachment_count: usize,
}

impl<'a> GlRenderPass<'a> {
    pub fn new(
        encoder: &'a GlCommandEncoder,
        color_attachment_count: usize,
        default_scissor_state: ScissorState,
    ) -> GlRenderPass<'a> {
        GlRenderPass {
            encoder,
            default_scissor_state,
            pipeline: None,
            vertex_buffers: Default::default(),
            vertex_buffer_dirty: true,
            index_buffer: None,
            index_type: IndexType::INT,
            index_buffer_dirty: false,
            scissor_state: default_scissor_state,
            scissor_state_dirty: true,
            uniforms: Vec::new(),
            dirty_uniforms: Vec::new(),
            any_uniform_dirty: false,
            color_attachment_count,
        }
    }

    pub fn color_attachment_count(&self) -> usize {
        self.color_attachment_count
    }

    pub fn is_scissor_enabled(&self) -> bool {
        self.scissor_state.enabled()
    }
}

impl RenderPassBackend for GlRenderPass<'_> {
    /// Panics where Java throws `IllegalArgumentException` for another backend's pipeline.
    fn set_pipeline(&mut self, pipeline: &Rc<dyn BackendRenderPipeline>) {
        let gl_render_pipeline: Rc<GlRenderPipeline> = (pipeline.clone() as Rc<dyn Any>)
            .downcast()
            .expect("Pipeline must be instance of GlRenderPipeline");
        if self
            .pipeline
            .as_ref()
            .is_none_or(|current| !Rc::ptr_eq(current, &gl_render_pipeline))
        {
            let uniform_count = gl_render_pipeline.program().uniform_count();
            self.uniforms.clear();
            self.uniforms.resize(uniform_count, None);
            self.dirty_uniforms.clear();
            self.dirty_uniforms.resize(uniform_count, true);
            self.any_uniform_dirty = true;
        }
        self.pipeline = Some(gl_render_pipeline);
        self.vertex_buffer_dirty = true;
        self.index_buffer_dirty = self.index_buffer.is_some();
    }

    fn set_uniform(&mut self, index: i32, value: Option<&UniformValue>) {
        self.uniforms[index as usize] = value.cloned();
        self.dirty_uniforms[index as usize] = true;
        self.any_uniform_dirty = true;
    }

    fn enable_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.scissor_state.enable(x, y, width, height);
        self.scissor_state_dirty = true;
    }

    fn disable_scissor(&mut self) {
        self.scissor_state = self.default_scissor_state;
        self.scissor_state_dirty = true;
    }

    fn set_vertex_buffer(&mut self, slot: i32, vertex_buffer: Option<&GpuBufferSlice>) {
        let existing = &self.vertex_buffers[slot as usize];
        let same_buffer = match (vertex_buffer, existing) {
            (Some(input), Some(existing)) => Rc::ptr_eq(&input.buffer, &existing.buffer),
            (None, None) => true,
            _ => false,
        };
        let input_offset = vertex_buffer.map_or(0, |slice| slice.offset);
        let existing_offset = existing.as_ref().map_or(0, |slice| slice.offset);
        self.vertex_buffer_dirty |= !same_buffer || input_offset != existing_offset;
        self.vertex_buffers[slot as usize] = vertex_buffer.cloned();
    }

    fn set_index_buffer(&mut self, index_buffer: &Rc<dyn GpuBuffer>, index_type: IndexType) {
        self.index_buffer = Some(index_buffer.clone());
        self.index_type = index_type;
        self.index_buffer_dirty = true;
    }

    fn draw_indexed(
        &mut self,
        index_count: i32,
        instance_count: i32,
        first_index: i32,
        vertex_offset: i32,
        first_instance: i32,
    ) {
        let index_type = self.index_type;
        self.encoder.execute_draw(
            self,
            vertex_offset,
            first_index,
            index_count,
            Some(index_type),
            instance_count,
            first_instance,
        );
    }

    fn draw(
        &mut self,
        vertex_count: i32,
        instance_count: i32,
        first_vertex: i32,
        first_instance: i32,
    ) {
        self.encoder.execute_draw(
            self,
            first_vertex,
            0,
            vertex_count,
            None,
            instance_count,
            first_instance,
        );
    }
    fn draw_indexed_indirect(&mut self, commands: &GpuBufferSlice, draw_count: i32) {
        let index_type = self.index_type;
        self.encoder
            .execute_draw_indirect(self, Some(index_type), commands, draw_count);
    }
}
