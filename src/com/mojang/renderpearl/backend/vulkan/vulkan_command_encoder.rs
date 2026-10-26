//! com/mojang/renderpearl/backend/vulkan/VulkanCommandEncoder.java
//!
//! Records a submit's commands into command buffers from one of two pools, one for each submit in
//! flight: a buffer for setting up new objects (images' layouts, buffers' first data), run first,
//! then the frame's. Every copy, clear and pass ends with a full memory barrier; images stay in
//! the `GENERAL` layout. `submit` hands the submission to the graphics queue with a timeline
//! semaphore's next value, waits for the submit before last, and destroys what was queued for
//! destruction two submits ago, which is when a copy's callback runs. A render pass is dynamic
//! rendering over the attachments' views.
//!
//! The encoder is the API's `CommandEncoder` until the frontend's is ported, so it makes each
//! render pass a `FrontendRenderPass` around its own and runs the frontend's checks
//! (`FrontendCommandEncoder`). Not ported: fences, timestamps, copies between buffers and between
//! textures, and the clear of a region (`clearColorAndDepthTextures` with a rectangle), which
//! nothing ported calls.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::checkpoint_extension::{
    CheckpointStorage, CheckpointType,
};
use crate::com::mojang::renderpearl::backend::vulkan::destroyable::Destroyable;
use crate::com::mojang::renderpearl::backend::vulkan::destruction_queue::DestructionQueue;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_command_pool::VulkanCommandPool;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_buffer::VulkanGpuBuffer;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture::VulkanGpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_queue::Submission;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_render_pass::VulkanRenderPass;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_transient_memory::VulkanTransientMemory;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::com::mojang::renderpearl::frontend::frontend_command_encoder::FrontendCommandEncoder;
use crate::com::mojang::renderpearl::frontend::frontend_render_pass::FrontendRenderPass;
use crate::org::joml::vector4f::Vector4f;
use ash::vk;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub const MAX_SUBMITS_IN_FLIGHT: u64 = 2;
/// `VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT`.
const ALL_COMMANDS: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::ALL_COMMANDS;
/// Memory reads and writes of any kind.
const MEMORY_READ_WRITE: vk::AccessFlags2 = vk::AccessFlags2::from_raw(98304);

pub struct VulkanCommandEncoder {
    device: Weak<VulkanDevice>,
    transient_memory: VulkanTransientMemory,
    submit_semaphore: vk::Semaphore,
    current_submit_index: Cell<u64>,
    completed_submit_index: Cell<u64>,
    checkpoint_storage: Box<dyn CheckpointStorage>,
    submission_builder: RefCell<Submission>,
    destroy_queue: DestructionQueue,
    command_pools: [VulkanCommandPool; 2],
    current_init_command_buffer: Cell<Option<vk::CommandBuffer>>,
    current_command_buffer: Cell<Option<vk::CommandBuffer>>,
    /// `currentRenderPass != null`, with the pass's label.
    current_render_pass: RefCell<Option<String>>,
}

impl VulkanCommandEncoder {
    pub fn new(device: &Rc<VulkanDevice>) -> VulkanCommandEncoder {
        let vk_device = device.vk_device();
        let current_submit_index = 2;
        let mut semaphore_type_create_info = vk::SemaphoreTypeCreateInfo::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(current_submit_index - 1);
        let semaphore_create_info =
            vk::SemaphoreCreateInfo::default().push_next(&mut semaphore_type_create_info);
        // SAFETY: the create info lives for the call.
        let submit_semaphore = VulkanUtils::crash_if_err(
            unsafe { vk_device.create_semaphore(&semaphore_create_info, None) },
            "Failed to create submit VkSemaphore",
        );
        let family = device.graphics_queue().queue_family_index();
        let encoder = VulkanCommandEncoder {
            device: Rc::downgrade(device),
            transient_memory: VulkanTransientMemory::new(device),
            submit_semaphore,
            current_submit_index: Cell::new(current_submit_index),
            completed_submit_index: Cell::new(0),
            checkpoint_storage: device
                .checkpoint_extension()
                .create_storage(MAX_SUBMITS_IN_FLIGHT as i32),
            submission_builder: RefCell::new(device.graphics_queue().begin_submit()),
            destroy_queue: DestructionQueue::new(MAX_SUBMITS_IN_FLIGHT as usize),
            command_pools: [
                VulkanCommandPool::new(vk_device, family),
                VulkanCommandPool::new(vk_device, family),
            ],
            current_init_command_buffer: Cell::new(None),
            current_command_buffer: Cell::new(None),
            current_render_pass: RefCell::new(None),
        };
        encoder.transient_memory.begin_submit(&encoder);
        encoder
    }

