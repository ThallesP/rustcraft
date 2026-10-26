//! com/mojang/renderpearl/backend/vulkan/VulkanPhysicalDevice.java
//!
//! A device's extensions, properties (with Vulkan 1.1's, the driver's and the multi-draw's) and
//! queues: the first family that does graphics and compute and can present to a window, and the
//! families with spare queues for compute and transfers, the ones with the fewest other abilities.
//! `queueFamilyCreateInfoMap`, fastutil's `Int2IntArrayMap`, is each family with how many of its
//! queues the device makes, in the order the families came. Its features (`vkPhysicalDeviceFeatures`)
//! are fetched in Java and never read, so they aren't here.

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::api::device::device_type::DeviceType;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::org::lwjgl::sdl::sdl_vulkan::SDLVulkan;
use ash::vk;

pub struct VulkanPhysicalDevice {
    vk_physical_device: vk::PhysicalDevice,
    vk_device_extensions: Vec<String>,
    vk_physical_device_properties: vk::PhysicalDeviceProperties,
    vk_physical_device_vulkan11_properties: vk::PhysicalDeviceVulkan11Properties<'static>,
    vk_physical_device_driver_properties: vk::PhysicalDeviceDriverProperties<'static>,
    vk_physical_device_multi_draw_properties: vk::PhysicalDeviceMultiDrawPropertiesEXT<'static>,
    queue_family_create_info_map: Vec<(i32, i32)>,
    graphics_queue_family_and_index: Option<(i32, i32)>,
    compute_queue_family_and_index: Option<(i32, i32)>,
    transfer_queue_family_and_index: Option<(i32, i32)>,
}

impl VulkanPhysicalDevice {
    pub fn new(
        instance: &ash::Instance,
        vk_physical_device: vk::PhysicalDevice,
    ) -> Result<VulkanPhysicalDevice, BackendCreationException> {
        // SAFETY: the device is the instance's.
        let extensions =
            unsafe { instance.enumerate_device_extension_properties(vk_physical_device) }.map_err(
                |error| {
                    BackendCreationException::new(
                        format!(
                            "{}: Failed to get extension properties",
                            VulkanUtils::result_to_string(error.as_raw())
                        ),
                        Reason::VULKAN_NO_DEVICE,
                    )
                },
            )?;
        let vk_device_extensions: Vec<String> = extensions
            .iter()
            .filter_map(|extension| extension.extension_name_as_c_str().ok())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();

        let mut driver_properties = vk::PhysicalDeviceDriverProperties::default();
        let mut vulkan11_properties = vk::PhysicalDeviceVulkan11Properties::default();
        let mut multi_draw_properties = vk::PhysicalDeviceMultiDrawPropertiesEXT::default();
        let has_multi_draw = vk_device_extensions
            .iter()
            .any(|name| name == "VK_EXT_multi_draw");
        // LWJGL's `pNext(struct)` puts each struct first in the chain.
        let mut properties = vk::PhysicalDeviceProperties2::default()
            .push_next(&mut driver_properties)
            .push_next(&mut vulkan11_properties);
        if has_multi_draw {
            properties = properties.push_next(&mut multi_draw_properties);
        }
        // SAFETY: the chain's structs live for the call.
        unsafe { instance.get_physical_device_properties2(vk_physical_device, &mut properties) };
        let vk_physical_device_properties = properties.properties;

        // SAFETY: as above.
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(vk_physical_device) };
        let mut graphics_queue_family = -1;
        let mut compute_queue_family = -1;
        let mut transfer_queue_family = -1;
        let mut compute_queue_family_bits = -1i32;
        let mut transfer_queue_family_bits = -1i32;
        for (i, queue_family_properties) in queue_families.iter().enumerate() {
            let i = i as i32;
            let mut family_used_queues = 0;
            let queue_flags = queue_family_properties.queue_flags.as_raw() as i32;
            if graphics_queue_family == -1
                && VulkanUtils::has_all_bits(i64::from(queue_flags), 3)
                && SDLVulkan::SDL_Vulkan_GetPresentationSupport(
                    instance.handle(),
                    vk_physical_device,
                    i as u32,
                )
            {
                graphics_queue_family = i;
                family_used_queues += 1;
            }

            let queue_count = queue_family_properties.queue_count as i32;
            if queue_count > family_used_queues {
                if VulkanUtils::has_all_bits(i64::from(queue_flags), 2)
                    && (compute_queue_family == -1
                        || queue_flags.count_ones() <= compute_queue_family_bits.count_ones())
                {
                    compute_queue_family = i;
                    compute_queue_family_bits = queue_flags;
                    family_used_queues += 1;
                }

                if queue_count > family_used_queues
                    && VulkanUtils::has_any_bit(i64::from(queue_flags), 7)
                    && (transfer_queue_family == -1
                        || queue_flags.count_ones() <= transfer_queue_family_bits.count_ones())
                {
                    transfer_queue_family = i;
                    transfer_queue_family_bits = queue_flags;
                }
            }
        }

