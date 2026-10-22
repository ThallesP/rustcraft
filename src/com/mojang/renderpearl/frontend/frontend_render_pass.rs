//! com/mojang/renderpearl/frontend/FrontendRenderPass.java
//!
//! Checks each call before the backend's pass runs it, and binds uniforms by name: it keeps every
//! uniform it's given, and hands the bound pipeline's ones to the backend by their index, again
//! whenever the pipeline changes. Closing it (dropping) runs the encoder's `onFinish`.
//!
//! `validateDraw`'s checks beyond a bound pipeline only run with `STRICT_VALIDATION`, which is
//! `SharedConstants.IS_RUNNING_IN_IDE` and off in the game, so they aren't ported, nor is
//! `hasDepthAttachment`, which only they read. Debug groups,
//! timestamps, push constants and the multi-draws come with `RenderPass`'s.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::{
    MAX_VERTEX_BUFFERS, RenderArea, RenderPass,
};
use crate::com::mojang::renderpearl::api::device::device_features::DeviceFeatures;
use crate::com::mojang::renderpearl::api::device::device_limits::DeviceLimits;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::api::render_pass_backend::{
    RenderPassBackend, UniformValue,
};
use crate::com::mojang::renderpearl::frontend::frontend_render_pipeline::FrontendRenderPipeline;
use crate::com::mojang::renderpearl::util::texture_view_and_sampler::TextureViewAndSampler;
use std::rc::Rc;

pub struct FrontendRenderPass<'a> {
    backend: Box<dyn RenderPassBackend + 'a>,
    device_features: DeviceFeatures,
    device_limits: DeviceLimits,
    on_finish: Option<Box<dyn FnOnce() + 'a>>,
    render_area: RenderArea,
    /// Each color attachment's texture format, `None` for an unused one.
    color_attachments: Vec<Option<GpuFormat>>,
    is_closed: bool,
    bound_pipeline: Option<Rc<dyn CompiledRenderPipeline>>,
    vertex_buffers: [Option<GpuBufferSlice>; MAX_VERTEX_BUFFERS],
    index_buffer: Option<Rc<dyn GpuBuffer>>,
    /// Java's `HashMap<String, Object>`, whose order only decides the order the backend is handed
    /// them in when the pipeline changes, which nothing sees.
    uniforms: Vec<(String, UniformValue)>,
    constants_pushed: bool,
}

impl<'a> FrontendRenderPass<'a> {
    pub fn new(
        backend: Box<dyn RenderPassBackend + 'a>,
        device_features: DeviceFeatures,
        device_limits: DeviceLimits,
        color_attachments: Vec<Option<GpuFormat>>,
        on_finish: Box<dyn FnOnce() + 'a>,
        render_area: RenderArea,
    ) -> FrontendRenderPass<'a> {
        FrontendRenderPass {
            backend,
            device_features,
            device_limits,
            on_finish: Some(on_finish),
            render_area,
            color_attachments,
            is_closed: false,
            bound_pipeline: None,
            vertex_buffers: Default::default(),
            index_buffer: None,
            uniforms: Vec::new(),
            constants_pushed: false,
        }
    }

    fn bound(&self) -> Option<&FrontendRenderPipeline> {
        self.bound_pipeline.as_ref().map(|pipeline| {
            pipeline
                .as_any()
                .downcast_ref::<FrontendRenderPipeline>()
                .expect("a frontend pipeline")
        })
    }

    /// `setUniform(String, Object)`: keeps the value (or forgets it), and hands it to the backend
    /// if the bound pipeline has the uniform.
    fn set_uniform(&mut self, name: &str, value: Option<UniformValue>) {
        let existing = self.uniforms.iter().position(|(key, _)| key == name);
        match (&value, existing) {
            (None, Some(index)) => {
                self.uniforms.remove(index);
            }
            (Some(value), Some(index)) => self.uniforms[index].1 = value.clone(),
            (Some(value), None) => self.uniforms.push((name.to_owned(), value.clone())),
            (None, None) => {}
        }
        if let Some(pipeline) = self.bound() {
            let uniform_index = pipeline.uniform_index(name);
            if uniform_index != -1 {
                self.backend.set_uniform(uniform_index, value.as_ref());
            }
        }
    }

    fn check_open(&self) {
        assert!(!self.is_closed, "Can't use a closed render pass");
    }

    /// Panics where Java throws `IllegalStateException`.
    fn validate_draw(&self) {
        assert!(
            self.bound_pipeline.is_some(),
            "Can't draw without a render pipeline"
        );
    }
}