    fn device(&self) -> Rc<VulkanDevice> {
        self.device.upgrade().expect("the encoder's device")
    }

    /// `destroy()`.
    pub fn destroy(&self) {
        let device = self.device();
        let vk_device = device.vk_device();
        self.transient_memory.end_submit(self, vk_device);
        self.submission_builder
            .borrow_mut()
            .close(device.synchronization2());
        device.graphics_queue().wait_idle(vk_device);
        self.destroy_queue.close();
        self.transient_memory.destroy();
        self.destroy_queue.close();
        for pool in &self.command_pools {
            pool.destroy(vk_device);
        }
        // SAFETY: the GPU is done with the semaphore.
        unsafe { vk_device.destroy_semaphore(self.submit_semaphore, None) };
    }

    pub fn queue_for_destroy(&self, destroyable: Destroyable) {
        self.destroy_queue.add(destroyable);
    }

    pub fn transient_memory(&self) -> &VulkanTransientMemory {
        &self.transient_memory
    }

    fn current_command_pool(&self) -> &VulkanCommandPool {
        &self.command_pools[(self.current_submit_index.get() % MAX_SUBMITS_IN_FLIGHT) as usize]
    }

    pub fn allocate_and_begin_transient_command_buffer(&self) -> vk::CommandBuffer {
        let device = self.device();
        let vk_device = device.vk_device();
        let command_buffer = self.current_command_pool().allocate_buffer(vk_device);
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: the buffer is the pool's, in its initial state.
        let result = unsafe { vk_device.begin_command_buffer(command_buffer, &begin_info) };
        VulkanUtils::crash_if_err(result, "Failed to begin VkCommandBuffer");
        command_buffer
    }

    /// The frame's command buffer, begun the first time it's asked for. Panics where Java throws
    /// `IllegalStateException`, inside a render pass.
    fn command_buffer(&self) -> vk::CommandBuffer {
        if let Some(command_buffer) = self.current_command_buffer.get() {
            return command_buffer;
        }
        assert!(
            self.current_render_pass.borrow().is_none(),
            "Cannot start command buffer while inside RenderPass"
        );
        let command_buffer = self.allocate_and_begin_transient_command_buffer();
        self.current_command_buffer.set(Some(command_buffer));
        command_buffer
    }

    /// `objectInitCommandBuffer()`: what new objects' setup goes in, run before the frame's.
    pub fn object_init_command_buffer(&self) -> vk::CommandBuffer {
        if let Some(command_buffer) = self.current_init_command_buffer.get() {
            return command_buffer;
        }
        let command_buffer = self.allocate_and_begin_transient_command_buffer();
        self.current_init_command_buffer.set(Some(command_buffer));
        command_buffer
    }