        // Int2IntArrayMap.put hands back the family's count before, 0 for a new one.
        let mut family_map: Vec<(i32, i32)> = Vec::new();
        let mut put_next = |family: i32| -> i32 {
            match family_map.iter_mut().find(|(key, _)| *key == family) {
                Some((_, count)) => {
                    *count += 1;
                    *count - 1
                }
                None => {
                    family_map.push((family, 1));
                    0
                }
            }
        };
        let graphics_queue_index = put_next(graphics_queue_family);
        let compute_queue_index = put_next(compute_queue_family);
        let transfer_queue_index = put_next(transfer_queue_family);
        family_map.retain(|(family, _)| *family != -1);

        let pair = |family: i32, index: i32| (family != -1).then_some((family, index));
        Ok(VulkanPhysicalDevice {
            vk_physical_device,
            vk_device_extensions,
            vk_physical_device_properties,
            vk_physical_device_vulkan11_properties: vulkan11_properties.next_null(),
            vk_physical_device_driver_properties: driver_properties.next_null(),
            vk_physical_device_multi_draw_properties: multi_draw_properties.next_null(),
            queue_family_create_info_map: family_map,
            graphics_queue_family_and_index: pair(graphics_queue_family, graphics_queue_index),
            compute_queue_family_and_index: pair(compute_queue_family, compute_queue_index),
            transfer_queue_family_and_index: pair(transfer_queue_family, transfer_queue_index),
        })
    }

    pub fn device_name(&self) -> String {
        self.vk_physical_device_properties
            .device_name_as_c_str()
            .map_or_else(
                |_| String::new(),
                |name| name.to_string_lossy().into_owned(),
            )
    }

    pub fn vendor_name(&self) -> String {
        let vendor_id = self.vk_physical_device_properties.vendor_id as i32;
        let name = match vendor_id {
            4098 | 4130 => "AMD",
            4112 => "IMGTEC",
            4203 => "APPLE",
            4318 | 4818 => "NVIDIA",
            5045 => "ARM",
            5140 => "MICROSOFT",
            5348 => "BROADCOM",
            5772 | 6091 | 6505 | 20803 => "QUALCOMM",
            32902 => "INTEL",
            // `String.format("0x%x")`: the int's unsigned hex.
            _ => return format!("0x{:x}", vendor_id as u32),
        };
        name.to_owned()
    }

    pub fn vk_physical_device(&self) -> vk::PhysicalDevice {
        self.vk_physical_device
    }

    pub fn vk_physical_device_properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.vk_physical_device_properties
    }

    pub fn vk_physical_device_vulkan11_properties(
        &self,
    ) -> &vk::PhysicalDeviceVulkan11Properties<'static> {
        &self.vk_physical_device_vulkan11_properties
    }

    pub fn vk_physical_device_driver_properties(
        &self,
    ) -> &vk::PhysicalDeviceDriverProperties<'static> {
        &self.vk_physical_device_driver_properties
    }

    pub fn vk_physical_device_multi_draw_properties_ext(
        &self,
    ) -> &vk::PhysicalDeviceMultiDrawPropertiesEXT<'static> {
        &self.vk_physical_device_multi_draw_properties
    }

    pub fn has_device_extension(&self, name: &str) -> bool {
        self.vk_device_extensions
            .iter()
            .any(|extension| extension == name)
    }

    pub fn queue_family_create_info_map(&self) -> &[(i32, i32)] {
        &self.queue_family_create_info_map
    }

    pub fn graphics_queue_family_and_index(&self) -> Option<(i32, i32)> {
        self.graphics_queue_family_and_index
    }

    pub fn compute_queue_family_and_index(&self) -> Option<(i32, i32)> {
        self.compute_queue_family_and_index
    }

    pub fn transfer_queue_family_and_index(&self) -> Option<(i32, i32)> {
        self.transfer_queue_family_and_index
    }

    fn get_standard_encoding_version(version: u32) -> String {
        let major = version >> 22 & 127;
        let minor = version >> 12 & 1023;
        let patch = version & 4095;
        format!("{major}.{minor}.{patch}")
    }

    /// `driverInfo()`: the Vulkan version, the driver's name and its info.
    pub fn driver_info(&self) -> String {
        let api_version = self.vk_physical_device_properties.api_version;
        let driver = &self.vk_physical_device_driver_properties;
        let text = |name: Result<&std::ffi::CStr, _>| {
            name.map_or_else(
                |_| String::new(),
                |name: &std::ffi::CStr| name.to_string_lossy().into_owned(),
            )
        };
        format!(
            "{} {} {}",
            VulkanPhysicalDevice::get_standard_encoding_version(api_version),
            text(driver.driver_name_as_c_str()),
            text(driver.driver_info_as_c_str())
        )
    }

    pub fn device_type(&self) -> DeviceType {
        match self.vk_physical_device_properties.device_type {
            vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::INTEGRATED,
            vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DISCRETE,
            vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VIRTUAL,
            vk::PhysicalDeviceType::CPU => DeviceType::CPU,
            _ => DeviceType::OTHER,
        }
    }
}

/// A struct read out of a chain, its pointer to the next one dropped.
trait NextNull {
    fn next_null(self) -> Self;
}

macro_rules! next_null {
    ($($struct:ident),*) => {
        $(impl NextNull for vk::$struct<'_> {
            fn next_null(self) -> Self {
                let mut copy = self;
                copy.p_next = std::ptr::null_mut();
                copy
            }
        })*
    };
}
next_null!(
    PhysicalDeviceVulkan11Properties,
    PhysicalDeviceDriverProperties,
    PhysicalDeviceMultiDrawPropertiesEXT
);
