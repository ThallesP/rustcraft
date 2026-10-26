//! org/lwjgl/util/vma/Vma.java and LibVma.java (LWJGL 3.4.3)
//!
//! The Vulkan Memory Allocator vanilla's Vulkan backend allocates its images and buffers with,
//! from LWJGL's own `liblwjgl_vma.so`, so the port's allocations are VMA's very same choices of
//! memory type, block and offset. Like stb (`LibSTB`), the library is LWJGL's JNI build: each
//! function is its JNI entry point (`Java_org_lwjgl_util_vma_Vma_nvmaCreateBuffer`), which takes
//! the environment and class first, never read, and its pointers as `jlong`s, and VMA's own
//! allocations go through the function pointers `LibVma.setupMalloc` hands it, here the C
//! library's. The structs are LWJGL's layouts (`VmaVulkanFunctions` has LWJGL's extra
//! `vkGetPhysicalDeviceProperties2KHR` last), checked by their sizes below.

use crate::org::lwjgl::stb::lib_stb::NO_ENV;
use crate::org::lwjgl::system::shared_library_loader;
use ash::vk::{self, Handle};
use std::ffi::c_void;
use std::sync::Once;

shared_library_loader::functions! {
    "vma", "lwjgl_vma";
    Java_org_lwjgl_util_vma_LibVma_setupMalloc: fn(*mut c_void, *mut c_void, i64, i64, i64, i64, i64, i64);
    Java_org_lwjgl_util_vma_Vma_nvmaCreateAllocator: fn(*mut c_void, *mut c_void, i64, i64) -> i32;
    Java_org_lwjgl_util_vma_Vma_nvmaDestroyAllocator: fn(*mut c_void, *mut c_void, i64);
    Java_org_lwjgl_util_vma_Vma_nvmaCreateBuffer: fn(*mut c_void, *mut c_void, i64, i64, i64, i64, i64, i64) -> i32;
    Java_org_lwjgl_util_vma_Vma_nvmaDestroyBuffer: fn(*mut c_void, *mut c_void, i64, i64, i64);
    Java_org_lwjgl_util_vma_Vma_nvmaCreateImage: fn(*mut c_void, *mut c_void, i64, i64, i64, i64, i64, i64) -> i32;
    Java_org_lwjgl_util_vma_Vma_nvmaDestroyImage: fn(*mut c_void, *mut c_void, i64, i64, i64);
    Java_org_lwjgl_util_vma_Vma_nvmaMapMemory: fn(*mut c_void, *mut c_void, i64, i64, i64) -> i32;
    Java_org_lwjgl_util_vma_Vma_nvmaUnmapMemory: fn(*mut c_void, *mut c_void, i64, i64);
}

/// `VmaAllocator`.
pub type VmaAllocator = i64;
/// `VmaAllocation`.
pub type VmaAllocation = i64;

/// `VmaVulkanFunctions`: the Vulkan functions VMA calls, in LWJGL's order.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VmaVulkanFunctions {
    pub functions: [*const c_void; 28],
}

/// The names of `VmaVulkanFunctions`' members, in order, as the loader names the functions.
/// LWJGL's `set(VkInstance, VkDevice)` fills the `KHR` ones with Vulkan 1.1's core functions.
pub const VULKAN_FUNCTION_NAMES: [&str; 28] = [
    "vkGetInstanceProcAddr",
    "vkGetDeviceProcAddr",
    "vkGetPhysicalDeviceProperties",
    "vkGetPhysicalDeviceMemoryProperties",
    "vkAllocateMemory",
    "vkFreeMemory",
    "vkMapMemory",
    "vkUnmapMemory",
    "vkFlushMappedMemoryRanges",
    "vkInvalidateMappedMemoryRanges",
    "vkBindBufferMemory",
    "vkBindImageMemory",
    "vkGetBufferMemoryRequirements",
    "vkGetImageMemoryRequirements",
    "vkCreateBuffer",
    "vkDestroyBuffer",
    "vkCreateImage",
    "vkDestroyImage",
    "vkCmdCopyBuffer",
    "vkGetBufferMemoryRequirements2",
    "vkGetImageMemoryRequirements2",
    "vkBindBufferMemory2",
    "vkBindImageMemory2",
    "vkGetPhysicalDeviceMemoryProperties2",
    "vkGetDeviceBufferMemoryRequirements",
    "vkGetDeviceImageMemoryRequirements",
    "vkGetMemoryWin32HandleKHR",
    "vkGetPhysicalDeviceProperties2",
];

/// `VmaAllocatorCreateInfo`.
#[repr(C)]
pub struct VmaAllocatorCreateInfo {
    pub flags: u32,
    pub physical_device: vk::PhysicalDevice,
    pub device: vk::Device,
    pub preferred_large_heap_block_size: u64,
    pub p_allocation_callbacks: *const c_void,
    pub p_device_memory_callbacks: *const c_void,
    pub p_heap_size_limit: *const u64,
    pub p_vulkan_functions: *const VmaVulkanFunctions,
    pub instance: vk::Instance,
    pub vulkan_api_version: u32,
    pub p_type_external_memory_handle_types: *const u32,
}

/// `VmaAllocationCreateInfo`, `calloc`'d: every field zero until set.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VmaAllocationCreateInfo {
    pub flags: u32,
    pub usage: u32,
    pub required_flags: u32,
    pub preferred_flags: u32,
    pub memory_type_bits: u32,
    pub pool: i64,
    pub p_user_data: i64,
    pub priority: f32,
    pub min_alignment: u64,
}