    fn end_command_buffer(&self) {
        if self.current_command_buffer.get().is_none()
            && self.current_init_command_buffer.get().is_none()
        {
            return;
        }
        assert!(
            self.current_render_pass.borrow().is_none(),
            "Cannot end command buffer while inside RenderPass"
        );
        let device = self.device();
        let vk_device = device.vk_device();
        if let Some(init_command_buffer) = self.current_init_command_buffer.take() {
            VulkanCommandEncoder::memory_barrier(&device, init_command_buffer);
            // SAFETY: the buffer is recording.
            let result = unsafe { vk_device.end_command_buffer(init_command_buffer) };
            VulkanUtils::crash_if_err(result, "Failed to end VkCommandBuffer");
            self.submission_builder
                .borrow_mut()
                .execute_commands(init_command_buffer);
        }
        if let Some(command_buffer) = self.current_command_buffer.take() {
            // SAFETY: as above.
            let result = unsafe { vk_device.end_command_buffer(command_buffer) };
            VulkanUtils::crash_if_err(result, "Failed to end VkCommandBuffer");
            self.submission_builder
                .borrow_mut()
                .execute_commands(command_buffer);
        }
    }

    pub fn wait_semaphore(
        &self,
        vk_semaphore: vk::Semaphore,
        value: u64,
        stage_mask: vk::PipelineStageFlags2,
    ) {
        assert!(
            self.current_render_pass.borrow().is_none(),
            "Cannot add semaphore operation while inside RenderPass"
        );
        self.end_command_buffer();
        self.submission_builder
            .borrow_mut()
            .wait_semaphore(vk_semaphore, value, stage_mask);
    }

    pub fn execute(&self, command_buffer: vk::CommandBuffer) {
        assert!(
            self.current_render_pass.borrow().is_none(),
            "Cannot execute command buffer while inside RenderPass"
        );
        self.end_command_buffer();
        self.submission_builder
            .borrow_mut()
            .execute_commands(command_buffer);
    }

    pub fn signal_semaphore(
        &self,
        vk_semaphore: vk::Semaphore,
        value: u64,
        stage_mask: vk::PipelineStageFlags2,
    ) {
        assert!(
            self.current_render_pass.borrow().is_none(),
            "Cannot add semaphore operation while inside RenderPass"
        );
        self.end_command_buffer();
        self.submission_builder
            .borrow_mut()
            .signal_semaphore(vk_semaphore, value, stage_mask);
    }

    /// `memoryBarrier(VkCommandBuffer, MemoryStack)`: every command before done with memory before
    /// any after touches it.
    pub fn memory_barrier(device: &VulkanDevice, command_buffer: vk::CommandBuffer) {
        let memory_barrier = [vk::MemoryBarrier2::default()
            .src_stage_mask(ALL_COMMANDS)
            .src_access_mask(MEMORY_READ_WRITE)
            .dst_stage_mask(ALL_COMMANDS)
            .dst_access_mask(MEMORY_READ_WRITE)];
        let dep_info = vk::DependencyInfo::default().memory_barriers(&memory_barrier);
        // SAFETY: the buffer is recording.
        unsafe {
            device
                .synchronization2()
                .cmd_pipeline_barrier2(command_buffer, &dep_info)
        };
    }

    fn memory_barrier_frame(&self, device: &VulkanDevice) {
        VulkanCommandEncoder::memory_barrier(device, self.command_buffer());
    }

    /// `awaitSubmitCompletion`: whether the submit is done within the timeout. Panics where Java
    /// throws `IllegalStateException`, for the submit still being recorded.
    fn await_submit_completion(&self, submit_index: u64, timeout_ns: u64) -> bool {
        if self.completed_submit_index.get() >= submit_index {
            return true;
        }
        if submit_index == self.current_submit_index.get() {
            assert!(
                timeout_ns == 0,
                "Cannot wait on a fence for the current submit"
            );
            return false;
        }
        let device = self.device();
        let semaphores = [self.submit_semaphore];
        let values = [submit_index];
        let wait_info = vk::SemaphoreWaitInfo::default()
            .semaphores(&semaphores)
            .values(&values);
        // SAFETY: the semaphore is the device's, and the info lives for the call.
        let result = unsafe { device.vk_device().wait_semaphores(&wait_info, timeout_ns) };
        let completed = match result {
            Ok(()) => true,
            Err(vk::Result::TIMEOUT) => false,
            Err(error) => {
                VulkanUtils::crash_if_failure(error, "Failed to wait for semaphore");
                false
            }
        };
        if completed {
            self.completed_submit_index.set(submit_index);
        }
        completed
    }

