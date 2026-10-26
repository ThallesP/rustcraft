//! com/mojang/renderpearl/backend/vulkan/VulkanRenderPass.java
//!
//! A pass's commands, recorded into the encoder's command buffer inside dynamic rendering: the
//! viewport over the whole output (not flipped: the frame is upside down from GL's in memory the
//! same way, which `Screenshot` turns over in both), the scissor, the pipeline (with or without a
//! depth attachment), vertex and index buffers, and the uniforms as push descriptors, pushed
//! before a draw when any changed. A texel buffer's view is made for the push and its
//! destruction queued. Debug groups, timestamps, push constants and the direct multi-draws come
//! with `RenderPassBackend`'s.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::{
    INDIRECT_INDEXED_DRAW_SIZE, RenderArea,
};
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::BackendRenderPipeline;
use crate::com::mojang::renderpearl::backend::api::render_pass_backend::{
    RenderPassBackend, UniformValue,
};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_command_encoder::VulkanCommandEncoder;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_buffer::VulkanGpuBuffer;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_sampler::VulkanGpuSampler;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture::VulkanGpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_render_pipeline::VulkanRenderPipeline;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk;
use std::rc::Rc;

pub struct VulkanRenderPass<'a> {
    device: Rc<VulkanDevice>,
    encoder: &'a VulkanCommandEncoder,
    render_area: RenderArea,
    has_depth: bool,
    command_buffer: vk::CommandBuffer,
    pipeline: Option<Rc<dyn BackendRenderPipeline>>,
    any_descriptor_dirty: bool,
    uniforms: Vec<Option<UniformValue>>,
}

impl<'a> VulkanRenderPass<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: Rc<VulkanDevice>,
        encoder: &'a VulkanCommandEncoder,
        command_buffer: vk::CommandBuffer,
        render_area: RenderArea,
        output_width: i32,
        output_height: i32,
        has_depth: bool,
    ) -> VulkanRenderPass<'a> {
        let viewport = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: output_width as f32,
            height: output_height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        // SAFETY: the buffer is recording.
        unsafe {
            device
                .vk_device()
                .cmd_set_viewport(command_buffer, 0, &viewport)
        };
        let pass = VulkanRenderPass {
            device,
            encoder,
            render_area,
            has_depth,
            command_buffer,
            pipeline: None,
            any_descriptor_dirty: false,
            uniforms: Vec::new(),
        };
        pass.set_scissor(
            render_area.x,
            render_area.y,
            render_area.width,
            render_area.height,
        );
        pass
    }

    fn set_scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D { x, y },
            extent: vk::Extent2D {
                width: width as u32,
                height: height as u32,
            },
        }];
        // SAFETY: the buffer is recording.
        unsafe {
            self.device
                .vk_device()
                .cmd_set_scissor(self.command_buffer, 0, &scissor)
        };
    }

    fn pipeline(&self) -> &VulkanRenderPipeline {
        self.pipeline
            .as_ref()
            .expect("a pipeline")
            .as_any()
            .downcast_ref::<VulkanRenderPipeline>()
            .expect("Pipeline must be instance of VulkanRenderPipeline")
    }

    /// `pushDescriptors()`: every uniform the pipeline has, when any changed. Panics where Java
    /// throws `IllegalStateException`, for one the pass hasn't been given.
    fn push_descriptors(&mut self) {
        if !self.any_descriptor_dirty {
            return;
        }
        let vk_device = self.device.vk_device();
        let pipeline = self.pipeline();
        let uniforms = pipeline.uniforms();
        let mut buffer_infos = Vec::with_capacity(uniforms.len());
        let mut image_infos = Vec::with_capacity(uniforms.len());
        let mut buffer_views = Vec::with_capacity(uniforms.len());
        for (i, uniform) in uniforms.iter().enumerate() {
            match uniform.uniform_type {
                UniformType::UNIFORM_BUFFER => {
                    let Some(UniformValue::Buffer(buffer)) = &self.uniforms[i] else {
                        panic!(
                            "Missing uniform {} (should be {:?})",
                            uniform.name, uniform.uniform_type
                        );
                    };
                    buffer_infos.push((
                        i,
                        [vk::DescriptorBufferInfo {
                            buffer: VulkanGpuBuffer::of(&*buffer.buffer).vk_buffer(),
                            offset: buffer.offset as u64,
                            range: buffer.length as u64,
                        }],
                    ));
                }
                UniformType::COMBINED_IMAGE_SAMPLER => {
                    let Some(UniformValue::Sampler(value)) = &self.uniforms[i] else {
                        panic!("Missing sampler {}", uniform.name);
                    };
                    image_infos.push((
                        i,
                        [vk::DescriptorImageInfo {
                            sampler: VulkanGpuSampler::of(&*value.sampler).vk_sampler(),
                            image_view: VulkanGpuTexture::image_view(&value.view),
                            image_layout: vk::ImageLayout::GENERAL,
                        }],
                    ));
                }
                UniformType::TEXEL_BUFFER => {
                    let Some(UniformValue::Buffer(value)) = &self.uniforms[i] else {
                        panic!(
                            "Missing uniform {} (should be {:?})",
                            uniform.name, uniform.uniform_type
                        );
                    };
                    let view_create_info = vk::BufferViewCreateInfo::default()
                        .buffer(VulkanGpuBuffer::of(&*value.buffer).vk_buffer())
                        .offset(value.offset as u64)
                        .range(value.length as u64)
                        .format(vk::Format::from_raw(VulkanConst::to_vk_format(
                            uniform.gpu_format.expect("a texel buffer's format"),
                        )));
                    // SAFETY: the buffer is the device's, and the create info lives for the call.
                    let buffer_view_handle = VulkanUtils::crash_if_err(
                        unsafe { vk_device.create_buffer_view(&view_create_info, None) },
                        "Couldn't create buffer view for texel buffer",
                    );
                    let device = self.device.clone();
                    self.encoder.queue_for_destroy(Box::new(move || {
                        // SAFETY: the view is the device's, and the GPU is done with it.
                        unsafe {
                            device
                                .vk_device()
                                .destroy_buffer_view(buffer_view_handle, None)
                        }
                    }));
                    buffer_views.push((i, [buffer_view_handle]));
                }
            }
        }

        let mut writes: Vec<(usize, vk::WriteDescriptorSet<'_>)> =
            Vec::with_capacity(uniforms.len());
        for (i, info) in &buffer_infos {
            writes.push((
                *i,
                vk::WriteDescriptorSet::default()
                    .dst_binding(*i as u32)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(info),
            ));
        }
        for (i, info) in &image_infos {
            writes.push((
                *i,
                vk::WriteDescriptorSet::default()
                    .dst_binding(*i as u32)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(info),
            ));
        }
        for (i, view) in &buffer_views {
            writes.push((
                *i,
                vk::WriteDescriptorSet::default()
                    .dst_binding(*i as u32)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_TEXEL_BUFFER)
                    .texel_buffer_view(view),
            ));
        }
        writes.sort_by_key(|(i, _)| *i);
        let writes: Vec<vk::WriteDescriptorSet<'_>> =
            writes.into_iter().map(|(_, write)| write).collect();
        // SAFETY: the buffer is recording inside the pass, with the pipeline bound, and the
        // writes' infos live for the call.
        unsafe {
            self.device.push_descriptor().cmd_push_descriptor_set(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline_layout(),
                0,
                &writes,
            );
        }
        self.any_descriptor_dirty = false;
    }
}

