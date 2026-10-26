//! com/mojang/renderpearl/backend/vulkan/VulkanGpuBuffer.java
//!
//! A `VkBuffer`: `Direct`, the device's own from VMA, whose memory is host visible where it's
//! mapped (read back or written), or a slice of the transient memory (`VulkanTransientMemory`'s
//! `TransientGpuBuffer`), which the next submit closes. A direct buffer's destruction is queued on
//! the encoder when it's dropped (Java's `close()`), for once the GPU is done with it. Mapping
//! hands out a copy of the bytes it reads, as the API's `map_read` does.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::org::lwjgl::util::vma::vma::{Vma, VmaAllocation, VmaAllocationCreateInfo};
use ash::vk::{self, Handle};
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

pub struct VulkanGpuBuffer {
    vk_buffer: vk::Buffer,
    usage: i32,
    size: i64,
    kind: Kind,
}

enum Kind {
    /// `VulkanGpuBuffer.Direct`.
    Direct {
        device: Rc<VulkanDevice>,
        vma_allocation: VmaAllocation,
        mapping_ref_count: Cell<i32>,
    },
    /// `VulkanTransientMemory.TransientGpuBuffer`: closed once the transient memory's submit
    /// index has moved past its own.
    Transient {
        buffer_submit_index: u64,
        current_submit_index: Rc<Cell<u64>>,
    },
}

impl VulkanGpuBuffer {
    /// `new Direct(device, label, usage, size, forceHostVisibleAllocation)`.
    pub fn direct(
        device: &Rc<VulkanDevice>,
        label: Option<&str>,
        usage: i32,
        size: i64,
        force_host_visible_allocation: bool,
    ) -> VulkanGpuBuffer {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(vk::BufferUsageFlags::from_raw(
                VulkanConst::buffer_usage_to_vk(usage) as u32,
            ))
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let mut alloc_create_info = VmaAllocationCreateInfo {
            usage: Vma::VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE,
            ..VmaAllocationCreateInfo::default()
        };
        if force_host_visible_allocation {
            alloc_create_info.required_flags |= 2;
        }
        if VulkanUtils::has_any_bit(i64::from(usage), 3) {
            alloc_create_info.required_flags |= 2 | 4;
            if VulkanUtils::has_any_bit(i64::from(usage), 1) {
                alloc_create_info.preferred_flags |= 8;
                alloc_create_info.flags |= Vma::VMA_ALLOCATION_CREATE_HOST_ACCESS_RANDOM_BIT;
            } else {
                alloc_create_info.flags |=
                    Vma::VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT;
            }
        }
        if VulkanUtils::has_any_bit(i64::from(usage), 5) {
            alloc_create_info.usage = Vma::VMA_MEMORY_USAGE_AUTO_PREFER_HOST;
        }

        let (result, vk_buffer, vma_allocation) =
            Vma::vmaCreateBuffer(device.vma(), &buffer_create_info, &alloc_create_info, None);
        VulkanUtils::crash_if_failure(result, "Failed to allocate VkBuffer");
        if let Some(label) = label {
            device.instance().debug().set_object_name(
                vk::ObjectType::BUFFER,
                vk_buffer.as_raw(),
                label,
            );
        }
        VulkanGpuBuffer {
            vk_buffer,
            usage,
            size,
            kind: Kind::Direct {
                device: device.clone(),
                vma_allocation,
                mapping_ref_count: Cell::new(0),
            },
        }
    }

    /// `new TransientGpuBuffer(vkBuffer, usage, size, bufferSubmitIndex)`.
    pub fn transient(
        vk_buffer: vk::Buffer,
        usage: i32,
        size: i32,
        buffer_submit_index: u64,
        current_submit_index: Rc<Cell<u64>>,
    ) -> VulkanGpuBuffer {
        VulkanGpuBuffer {
            vk_buffer,
            usage,
            size: i64::from(size),
            kind: Kind::Transient {
                buffer_submit_index,
                current_submit_index,
            },
        }
    }

    /// `(VulkanGpuBuffer) buffer`. Panics where Java throws `ClassCastException`.
    pub fn of(buffer: &dyn GpuBuffer) -> &VulkanGpuBuffer {
        buffer
            .as_any()
            .downcast_ref::<VulkanGpuBuffer>()
            .expect("a Vulkan buffer")
    }

    pub fn vk_buffer(&self) -> vk::Buffer {
        self.vk_buffer
    }

    /// `isClosed()`: a transient buffer once its submit is over.
    pub fn is_closed(&self) -> bool {
        match &self.kind {
            Kind::Direct { .. } => false,
            Kind::Transient {
                buffer_submit_index,
                current_submit_index,
            } => *buffer_submit_index < current_submit_index.get(),
        }
    }
}

impl GpuBuffer for VulkanGpuBuffer {
    fn size(&self) -> i64 {
        self.size
    }

    fn usage(&self) -> i32 {
        self.usage
    }

    /// `map(0, size, true, false)`, read and unmapped. Panics where Java throws: a transient
    /// buffer, or one that isn't readable.
    fn map_read(&self) -> Vec<u8> {
        let Kind::Direct {
            device,
            vma_allocation,
            mapping_ref_count,
        } = &self.kind
        else {
            panic!("Cannot map transient buffer");
        };
        assert!(
            self.usage & gpu_buffer::USAGE_MAP_READ != 0,
            "Buffer is not readable"
        );
        mapping_ref_count.set(mapping_ref_count.get() + 1);
        let (result, pointer) = Vma::vmaMapMemory(device.vma(), *vma_allocation);
        VulkanUtils::crash_if_failure(result, "Failed to map buffer");
        // SAFETY: the mapping holds the buffer's size from its start.
        let bytes = unsafe { std::slice::from_raw_parts(pointer, self.size as usize) }.to_vec();
        mapping_ref_count.set(mapping_ref_count.get() - 1);
        Vma::vmaUnmapMemory(device.vma(), *vma_allocation);
        bytes
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for VulkanGpuBuffer {
    /// `close()`: a direct buffer's destruction queued for once the GPU is done with it.
    fn drop(&mut self) {
        if let Kind::Direct {
            device,
            vma_allocation,
            mapping_ref_count,
        } = &self.kind
        {
            assert!(
                mapping_ref_count.get() == 0,
                "Attempt to close a mapped buffer"
            );
            let vma = device.vma();
            let vk_buffer = self.vk_buffer;
            let vma_allocation = *vma_allocation;
            device
                .create_command_encoder()
                .queue_for_destroy(Box::new(move || {
                    Vma::vmaDestroyBuffer(vma, vk_buffer, vma_allocation)
                }));
        }
    }
}