    /// `submitRenderPass()`: the pass's end. Panics where Java throws, with none open.
    fn submit_render_pass(&self) {
        let label = self
            .current_render_pass
            .borrow_mut()
            .take()
            .expect("Cannot submit a renderpass if one hasn't been started!");
        let device = self.device();
        let command_buffer = self
            .current_command_buffer
            .get()
            .expect("the pass's buffer");
        // SAFETY: the buffer is recording inside the pass.
        unsafe { device.dynamic_rendering().cmd_end_rendering(command_buffer) };
        device.instance().debug().end_debug_group(command_buffer);
        self.checkpoint_storage.record_checkpoint(
            command_buffer,
            CheckpointType::END_RENDER_PASS,
            &label,
        );
        self.memory_barrier_frame(&device);
    }

    fn subresource_range(
        texture: &dyn GpuTexture,
        aspect_mask: vk::ImageAspectFlags,
    ) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: texture.get_mip_levels() as u32,
            base_array_layer: 0,
            layer_count: 1,
        }
    }

    fn clear_color_texture_unsynced(
        &self,
        device: &VulkanDevice,
        color_texture: &dyn GpuTexture,
        clear_color: &Vector4f,
    ) {
        let vk_clear_color = VulkanUtils::put_argb(clear_color);
        let range = [VulkanCommandEncoder::subresource_range(
            color_texture,
            vk::ImageAspectFlags::COLOR,
        )];
        // SAFETY: the buffer is recording, and the image the device's.
        unsafe {
            device.vk_device().cmd_clear_color_image(
                self.command_buffer(),
                VulkanGpuTexture::of(color_texture).vk_image(),
                vk::ImageLayout::GENERAL,
                &vk_clear_color,
                &range,
            );
        }
    }

    fn clear_depth_texture_unsynced(
        &self,
        device: &VulkanDevice,
        depth_texture: &dyn GpuTexture,
        clear_depth: f64,
    ) {
        let vk_clear_depth = vk::ClearDepthStencilValue {
            depth: clear_depth as f32,
            stencil: 0,
        };
        let range = [VulkanCommandEncoder::subresource_range(
            depth_texture,
            vk::ImageAspectFlags::DEPTH,
        )];
        // SAFETY: as above.
        unsafe {
            device.vk_device().cmd_clear_depth_stencil_image(
                self.command_buffer(),
                VulkanGpuTexture::of(depth_texture).vk_image(),
                vk::ImageLayout::GENERAL,
                &vk_clear_depth,
                &range,
            );
        }
    }
}

impl CommandEncoder for VulkanCommandEncoder {
    /// Panics where Java throws `IllegalStateException`, when the submit before last doesn't
    /// finish within 5 seconds.
    fn submit(&self) {
        let device = self.device();
        let vk_device = device.vk_device();
        self.end_command_buffer();
        self.transient_memory.end_submit(self, vk_device);
        self.signal_semaphore(
            self.submit_semaphore,
            self.current_submit_index.get(),
            ALL_COMMANDS,
        );
        self.submission_builder
            .borrow_mut()
            .close(device.synchronization2());
        *self.submission_builder.borrow_mut() = device.graphics_queue().begin_submit();
        self.current_submit_index
            .set(self.current_submit_index.get() + 1);
        if !self.await_submit_completion(self.current_submit_index.get() - 2, 5_000_000_000) {
            let checkpoints = device.checkpoint_extension().retrieve_checkpoints(false);
            panic!(
                "5s timeout reached when waiting for VK semaphore: {}",
                VulkanUtils::format_checkpoints(&checkpoints)
            );
        }
        self.current_command_pool().reset(vk_device);
        self.destroy_queue.rotate();
        self.checkpoint_storage.rotate();
        self.transient_memory.begin_submit(self);
    }

