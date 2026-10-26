//! com/mojang/renderpearl/backend/vulkan/VulkanCommandPool.java
//!
//! A pool of primary command buffers for one queue's family, allocated 32 at a time and handed
//! out again after `reset`. Java's `PointerBuffer` of them and its position are a list and an
//! index here.

use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk;
use std::cell::{Cell, RefCell};

const BUFFER_ALLOC_COUNT: u32 = 32;

pub struct VulkanCommandPool {
    command_pool: vk::CommandPool,
    allocated_buffers: RefCell<Vec<vk::CommandBuffer>>,
    position: Cell<usize>,
}

impl VulkanCommandPool {
    pub fn new(device: &ash::Device, queue_family_index: u32) -> VulkanCommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family_index);
        // SAFETY: the create info lives for the call.
        let command_pool = VulkanUtils::crash_if_err(
            unsafe { device.create_command_pool(&command_pool_create_info, None) },
            "Failed to create VkCommandPool",
        );
        VulkanCommandPool {
            command_pool,
            allocated_buffers: RefCell::new(Vec::new()),
            position: Cell::new(0),
        }
    }

    /// `destroy()`.
    pub fn destroy(&self, device: &ash::Device) {
        self.release(device);
        // SAFETY: the pool's buffers are done.
        unsafe { device.destroy_command_pool(self.command_pool, None) };
    }

    /// `release()`: every buffer freed, and the pool's memory given back.
    pub fn release(&self, device: &ash::Device) {
        let mut allocated_buffers = self.allocated_buffers.borrow_mut();
        // SAFETY: the buffers are the pool's, and done.
        unsafe {
            if !allocated_buffers.is_empty() {
                device.free_command_buffers(self.command_pool, &allocated_buffers);
                allocated_buffers.clear();
            }
            let _ = device.reset_command_pool(
                self.command_pool,
                vk::CommandPoolResetFlags::RELEASE_RESOURCES,
            );
        }
        self.position.set(0);
    }

    /// `reset()`: every buffer back to the initial state, to be handed out again.
    pub fn reset(&self, device: &ash::Device) {
        // SAFETY: the pool's buffers are done.
        let _ = unsafe {
            device.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
        };
        self.position.set(0);
    }

    fn allocate_more_buffers(&self, device: &ash::Device) {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(BUFFER_ALLOC_COUNT);
        // SAFETY: the allocate info lives for the call.
        let buffers = VulkanUtils::crash_if_err(
            unsafe { device.allocate_command_buffers(&allocate_info) },
            "Failed to allocate VkCommandBuffers",
        );
        self.allocated_buffers.borrow_mut().extend(buffers);
    }

    pub fn allocate_buffer(&self, device: &ash::Device) -> vk::CommandBuffer {
        if self.position.get() == self.allocated_buffers.borrow().len() {
            self.allocate_more_buffers(device);
        }
        let buffer = self.allocated_buffers.borrow()[self.position.get()];
        self.position.set(self.position.get() + 1);
        buffer
    }
}
