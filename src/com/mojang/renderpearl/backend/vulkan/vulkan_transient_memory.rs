//! com/mojang/renderpearl/backend/vulkan/VulkanTransientMemory.java
//!
//! A submit's short-lived memory: staging buffers that new buffers' data, buffer writes and
//! texture writes are copied from, in 512 KiB blocks of host-visible memory from VMA, mapped once
//! and reused two submits later (`TransientBlockAllocator`). The heaps are chosen like Java's: the
//! largest device-local heap, and for staging a host-visible one that isn't device-local unless the
//! largest device-local heap is host-visible too (an integrated GPU's, lavapipe's).
//!
//! Only staging is ported: the CPU, GPU and mapped GPU memory (`allocateCpu`, `allocateGpu`,
//! `allocateGpuMapped`, `uploadGpu`, `multiUpload…`) serve the API's `TransientMemory`, which the
//! port's frame doesn't use yet (it makes buffers of its own each frame), and their rotations at
//! each submit's end do nothing while they're unused, nor does the copy they'd record.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::backend::util::transient_block_allocator::{
    Allocation, Allocator, Block, TransientBlockAllocator,
};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_command_encoder::VulkanCommandEncoder;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_buffer::VulkanGpuBuffer;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::net::minecraft::util::mth::Mth;
use crate::org::lwjgl::util::vma::vma::{
    Vma, VmaAllocationCreateInfo, VmaAllocationInfo, VmaAllocator,
};
use ash::vk;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const BLOCK_SIZE: i64 = 524288;
/// `Long.highestOneBit(Long.MAX_VALUE)`.
const MAX_GPU_ALIGNMENT: i64 = 1 << 62;
/// Transfers both ways, uniform and storage and texel buffers, index, vertex and indirect.
const BUFFER_USAGE_BITS: u32 = 471;

pub struct VulkanTransientMemory {
    use_device_memory_for_mapped_gpu_staging: bool,
    staging_block_allocator: Rc<RefCell<TransientBlockAllocator<VulkanAllocation>>>,
    submit_index: Rc<Cell<u64>>,
    command_buffer: Cell<Option<vk::CommandBuffer>>,
}

/// `VulkanTransientMemory.VulkanAllocation`.
#[derive(Clone, Copy, Debug)]
pub struct VulkanAllocation {
    vk_buffer: vk::Buffer,
    vma_allocation: i64,
    host_ptr: *mut u8,
    size: i64,
    suboptimal: bool,
}

impl Block for VulkanAllocation {
    fn suboptimal(&self) -> bool {
        self.suboptimal
    }
}

/// `allocateVulkanBlock` and `freeVulkanBlock` for staging blocks.
struct StagingBlocks {
    vma: VmaAllocator,
    use_device_memory_for_mapped_gpu_staging: bool,
    expected_cpu_memory_heap: u32,
    expected_gpu_memory_heap: u32,
    memory_type_to_heap_map: [u32; 32],
}

impl Allocator<VulkanAllocation> for StagingBlocks {
    fn alloc(&mut self, size: i64) -> VulkanAllocation {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(vk::BufferUsageFlags::from_raw(BUFFER_USAGE_BITS))
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let mut alloc_create_info = VmaAllocationCreateInfo::default();
        let expected_heap = if !self.use_device_memory_for_mapped_gpu_staging {
            alloc_create_info.usage = Vma::VMA_MEMORY_USAGE_AUTO_PREFER_HOST;
            self.expected_cpu_memory_heap
        } else {
            alloc_create_info.usage = Vma::VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE;
            self.expected_gpu_memory_heap
        };
        // Staging: host visible and coherent, written in order.
        alloc_create_info.required_flags = 6;
        alloc_create_info.flags = Vma::VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT;

        let mut allocation_info = VmaAllocationInfo::default();
        let (result, buffer, allocation) = Vma::vmaCreateBuffer(
            self.vma,
            &buffer_create_info,
            &alloc_create_info,
            Some(&mut allocation_info),
        );
        VulkanUtils::crash_if_failure(result, "Failed to allocate VkBuffer");
        let (result, host_ptr) = Vma::vmaMapMemory(self.vma, allocation);
        VulkanUtils::crash_if_failure(result, "Failed to map buffer");
        let allocated_heap = self.memory_type_to_heap_map[allocation_info.memory_type as usize];
        VulkanAllocation {
            vk_buffer: buffer,
            vma_allocation: allocation,
            host_ptr,
            size,
            suboptimal: allocated_heap != expected_heap,
        }
    }

    fn free(&mut self, allocation: VulkanAllocation) {
        Vma::vmaDestroyBuffer(self.vma, allocation.vk_buffer, allocation.vma_allocation);
    }
}