    /// Panics where Java's frontend throws (`FrontendCommandEncoder`).
    fn create_render_pass(&self, descriptor: RenderPassDescriptor) -> Box<dyn RenderPass + '_> {
        let device = self.device();
        let device_info = device.get_device_info();
        FrontendCommandEncoder::check_render_pass(
            self.current_render_pass.borrow().is_some(),
            &descriptor,
            device_info.limits.max_color_attachments,
        );
        let command_buffer = self.command_buffer();
        device
            .instance()
            .debug()
            .begin_debug_group(command_buffer, &descriptor.label);
        self.checkpoint_storage.record_checkpoint(
            command_buffer,
            CheckpointType::BEGIN_RENDER_PASS,
            &descriptor.label,
        );

        let (mut width, mut height) = (0, 0);
        if !descriptor.color_attachments.is_empty() {
            for color_attachment in descriptor.color_attachments.iter().flatten() {
                width = color_attachment.texture_view.get_width(0);
                height = color_attachment.texture_view.get_height(0);
            }
        } else if let Some(depth_attachment) = &descriptor.depth_attachment {
            width = depth_attachment.texture_view.get_width(0);
            height = depth_attachment.texture_view.get_height(0);
        }

        let render_area = descriptor.render_area;
        let vk_render_area = vk::Rect2D {
            offset: vk::Offset2D {
                x: render_area.x,
                y: render_area.y,
            },
            extent: vk::Extent2D {
                width: render_area.width as u32,
                height: render_area.height as u32,
            },
        };
        let color_attachment_info: Vec<vk::RenderingAttachmentInfo<'_>> = descriptor
            .color_attachments
            .iter()
            .map(|attachment| match attachment {
                Some(attachment) => {
                    let info = vk::RenderingAttachmentInfo::default()
                        .image_view(VulkanGpuTexture::image_view(&attachment.texture_view))
                        .image_layout(vk::ImageLayout::GENERAL)
                        .store_op(vk::AttachmentStoreOp::STORE);
                    match attachment.clear_value {
                        Some(color) => {
                            info.load_op(vk::AttachmentLoadOp::CLEAR)
                                .clear_value(vk::ClearValue {
                                    color: VulkanUtils::put_argb(&color),
                                })
                        }
                        None => info.load_op(vk::AttachmentLoadOp::LOAD),
                    }
                }
                None => vk::RenderingAttachmentInfo::default()
                    .image_view(vk::ImageView::null())
                    .image_layout(vk::ImageLayout::UNDEFINED)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .load_op(vk::AttachmentLoadOp::DONT_CARE),
            })
            .collect();
        let depth_attachment_info = descriptor
            .depth_attachment
            .as_ref()
            .map(|depth_attachment| {
                let info = vk::RenderingAttachmentInfo::default()
                    .image_view(VulkanGpuTexture::image_view(&depth_attachment.texture_view))
                    .image_layout(vk::ImageLayout::GENERAL)
                    .store_op(vk::AttachmentStoreOp::STORE);
                match depth_attachment.clear_value {
                    Some(depth) => {
                        info.load_op(vk::AttachmentLoadOp::CLEAR)
                            .clear_value(vk::ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: depth as f32,
                                    stencil: 0,
                                },
                            })
                    }
                    None => info.load_op(vk::AttachmentLoadOp::LOAD),
                }
            });
        let mut rendering_info = vk::RenderingInfo::default()
            .render_area(vk_render_area)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&color_attachment_info);
        if let Some(depth_attachment_info) = &depth_attachment_info {
            rendering_info = rendering_info.depth_attachment(depth_attachment_info);
        }
        // SAFETY: the buffer is recording outside a pass, and the views are the device's.
        unsafe {
            device
                .dynamic_rendering()
                .cmd_begin_rendering(command_buffer, &rendering_info)
        };
        *self.current_render_pass.borrow_mut() = Some(descriptor.label.clone());

        let backend = VulkanRenderPass::new(
            device.clone(),
            self,
            command_buffer,
            render_area,
            width,
            height,
            descriptor.depth_attachment.is_some(),
        );
        let formats = descriptor
            .color_attachments
            .iter()
            .map(|attachment| {
                attachment
                    .as_ref()
                    .map(|attachment| attachment.texture_view.texture().get_format())
            })
            .collect();
        Box::new(FrontendRenderPass::new(
            Box::new(backend),
            device_info.features,
            device_info.limits,
            formats,
            Box::new(|| self.submit_render_pass()),
            render_area,
        ))
    }

    fn clear_color_texture(&self, color_texture: &dyn GpuTexture, clear_color: &Vector4f) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        let device = self.device();
        self.clear_color_texture_unsynced(&device, color_texture, clear_color);
        self.memory_barrier_frame(&device);
    }

    fn clear_color_and_depth_textures(
        &self,
        color_texture: &dyn GpuTexture,
        clear_color: &Vector4f,
        depth_texture: &dyn GpuTexture,
        clear_depth: f64,
    ) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        let device = self.device();
        self.clear_color_texture_unsynced(&device, color_texture, clear_color);
        self.clear_depth_texture_unsynced(&device, depth_texture, clear_depth);
        self.memory_barrier_frame(&device);
    }

    fn clear_depth_texture(&self, depth_texture: &dyn GpuTexture, clear_depth: f64) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        let device = self.device();
        self.clear_depth_texture_unsynced(&device, depth_texture, clear_depth);
        self.memory_barrier_frame(&device);
    }

    /// `writeToBuffer`: the data copied from staging memory. Panics where Java's frontend throws.
    fn write_to_buffer(&self, slice: &GpuBufferSlice, data: &[u8]) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        FrontendCommandEncoder::check_write_to_buffer(slice, data.len() as i64);
        let device = self.device();
        let staging_buffer = self.transient_memory.upload_staging(&[data], 1, 16);
        let regions = [vk::BufferCopy::default()
            .src_offset(staging_buffer.offset as u64)
            .dst_offset(slice.offset as u64)
            .size(data.len() as u64)];
        // SAFETY: the buffer is recording, and both buffers are the device's.
        unsafe {
            device.vk_device().cmd_copy_buffer(
                self.command_buffer(),
                VulkanGpuBuffer::of(&*staging_buffer.buffer).vk_buffer(),
                VulkanGpuBuffer::of(&*slice.buffer).vk_buffer(),
                &regions,
            );
        }
        self.memory_barrier_frame(&device);
    }

    /// Panics where Java's frontend throws.
    fn write_to_texture_image(
        &self,
        destination: &dyn GpuTexture,
        source: &NativeImage,
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
    ) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        FrontendCommandEncoder::check_write_to_texture_image(
            destination,
            source,
            mip_level,
            depth_or_layer,
            dest_x,
            dest_y,
        );
        self.write_to_texture(
            destination,
            &source.get_pixel_bytes(),
            mip_level,
            depth_or_layer,
            dest_x,
            dest_y,
            source.get_width(),
            source.get_height(),
        );
    }

    /// `writeToTexture`: the bytes copied from staging memory. Panics where Java's frontend throws.
    fn write_to_texture(
        &self,
        destination: &dyn GpuTexture,
        source: &[u8],
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
        width: i32,
        height: i32,
    ) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        FrontendCommandEncoder::check_write_to_texture(
            destination,
            source.len(),
            mip_level,
            depth_or_layer,
            dest_x,
            dest_y,
            width,
            height,
        );
        let device = self.device();
        let staging_buffer = self.transient_memory.upload_staging(&[source], 1, 16);
        let region = [vk::BufferImageCopy {
            buffer_offset: staging_buffer.offset as u64,
            buffer_row_length: width as u32,
            buffer_image_height: height as u32,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: mip_level as u32,
                base_array_layer: depth_or_layer as u32,
                layer_count: 1,
            },
            image_offset: vk::Offset3D {
                x: dest_x,
                y: dest_y,
                z: 0,
            },
            image_extent: vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        }];
        // SAFETY: the buffer is recording, and the staging buffer and the image are the device's.
        unsafe {
            device.vk_device().cmd_copy_buffer_to_image(
                self.command_buffer(),
                VulkanGpuBuffer::of(&*staging_buffer.buffer).vk_buffer(),
                VulkanGpuTexture::of(destination).vk_image(),
                vk::ImageLayout::GENERAL,
                &region,
            );
        }
        self.memory_barrier_frame(&device);
    }

    fn copy_buffer_to_texture(
        &self,
        source: &GpuBufferSlice,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        destination: &dyn GpuTexture,
        destination_x: i32,
        destination_y: i32,
        copy_width: i32,
        copy_height: i32,
        mip_level: i32,
        array_layer: i32,
    ) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        let device = self.device();
        let texel_size = i64::from(destination.get_format().block_size());
        let skip_texels = i64::from(source_x) + i64::from(source_y) * i64::from(source_width);
        let skip_bytes = skip_texels * texel_size;
        let region = [vk::BufferImageCopy {
            buffer_offset: (source.offset + skip_bytes) as u64,
            buffer_row_length: source_width as u32,
            buffer_image_height: source_height as u32,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: mip_level as u32,
                base_array_layer: array_layer as u32,
                layer_count: 1,
            },
            image_offset: vk::Offset3D {
                x: destination_x,
                y: destination_y,
                z: 0,
            },
            image_extent: vk::Extent3D {
                width: copy_width as u32,
                height: copy_height as u32,
                depth: 1,
            },
        }];
        // SAFETY: as above.
        unsafe {
            device.vk_device().cmd_copy_buffer_to_image(
                self.command_buffer(),
                VulkanGpuBuffer::of(&*source.buffer).vk_buffer(),
                VulkanGpuTexture::of(destination).vk_image(),
                vk::ImageLayout::GENERAL,
                &region,
            );
        }
        self.memory_barrier_frame(&device);
    }

    /// The whole mip level copied into the buffer; the callback runs when the destruction queue
    /// comes round to this submit again, two submits on, once the GPU is done with the copy.
    fn copy_texture_to_buffer(
        &self,
        source: &dyn GpuTexture,
        destination: &Rc<dyn GpuBuffer>,
        offset: i64,
        callback: Box<dyn FnOnce()>,
        mip_level: i32,
    ) {
        FrontendCommandEncoder::check_not_in_render_pass(
            self.current_render_pass.borrow().is_some(),
        );
        let device = self.device();
        let width = source.get_width(mip_level);
        let height = source.get_height(mip_level);
        let copy = [vk::BufferImageCopy {
            buffer_offset: offset as u64,
            buffer_row_length: width as u32,
            buffer_image_height: height as u32,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::from_raw(VulkanConst::format_aspect_mask(
                    source.get_format(),
                ) as u32),
                mip_level: mip_level as u32,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        }];
        // SAFETY: as above.
        unsafe {
            device.vk_device().cmd_copy_image_to_buffer(
                self.command_buffer(),
                VulkanGpuTexture::of(source).vk_image(),
                vk::ImageLayout::GENERAL,
                VulkanGpuBuffer::of(&**destination).vk_buffer(),
                &copy,
            );
        }
        self.memory_barrier_frame(&device);
        self.queue_for_destroy(callback);
    }
}