impl RenderPass for FrontendRenderPass<'_> {
    /// Panics where Java throws `IllegalStateException`: the pipeline's color targets have to be
    /// the pass's attachments, with the same formats.
    fn set_pipeline(&mut self, pipeline: &Rc<dyn CompiledRenderPipeline>) {
        let frontend_pipeline = pipeline
            .as_any()
            .downcast_ref::<FrontendRenderPipeline>()
            .expect("Pipeline must be instance of FrontendCompiledRenderPipeline");
        let color_target_states = &frontend_pipeline.color_target_states;
        assert!(
            color_target_states.len() == self.color_attachments.len(),
            "Render pass color attachment count must match pipeline color target state count."
        );
        for (i, attachment) in self.color_attachments.iter().enumerate() {
            if let Some(format) = attachment {
                let matches = color_target_states[i]
                    .as_ref()
                    .is_some_and(|state| state.format == *format);
                assert!(
                    matches,
                    "Render pass color attachment {i} format doesn't match pipeline format."
                );
            }
        }

        self.backend
            .set_pipeline(&frontend_pipeline.backend_render_pipeline);
        self.bound_pipeline = Some(pipeline.clone());
        for (name, value) in self.uniforms.clone() {
            self.set_uniform(&name, Some(value));
        }
        self.constants_pushed = false;
    }

    /// Panics where Java throws `IllegalArgumentException`, for only one of the two.
    fn set_uniform_sampler(
        &mut self,
        name: &str,
        texture_view: Option<&GpuTextureView>,
        sampler: Option<&Rc<dyn GpuSampler>>,
    ) {
        match (texture_view, sampler) {
            (Some(view), Some(sampler)) => {
                let pair = TextureViewAndSampler {
                    view: view.clone(),
                    sampler: sampler.clone(),
                };
                self.set_uniform(name, Some(UniformValue::Sampler(pair)));
            }
            (None, None) => self.set_uniform(name, None),
            _ => panic!("textureView and sampler must both or neither be null"),
        }
    }

    fn set_uniform_buffer(&mut self, name: &str, value: &Rc<dyn GpuBuffer>) {
        self.set_uniform_slice(name, &gpu_buffer::slice(value));
    }

    /// Panics where Java throws `IllegalArgumentException` for an offset that isn't aligned.
    fn set_uniform_slice(&mut self, name: &str, value: &GpuBufferSlice) {
        let alignment = i64::from(self.device_limits.min_uniform_offset_alignment);
        assert!(
            value.offset % alignment <= 0,
            "Uniform buffer offset must be aligned to {alignment}"
        );
        self.set_uniform(name, Some(UniformValue::Buffer(value.clone())));
    }

    /// Panics where Java throws `IllegalArgumentException`, for an empty scissor or one outside
    /// the render area.
    fn enable_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        assert!(
            width > 0 && height > 0,
            "Scissor size must be >0, was {width}x{height}"
        );
        let area = self.render_area;
        assert!(
            x >= area.x
                && y >= area.y
                && x + width <= area.x + area.width
                && y + height <= area.y + area.height,
            "Scissor at {x}, {y} with size {width}x{height} is out of bounds for render area {area:?}"
        );
        self.backend.enable_scissor(x, y, width, height);
    }

    fn disable_scissor(&mut self) {
        self.backend.disable_scissor();
    }

    /// Panics where Java throws, for a slot out of range or a buffer that isn't a vertex buffer.
    fn set_vertex_buffer(&mut self, slot: i32, vertex_buffer: Option<&GpuBufferSlice>) {
        assert!(
            (0..MAX_VERTEX_BUFFERS as i32).contains(&slot),
            "Vertex buffer slot is out of range: {slot}"
        );
        if let Some(vertex_buffer) = vertex_buffer {
            assert!(
                vertex_buffer.buffer.usage() & gpu_buffer::USAGE_VERTEX != 0,
                "Vertex buffer at slot {slot} doesn't have GpuBuffer.USAGE_VERTEX flag!"
            );
        }
        self.vertex_buffers[slot as usize] = vertex_buffer.cloned();
        self.backend.set_vertex_buffer(slot, vertex_buffer);
    }

    fn set_index_buffer(&mut self, index_buffer: &Rc<dyn GpuBuffer>, index_type: IndexType) {
        self.index_buffer = Some(index_buffer.clone());
        self.backend.set_index_buffer(index_buffer, index_type);
    }

    /// Panics where Java throws, for a first instance the device can't start from.
    fn draw_indexed(
        &mut self,
        index_count: i32,
        instance_count: i32,
        first_index: i32,
        vertex_offset: i32,
        first_instance: i32,
    ) {
        self.check_open();
        assert!(
            first_instance == 0 || self.device_features.non_zero_first_instance,
            "firstInstance must be zero on when device does not support nonZeroFirstInstance"
        );
        self.validate_draw();
        self.backend.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        );
    }

    /// Panics where Java throws, for a first instance the device can't start from.
    fn draw(
        &mut self,
        vertex_count: i32,
        instance_count: i32,
        first_vertex: i32,
        first_instance: i32,
    ) {
        self.check_open();
        assert!(
            first_instance == 0 || self.device_features.non_zero_first_instance,
            "firstInstance must be zero on when device does not support nonZeroFirstInstance"
        );
        self.validate_draw();
        self.backend
            .draw(vertex_count, instance_count, first_vertex, first_instance);
    }
}

impl Drop for FrontendRenderPass<'_> {
    fn drop(&mut self) {
        if !self.is_closed {
            self.is_closed = true;
            if let Some(on_finish) = self.on_finish.take() {
                on_finish();
            }
        }
    }
}