/// `VmaAllocationInfo`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VmaAllocationInfo {
    pub memory_type: u32,
    pub device_memory: u64,
    pub offset: u64,
    pub size: u64,
    pub p_mapped_data: i64,
    pub p_user_data: i64,
    pub p_name: i64,
}

const _: () = {
    assert!(size_of::<VmaVulkanFunctions>() == 224);
    assert!(size_of::<VmaAllocatorCreateInfo>() == 88);
    assert!(size_of::<VmaAllocationCreateInfo>() == 56);
    assert!(size_of::<VmaAllocationInfo>() == 56);
};

pub struct Vma;

static INITIALIZED: Once = Once::new();

// SAFETY (all): the allocator, allocations, buffers and images passed in are live ones VMA made,
// and the create infos point at live structs for the call.
#[allow(non_snake_case)]
impl Vma {
    pub const VMA_MEMORY_USAGE_AUTO: u32 = 7;
    pub const VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE: u32 = 8;
    pub const VMA_MEMORY_USAGE_AUTO_PREFER_HOST: u32 = 9;
    pub const VMA_ALLOCATION_CREATE_HOST_ACCESS_SEQUENTIAL_WRITE_BIT: u32 = 1024;
    pub const VMA_ALLOCATION_CREATE_HOST_ACCESS_RANDOM_BIT: u32 = 2048;

    /// `LibVma.initialize()`, which `Vma`'s static initialiser runs first.
    fn initialize() {
        INITIALIZED.call_once(|| {
            // SAFETY: the C library's allocator functions, as `LibSTB` hands stb them.
            unsafe {
                (FUNCTIONS.Java_org_lwjgl_util_vma_LibVma_setupMalloc)(
                    NO_ENV,
                    NO_ENV,
                    libc::malloc as *const () as i64,
                    libc::calloc as *const () as i64,
                    libc::realloc as *const () as i64,
                    libc::free as *const () as i64,
                    libc::aligned_alloc as *const () as i64,
                    libc::free as *const () as i64,
                );
            }
        });
    }

    pub fn vmaCreateAllocator(create_info: &VmaAllocatorCreateInfo) -> (vk::Result, VmaAllocator) {
        Vma::initialize();
        let mut allocator: VmaAllocator = 0;
        let result = unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaCreateAllocator)(
                NO_ENV,
                NO_ENV,
                create_info as *const _ as i64,
                &mut allocator as *mut _ as i64,
            )
        };
        (vk::Result::from_raw(result), allocator)
    }

    pub fn vmaDestroyAllocator(allocator: VmaAllocator) {
        unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaDestroyAllocator)(NO_ENV, NO_ENV, allocator)
        }
    }

    pub fn vmaCreateBuffer(
        allocator: VmaAllocator,
        buffer_create_info: &vk::BufferCreateInfo<'_>,
        allocation_create_info: &VmaAllocationCreateInfo,
        allocation_info: Option<&mut VmaAllocationInfo>,
    ) -> (vk::Result, vk::Buffer, VmaAllocation) {
        let mut buffer = 0u64;
        let mut allocation: VmaAllocation = 0;
        let result = unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaCreateBuffer)(
                NO_ENV,
                NO_ENV,
                allocator,
                buffer_create_info as *const _ as i64,
                allocation_create_info as *const _ as i64,
                &mut buffer as *mut _ as i64,
                &mut allocation as *mut _ as i64,
                allocation_info.map_or(0, |info| info as *mut _ as i64),
            )
        };
        (
            vk::Result::from_raw(result),
            vk::Buffer::from_raw(buffer),
            allocation,
        )
    }

    pub fn vmaDestroyBuffer(
        allocator: VmaAllocator,
        buffer: vk::Buffer,
        allocation: VmaAllocation,
    ) {
        unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaDestroyBuffer)(
                NO_ENV,
                NO_ENV,
                allocator,
                buffer.as_raw() as i64,
                allocation,
            )
        }
    }

    pub fn vmaCreateImage(
        allocator: VmaAllocator,
        image_create_info: &vk::ImageCreateInfo<'_>,
        allocation_create_info: &VmaAllocationCreateInfo,
    ) -> (vk::Result, vk::Image, VmaAllocation) {
        let mut image = 0u64;
        let mut allocation: VmaAllocation = 0;
        let result = unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaCreateImage)(
                NO_ENV,
                NO_ENV,
                allocator,
                image_create_info as *const _ as i64,
                allocation_create_info as *const _ as i64,
                &mut image as *mut _ as i64,
                &mut allocation as *mut _ as i64,
                0,
            )
        };
        (
            vk::Result::from_raw(result),
            vk::Image::from_raw(image),
            allocation,
        )
    }

    pub fn vmaDestroyImage(allocator: VmaAllocator, image: vk::Image, allocation: VmaAllocation) {
        unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaDestroyImage)(
                NO_ENV,
                NO_ENV,
                allocator,
                image.as_raw() as i64,
                allocation,
            )
        }
    }

    /// The allocation's memory, mapped: VMA counts the maps, so each needs an unmap.
    pub fn vmaMapMemory(
        allocator: VmaAllocator,
        allocation: VmaAllocation,
    ) -> (vk::Result, *mut u8) {
        let mut data: *mut u8 = std::ptr::null_mut();
        let result = unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaMapMemory)(
                NO_ENV,
                NO_ENV,
                allocator,
                allocation,
                &mut data as *mut _ as i64,
            )
        };
        (vk::Result::from_raw(result), data)
    }

    pub fn vmaUnmapMemory(allocator: VmaAllocator, allocation: VmaAllocation) {
        unsafe {
            (FUNCTIONS.Java_org_lwjgl_util_vma_Vma_nvmaUnmapMemory)(
                NO_ENV, NO_ENV, allocator, allocation,
            )
        }
    }
}
