//! com/mojang/renderpearl/backend/vulkan/init/VulkanPNextStruct.java
//!
//! A features struct that can sit in a `VkPhysicalDeviceFeatures2`'s pNext chain: its `sType`
//! and size, which Java finds through LWJGL's class by reflection and the port has from `ash`'s
//! struct, and a feature's offset in it (`fieldOffset`, a `VkBool32` member by its name; the
//! Vulkan 1.0 features sit in `VkPhysicalDeviceFeatures2`'s `features`). The chain is raw memory,
//! as in Java: each struct a zeroed block of its size, its `sType` set, linked in right after the
//! chain's head. The blocks live in a `PNextArena`, Java's `MemoryStack`.

use ash::vk;
use std::mem::offset_of;

/// The `sType` and `pNext` every chained struct starts with, `VkBaseOutStructure`.
const OFFSET_STYPE: usize = offset_of!(vk::BaseOutStructure<'static>, s_type);
const OFFSET_PNEXT: usize = offset_of!(vk::BaseOutStructure<'static>, p_next);

pub struct VulkanPNextStruct {
    /// LWJGL's class's simple name, which `toString` prints.
    name: &'static str,
    s_type: vk::StructureType,
    struct_size: usize,
    /// Each `VkBool32` member's offset by its name.
    fields: fn(&str) -> Option<usize>,
}

/// The blocks a chain's structs live in, freed with it: Java's `MemoryStack`.
#[derive(Default)]
pub struct PNextArena {
    blocks: Vec<Box<[u64]>>,
}

impl PNextArena {
    /// A zeroed, 8-byte aligned block of `size` bytes (`ncalloc`).
    fn calloc(&mut self, size: usize) -> *mut u8 {
        let mut block = vec![0u64; size.div_ceil(8)].into_boxed_slice();
        let pointer = block.as_mut_ptr().cast::<u8>();
        self.blocks.push(block);
        pointer
    }

    /// A chain's head: a zeroed `VkPhysicalDeviceFeatures2` with its `sType`, `sType$Default`.
    pub fn features2(&mut self) -> *mut u8 {
        let head = self.calloc(size_of::<vk::PhysicalDeviceFeatures2<'static>>());
        // SAFETY: the block is the struct's size, and 8-byte aligned.
        unsafe { s_type_put(head, vk::StructureType::PHYSICAL_DEVICE_FEATURES_2) };
        head
    }
}

/// The structs' `VkBool32` members by name, `ash`'s fields at C's offsets.
macro_rules! bool_fields {
    ($struct:ty, $base:expr; $($name:literal => $field:ident),* $(,)?) => {
        |name: &str| match name {
            $($name => Some($base + offset_of!($struct, $field)),)*
            _ => None,
        }
    };
}

impl VulkanPNextStruct {
    pub const VK_PHYSICAL_DEVICE_FEATURES2: VulkanPNextStruct = VulkanPNextStruct {
        name: "VkPhysicalDeviceFeatures2",
        s_type: vk::StructureType::PHYSICAL_DEVICE_FEATURES_2,
        struct_size: size_of::<vk::PhysicalDeviceFeatures2<'static>>(),
        fields: bool_fields!(vk::PhysicalDeviceFeatures, offset_of!(vk::PhysicalDeviceFeatures2<'static>, features);
            "multiDrawIndirect" => multi_draw_indirect,
            "drawIndirectFirstInstance" => draw_indirect_first_instance,
            "samplerAnisotropy" => sampler_anisotropy,
            "fillModeNonSolid" => fill_mode_non_solid,
        ),
    };
    pub const VK_PHYSICAL_DEVICE_VULKAN11_FEATURES: VulkanPNextStruct = VulkanPNextStruct {
        name: "VkPhysicalDeviceVulkan11Features",
        s_type: vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_1_FEATURES,
        struct_size: size_of::<vk::PhysicalDeviceVulkan11Features<'static>>(),
        fields: bool_fields!(vk::PhysicalDeviceVulkan11Features<'static>, 0;
            "shaderDrawParameters" => shader_draw_parameters,
        ),
    };
    pub const VK_PHYSICAL_DEVICE_VULKAN12_FEATURES: VulkanPNextStruct = VulkanPNextStruct {
        name: "VkPhysicalDeviceVulkan12Features",
        s_type: vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_FEATURES,
        struct_size: size_of::<vk::PhysicalDeviceVulkan12Features<'static>>(),
        fields: bool_fields!(vk::PhysicalDeviceVulkan12Features<'static>, 0;
            "timelineSemaphore" => timeline_semaphore,
            "hostQueryReset" => host_query_reset,
        ),
    };
    pub const VK_PHYSICAL_DEVICE_SYNCHRONIZATION2_FEATURES: VulkanPNextStruct = VulkanPNextStruct {
        name: "VkPhysicalDeviceSynchronization2Features",
        s_type: vk::StructureType::PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
        struct_size: size_of::<vk::PhysicalDeviceSynchronization2Features<'static>>(),
        fields: bool_fields!(vk::PhysicalDeviceSynchronization2Features<'static>, 0;
            "synchronization2" => synchronization2,
        ),
    };
    pub const VK_PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES: VulkanPNextStruct =
        VulkanPNextStruct {
            name: "VkPhysicalDeviceDynamicRenderingFeatures",
            s_type: vk::StructureType::PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES,
            struct_size: size_of::<vk::PhysicalDeviceDynamicRenderingFeatures<'static>>(),
            fields: bool_fields!(vk::PhysicalDeviceDynamicRenderingFeatures<'static>, 0;
                "dynamicRendering" => dynamic_rendering,
            ),
        };
    pub const VK_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT: VulkanPNextStruct =
        VulkanPNextStruct {
            name: "VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT",
            s_type: vk::StructureType::PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT,
            struct_size: size_of::<vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT<'static>>(),
            fields: bool_fields!(vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT<'static>, 0;
                "vertexAttributeInstanceRateDivisor" => vertex_attribute_instance_rate_divisor,
            ),
        };
    pub const VK_PHYSICAL_DEVICE_PORTABILITY_SUBSET_FEATURES_KHR: VulkanPNextStruct =
        VulkanPNextStruct {
            name: "VkPhysicalDevicePortabilitySubsetFeaturesKHR",
            s_type: vk::StructureType::PHYSICAL_DEVICE_PORTABILITY_SUBSET_FEATURES_KHR,
            struct_size: size_of::<vk::PhysicalDevicePortabilitySubsetFeaturesKHR<'static>>(),
            fields: bool_fields!(vk::PhysicalDevicePortabilitySubsetFeaturesKHR<'static>, 0;
                "triangleFans" => triangle_fans,
            ),
        };
    pub const VK_PHYSICAL_DEVICE_MULTI_DRAW_FEATURES_EXT: VulkanPNextStruct = VulkanPNextStruct {
        name: "VkPhysicalDeviceMultiDrawFeaturesEXT",
        s_type: vk::StructureType::PHYSICAL_DEVICE_MULTI_DRAW_FEATURES_EXT,
        struct_size: size_of::<vk::PhysicalDeviceMultiDrawFeaturesEXT<'static>>(),
        fields: bool_fields!(vk::PhysicalDeviceMultiDrawFeaturesEXT<'static>, 0;
            "multiDraw" => multi_draw,
        ),
    };

    pub fn name(&self) -> &'static str {
        self.name
    }

    /// `fieldOffset(name)`. Panics where Java throws `IllegalArgumentException`.
    pub fn field_offset(&self, name: &str) -> usize {
        (self.fields)(name)
            .unwrap_or_else(|| panic!("Could not find field {name} in struct {}", self.name))
    }

    /// `findOrCreateStructInPNextChain`: the chain's struct of this type, or a new one linked in
    /// right after the head.
    ///
    /// # Safety
    ///
    /// `p_next_chain` is a chain of live structs whose blocks outlive the result.
    pub unsafe fn find_or_create_struct_in_p_next_chain(
        &self,
        p_next_chain: *mut u8,
        arena: &mut PNextArena,
    ) -> *mut u8 {
        // SAFETY: the caller's chain.
        unsafe {
            let found_struct = find_struct_in_p_next_chain(p_next_chain, self.s_type);
            if !found_struct.is_null() {
                return found_struct;
            }

            let new_struct = arena.calloc(self.struct_size);
            s_type_put(new_struct, self.s_type);
            p_next_put(new_struct, p_next(p_next_chain));
            p_next_put(p_next_chain, new_struct);
            new_struct
        }
    }

    /// `findStructInPNextChain`: the chain's struct of this type, or null.
    ///
    /// # Safety
    ///
    /// `p_next_chain` is null or a chain of live structs.
    pub unsafe fn find_struct_in_p_next_chain(&self, p_next_chain: *mut u8) -> *mut u8 {
        // SAFETY: the caller's chain.
        unsafe { find_struct_in_p_next_chain(p_next_chain, self.s_type) }
    }
}

/// Java's record compares its class, which its `sType` is.
impl PartialEq for VulkanPNextStruct {
    fn eq(&self, other: &VulkanPNextStruct) -> bool {
        self.s_type == other.s_type
    }
}

impl Eq for VulkanPNextStruct {}

impl std::fmt::Debug for VulkanPNextStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl std::fmt::Display for VulkanPNextStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// # Safety
///
/// `p_next_chain` is null or a chain of live structs.
unsafe fn find_struct_in_p_next_chain(
    mut p_next_chain: *mut u8,
    s_type: vk::StructureType,
) -> *mut u8 {
    // SAFETY: the caller's chain.
    unsafe {
        while !p_next_chain.is_null() {
            if s_type_get(p_next_chain) == s_type {
                return p_next_chain;
            }
            p_next_chain = p_next(p_next_chain);
        }
    }
    std::ptr::null_mut()
}

unsafe fn s_type_get(pointer: *const u8) -> vk::StructureType {
    // SAFETY: a chained struct starts with its sType.
    unsafe { pointer.add(OFFSET_STYPE).cast::<vk::StructureType>().read() }
}

unsafe fn s_type_put(pointer: *mut u8, value: vk::StructureType) {
    // SAFETY: as above.
    unsafe {
        pointer
            .add(OFFSET_STYPE)
            .cast::<vk::StructureType>()
            .write(value)
    }
}

unsafe fn p_next(pointer: *const u8) -> *mut u8 {
    // SAFETY: a chained struct has its pNext after its sType.
    unsafe { pointer.add(OFFSET_PNEXT).cast::<*mut u8>().read() }
}

unsafe fn p_next_put(pointer: *mut u8, value: *mut u8) {
    // SAFETY: as above.
    unsafe { pointer.add(OFFSET_PNEXT).cast::<*mut u8>().write(value) }
}