impl VulkanTransientMemory {
    pub fn new(device: &VulkanDevice) -> VulkanTransientMemory {
        let instance = device.instance().vk_instance();
        // SAFETY: the device is the instance's.
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device.physical_device()) };
        let heap_count = memory_properties.memory_heap_count as usize;
        let type_count = memory_properties.memory_type_count as usize;
        let mut memory_type_to_heap_map = [0u32; 32];
        for (i, slot) in memory_type_to_heap_map
            .iter_mut()
            .enumerate()
            .take(type_count)
        {
            *slot = memory_properties.memory_types[i].heap_index;
        }

        let mut largest_device_local_heap_index = -1i32;
        let mut largest_device_local_heap_size = -1i64;
        for i in 0..heap_count {
            let heap_properties = memory_properties.memory_heaps[i];
            if VulkanUtils::has_any_bit(i64::from(heap_properties.flags.as_raw()), 1)
                && heap_properties.size as i64 >= largest_device_local_heap_size
            {
                largest_device_local_heap_index = i as i32;
                largest_device_local_heap_size = heap_properties.size as i64;
            }
        }
        assert!(largest_device_local_heap_index != -1);

        let largest_heap_is_host_visible_and_coherent = memory_properties.memory_types
            [..type_count]
            .iter()
            .any(|type_properties| {
                type_properties.heap_index as i32 == largest_device_local_heap_index
                    && VulkanUtils::has_all_bits(
                        i64::from(type_properties.property_flags.as_raw()),
                        6,
                    )
            });
        let use_device_memory_for_mapped_gpu_staging = largest_heap_is_host_visible_and_coherent;
        let mut selected_cpu_heap = -1i32;
        if use_device_memory_for_mapped_gpu_staging {
            selected_cpu_heap = largest_device_local_heap_index;
        } else {
            for type_properties in &memory_properties.memory_types[..type_count] {
                if VulkanUtils::has_all_bits(i64::from(type_properties.property_flags.as_raw()), 6)
                {
                    let heap_properties =
                        memory_properties.memory_heaps[type_properties.heap_index as usize];
                    if !VulkanUtils::has_any_bit(i64::from(heap_properties.flags.as_raw()), 1) {
                        selected_cpu_heap = type_properties.heap_index as i32;
                        break;
                    }
                }
            }
        }
        assert!(
            selected_cpu_heap != -1,
            "Could not select heap for CPU allocations"
        );

        let staging_blocks = StagingBlocks {
            vma: device.vma(),
            use_device_memory_for_mapped_gpu_staging,
            expected_cpu_memory_heap: selected_cpu_heap as u32,
            expected_gpu_memory_heap: largest_device_local_heap_index as u32,
            memory_type_to_heap_map,
        };
        VulkanTransientMemory {
            use_device_memory_for_mapped_gpu_staging,
            staging_block_allocator: Rc::new(RefCell::new(TransientBlockAllocator::new(
                BLOCK_SIZE,
                MAX_GPU_ALIGNMENT,
                Box::new(staging_blocks),
            ))),
            submit_index: Rc::new(Cell::new(0)),
            command_buffer: Cell::new(None),
        }
    }

    pub fn use_device_memory_for_mapped_gpu_staging(&self) -> bool {
        self.use_device_memory_for_mapped_gpu_staging
    }

    /// `destroy()`.
    pub fn destroy(&self) {
        self.staging_block_allocator.borrow_mut().close();
    }

    /// `beginSubmit()`: the command buffer the submit's mapped copies would go in, first in it.
    pub fn begin_submit(&self, encoder: &VulkanCommandEncoder) {
        assert!(self.command_buffer.get().is_none());
        let command_buffer = encoder.allocate_and_begin_transient_command_buffer();
        encoder.execute(command_buffer);
        self.command_buffer.set(Some(command_buffer));
    }

    /// `endSubmit()`: the staging blocks given back once the GPU is done with them (the encoder's
    /// destruction queue), and the command buffer ended.
    pub fn end_submit(&self, encoder: &VulkanCommandEncoder, device: &ash::Device) {
        let rotation = self.staging_block_allocator.borrow_mut().rotate();
        let allocator = self.staging_block_allocator.clone();
        encoder.queue_for_destroy(Box::new(move || {
            allocator.borrow_mut().finish_rotation(rotation)
        }));
        let command_buffer = self
            .command_buffer
            .take()
            .expect("a submit's command buffer");
        // SAFETY: the buffer is recording.
        let _ = unsafe { device.end_command_buffer(command_buffer) };
        self.submit_index.set(self.submit_index.get() + 1);
    }

    /// `allocateStaging`: a slice of a staging block, and where it's mapped.
    pub fn allocate_staging(
        &self,
        size: i64,
        alignment: i64,
        usage: i32,
        minimum_allocation: i64,
        element_size: i64,
    ) -> (GpuBufferSlice, *mut u8) {
        assert!(size <= i64::from(i32::MAX));
        let Allocation {
            block,
            offset,
            size,
        } = self.staging_block_allocator.borrow_mut().allocate(
            size,
            alignment,
            minimum_allocation,
            element_size,
        );
        let api_buffer = VulkanGpuBuffer::transient(
            block.vk_buffer,
            usage,
            block.size as i32,
            self.submit_index.get(),
            self.submit_index.clone(),
        );
        // SAFETY: the block is mapped, and the allocation is inside it.
        let cpu_buffer = unsafe { block.host_ptr.add(offset as usize) };
        (
            GpuBufferSlice::new(Rc::new(api_buffer), offset, size),
            cpu_buffer,
        )
    }

    /// `uploadStaging(List, alignment, usage)`: the data one after another, each at the alignment,
    /// in staging memory.
    pub fn upload_staging(&self, data: &[&[u8]], alignment: i64, usage: i32) -> GpuBufferSlice {
        let mut total_size = 0;
        for buffer in data {
            total_size += buffer.len() as i64;
            total_size = Mth::round_toward_i64(total_size, alignment);
        }

        // `minimumAllocation` defaults to the whole size, and `elementSize` to 1.
        let (slice, mapped_ptr) =
            self.allocate_staging(total_size, alignment, usage, total_size, 1);
        let mut offset = 0;
        for buffer in data {
            let length = (slice.length - offset).min(buffer.len() as i64);
            // SAFETY: the mapped allocation holds the slice's length from its start.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    buffer.as_ptr(),
                    mapped_ptr.add(offset as usize),
                    length as usize,
                );
            }
            offset += buffer.len() as i64;
            offset = Mth::round_toward_i64(offset, alignment);
            if offset >= slice.length {
                break;
            }
        }
        slice
    }
}

impl std::fmt::Debug for VulkanTransientMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanTransientMemory")
            .field("submit_index", &self.submit_index.get())
            .finish()
    }
}