impl RenderPassBackend for VulkanRenderPass<'_> {
    /// Panics where Java throws `IllegalArgumentException`, for another backend's pipeline.
    fn set_pipeline(&mut self, pipeline: &Rc<dyn BackendRenderPipeline>) {
        self.pipeline = Some(pipeline.clone());
        let vulkan_pipeline = self.pipeline();
        let uniform_count = vulkan_pipeline.uniforms().len();
        let vk_pipeline = if self.has_depth {
            vulkan_pipeline.with_depth_pipeline()
        } else {
            vulkan_pipeline.without_depth_pipeline()
        };
        // SAFETY: the buffer is recording inside the pass, and the pipeline is the device's.
        unsafe {
            self.device.vk_device().cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                vk_pipeline,
            );
        }
        self.any_descriptor_dirty = true;
        self.uniforms.clear();
        self.uniforms.resize(uniform_count, None);
    }

    fn set_uniform(&mut self, index: i32, value: Option<&UniformValue>) {
        self.uniforms[index as usize] = value.cloned();
        self.any_descriptor_dirty = true;
    }

    fn enable_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.set_scissor(x, y, width, height);
    }

    fn disable_scissor(&mut self) {
        let area = self.render_area;
        self.set_scissor(area.x, area.y, area.width, area.height);
    }

    fn set_vertex_buffer(&mut self, slot: i32, vertex_buffer: Option<&GpuBufferSlice>) {
        if let Some(vertex_buffer) = vertex_buffer {
            let buffer = VulkanGpuBuffer::of(&*vertex_buffer.buffer).vk_buffer();
            // SAFETY: the buffer is recording, and the vertex buffer is the device's.
            unsafe {
                self.device.vk_device().cmd_bind_vertex_buffers(
                    self.command_buffer,
                    slot as u32,
                    &[buffer],
                    &[vertex_buffer.offset as u64],
                );
            }
        }
    }

    fn set_index_buffer(&mut self, index_buffer: &Rc<dyn GpuBuffer>, index_type: IndexType) {
        let r#type = match index_type {
            IndexType::SHORT => vk::IndexType::UINT16,
            IndexType::INT => vk::IndexType::UINT32,
        };
        // SAFETY: as above.
        unsafe {
            self.device.vk_device().cmd_bind_index_buffer(
                self.command_buffer,
                VulkanGpuBuffer::of(&**index_buffer).vk_buffer(),
                0,
                r#type,
            );
        }
    }

    fn draw_indexed(
        &mut self,
        index_count: i32,
        instance_count: i32,
        first_index: i32,
        vertex_offset: i32,
        first_instance: i32,
    ) {
        self.push_descriptors();
        // SAFETY: the buffer is recording inside the pass, with everything bound.
        unsafe {
            self.device.vk_device().cmd_draw_indexed(
                self.command_buffer,
                index_count as u32,
                instance_count as u32,
                first_index as u32,
                vertex_offset,
                first_instance as u32,
            );
        }
    }

    fn draw(
        &mut self,
        vertex_count: i32,
        instance_count: i32,
        first_vertex: i32,
        first_instance: i32,
    ) {
        self.push_descriptors();
        // SAFETY: as above.
        unsafe {
            self.device.vk_device().cmd_draw(
                self.command_buffer,
                vertex_count as u32,
                instance_count as u32,
                first_vertex as u32,
                first_instance as u32,
            );
        }
    }

    fn draw_indexed_indirect(&mut self, commands: &GpuBufferSlice, draw_count: i32) {
        self.push_descriptors();
        // SAFETY: as above, and the commands are the device's buffer's.
        unsafe {
            self.device.vk_device().cmd_draw_indexed_indirect(
                self.command_buffer,
                VulkanGpuBuffer::of(&*commands.buffer).vk_buffer(),
                commands.offset as u64,
                draw_count as u32,
                INDIRECT_INDEXED_DRAW_SIZE as u32,
            );
        }
    }
}
