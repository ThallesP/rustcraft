//! com/mojang/renderpearl/backend/vulkan/init/VulkanFeature.java
//!
//! One `VkBool32` feature of a features struct, read and written at its offset in the struct
//! found (or made) in a `VkPhysicalDeviceFeatures2`'s pNext chain.

use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_p_next_struct::{
    PNextArena, VulkanPNextStruct,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VulkanFeature {
    pub r#struct: &'static VulkanPNextStruct,
    pub name: &'static str,
    pub offset: usize,
}

impl VulkanFeature {
    /// `new VulkanFeature(struct, name)`: the member's offset in the struct. Panics where Java
    /// throws `IllegalArgumentException`.
    pub fn new(r#struct: &'static VulkanPNextStruct, name: &'static str) -> VulkanFeature {
        assert!(
            r#struct.name().contains("Features"),
            "Struct name \"{}\" does not contain \"Features\". All Vulkan features structs are expected to have \"Features\" in the name.",
            r#struct.name()
        );
        VulkanFeature {
            r#struct,
            name,
            offset: r#struct.field_offset(name),
        }
    }

    /// `get(long pNextChain)`: false where the chain has no such struct.
    ///
    /// # Safety
    ///
    /// `p_next_chain` is a chain of live structs.
    pub unsafe fn get(&self, p_next_chain: *mut u8) -> bool {
        // SAFETY: the caller's chain, and the member's offset in its struct.
        unsafe {
            let struct_addr = self.r#struct.find_struct_in_p_next_chain(p_next_chain);
            !struct_addr.is_null() && struct_addr.add(self.offset).cast::<u32>().read() != 0
        }
    }

    /// `set(long pNextChain, boolean, MemoryStack)`: the member set, in the chain's struct, made
    /// if it isn't there.
    ///
    /// # Safety
    ///
    /// `p_next_chain` is a chain of live structs whose blocks are in `arena` or outlive it.
    pub unsafe fn set(&self, p_next_chain: *mut u8, value: bool, arena: &mut PNextArena) {
        // SAFETY: as above.
        unsafe {
            let struct_addr = self
                .r#struct
                .find_or_create_struct_in_p_next_chain(p_next_chain, arena);
            struct_addr
                .add(self.offset)
                .cast::<u32>()
                .write(u32::from(value));
        }
    }
}

impl fmt::Display for VulkanFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.r#struct, self.name)
    }
}
