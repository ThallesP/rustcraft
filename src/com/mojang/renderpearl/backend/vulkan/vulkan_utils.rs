//! com/mojang/renderpearl/backend/vulkan/VulkanUtils.java
//!
//! Results, bits, the devices the game won't use (Intel's Windows drivers on some GPUs), and the
//! device's extensions and features. `crashIfFailure` panics where Java throws, with the queue
//! checkpoints a lost device left (none: the vendors' checkpoint extensions aren't ported,
//! `CheckpointExtension`).

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::checkpoint_extension::QueueCheckpoints;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_feature::VulkanFeature;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_p_next_struct::PNextArena;
use ash::vk;
use std::collections::HashSet;

pub struct VulkanUtils;

/// `VulkanUtils.DeviceUUID`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceUUID {
    pub driver_id: i32,
    pub vendor_id: i32,
    pub device_id: i32,
}

/// Intel's proprietary Windows driver (driver 14, vendor 0x8086) on these GPUs.
const KNOWN_PROBLEMATIC_DEVICE_IDS: [i32; 58] = [
    338, 341, 343, 346, 354, 358, 362, 1026, 1030, 1034, 1035, 1038, 1042, 1046, 1050, 1051, 1054,
    1058, 1062, 1066, 1067, 1070, 2562, 2566, 2570, 2571, 2574, 2578, 2582, 2586, 2587, 2590, 2594,
    2598, 2602, 2603, 2606, 3362, 3366, 3370, 3371, 3374, 3888, 3889, 3890, 3891, 5638, 5650, 5654,
    5662, 5666, 5670, 5674, 5675, 8880, 8881, 8882, 8883,
];

impl VulkanUtils {
    /// `KNOWN_PROBLEMATIC_DEVICES.contains(uuid)`.
    pub fn is_known_problematic(uuid: DeviceUUID) -> bool {
        uuid.driver_id == 14
            && uuid.vendor_id == 32902
            && KNOWN_PROBLEMATIC_DEVICE_IDS.contains(&uuid.device_id)
    }

    /// `throwIfFailure`: an error result as the exception.
    pub fn throw_if_failure(
        result: vk::Result,
        message: &str,
        reason: Reason,
    ) -> Result<(), BackendCreationException> {
        if result.as_raw() < 0 {
            return Err(BackendCreationException::new(
                format!(
                    "{}: {message}",
                    VulkanUtils::result_to_string(result.as_raw())
                ),
                reason,
            ));
        }
        Ok(())
    }

    /// `crashIfFailure`. Panics where Java throws: `GpuDeviceLossException` for a lost device,
    /// with its checkpoints, `IllegalStateException` for any other error.
    pub fn crash_if_failure(result: vk::Result, message: &str) {
        if result.as_raw() < 0 {
            let error = format!(
                "{}: {message}",
                VulkanUtils::result_to_string(result.as_raw())
            );
            if result == vk::Result::ERROR_DEVICE_LOST {
                panic!("{error}\n{}", VulkanUtils::format_checkpoints(&[]));
            }
            panic!("{error}");
        }
    }

    /// `crashIfFailure` of an `ash` call's result, handing back its value.
    pub fn crash_if_err<T>(result: Result<T, vk::Result>, message: &str) -> T {
        result.unwrap_or_else(|error| {
            VulkanUtils::crash_if_failure(error, message);
            unreachable!("{message}: {error:?} isn't an error")
        })
    }

    pub fn format_checkpoints(queue_checkpoints: &[QueueCheckpoints]) -> String {
        let mut result = String::new();
        for queue in queue_checkpoints {
            result.push_str(&format!("Queue 0x{:x}\n", queue.queue));
            for checkpoint in &queue.checkpoints {
                result.push_str(&format!(
                    " {} = {:?} {}\n",
                    VulkanUtils::pipeline_stage_to_string(checkpoint.stage),
                    checkpoint.r#type,
                    checkpoint.label
                ));
            }
        }
        result
    }

    pub fn pipeline_stage_to_string(pipeline_stage: i64) -> String {
        let name = match pipeline_stage {
            0 => "NONE",
            1 => "TOP_OF_PIPE",
            2 => "DRAW_INDIRECT",
            4 => "VERTEX_INPUT",
            8 => "VERTEX_SHADER",
            16 => "TESSELLATION_CONTROL_SHADER",
            32 => "TESSELLATION_EVALUATION_SHADER",
            64 => "GEOMETRY_SHADER",
            128 => "FRAGMENT_SHADER",
            256 => "EARLY_FRAGMENT_TESTS",
            512 => "LATE_FRAGMENT_TESTS",
            1024 => "COLOR_ATTACHMENT_OUTPUT",
            2048 => "COMPUTE_SHADER",
            4096 => "TRANSFER",
            8192 => "BOTTOM_OF_PIPE",
            16384 => "HOST",
            32768 => "ALL_GRAPHICS",
            65536 => "ALL_COMMANDS",
            131072 => "COMMAND_PREPROCESS",
            262144 => "CONDITIONAL_RENDERING",
            524288 => "TASK_SHADER",
            1048576 => "MESH_SHADER",
            2097152 => "RAY_TRACING_SHADER",
            4194304 => "FRAGMENT_SHADING_RATE_ATTACHMENT",
            8388608 => "FRAGMENT_DENSITY_PROCESS",
            16777216 => "TRANSFORM_FEEDBACK",
            33554432 => "ACCELERATION_STRUCTURE_BUILD",
            4294967296 => "COPY",
            8589934592 => "RESOLVE",
            17179869184 => "BLIT",
            34359738368 => "CLEAR",
            68719476736 => "INDEX_INPUT",
            137438953472 => "VERTEX_ATTRIBUTE_INPUT",
            274877906944 => "PRE_RASTERIZATION_SHADERS",
            _ => return format!("0x{pipeline_stage:x}"),
        };
        name.to_owned()
    }

    pub fn result_to_string(error: i32) -> String {
        let name = match error {
            -1000257000 => "VK_ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS",
            -1000161000 => "VK_ERROR_FRAGMENTATION",
            -1000072003 => "VK_ERROR_INVALID_EXTERNAL_HANDLE",
            -1000069000 => "VK_ERROR_OUT_OF_POOL_MEMORY",
            -1000001004 => "VK_ERROR_OUT_OF_DATE_KHR",
            -1000000001 => "VK_ERROR_NATIVE_WINDOW_IN_USE_KHR",
            -1000000000 => "VK_ERROR_SURFACE_LOST_KHR",
            -13 => "VK_ERROR_UNKNOWN",
            -12 => "VK_ERROR_FRAGMENTED_POOL",
            -11 => "VK_ERROR_FORMAT_NOT_SUPPORTED",
            -10 => "VK_ERROR_TOO_MANY_OBJECTS",
            -9 => "VK_ERROR_INCOMPATIBLE_DRIVER",
            -8 => "VK_ERROR_FEATURE_NOT_PRESENT",
            -7 => "VK_ERROR_EXTENSION_NOT_PRESENT",
            -6 => "VK_ERROR_LAYER_NOT_PRESENT",
            -5 => "VK_ERROR_MEMORY_MAP_FAILED",
            -4 => "VK_ERROR_DEVICE_LOST",
            -3 => "VK_ERROR_INITIALIZATION_FAILED",
            -2 => "VK_ERROR_OUT_OF_DEVICE_MEMORY",
            -1 => "VK_ERROR_OUT_OF_HOST_MEMORY",
            0 => "VK_SUCCESS",
            1 => "VK_NOT_READY",
            2 => "VK_TIMEOUT",
            3 => "VK_EVENT_SET",
            4 => "VK_EVENT_RESET",
            5 => "VK_INCOMPLETE",
            1000001003 => "VK_SUBOPTIMAL_KHR",
            // `Integer.toHexString`: the two's complement of a negative one.
            _ => return format!("0x{:x}", error as u32),
        };
        name.to_owned()
    }

    pub fn has_all_bits(bitfield: i64, bitmask: i64) -> bool {
        bitfield & bitmask == bitmask
    }

    pub fn has_any_bit(bitfield: i64, bitmask: i64) -> bool {
        bitfield & bitmask != 0
    }

    pub fn has_no_bit(bitfield: i64, bitmask: i64) -> bool {
        bitfield & bitmask == 0
    }

    /// `putArgb`: the vector's components in order, as Java's color is.
    pub fn put_argb(argb: &crate::org::joml::vector4f::Vector4f) -> vk::ClearColorValue {
        vk::ClearColorValue {
            float32: [argb.x, argb.y, argb.z, argb.w],
        }
    }

    /// The device's extensions' names (Java's `ObjectOpenHashSet`, whose order nothing reads).
    pub fn enumerate_extensions(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<HashSet<String>, BackendCreationException> {
        // SAFETY: the device is the instance's.
        let properties = unsafe { instance.enumerate_device_extension_properties(physical_device) }
            .map_err(|error| {
                BackendCreationException::new(
                    format!(
                        "{}: Failed to get extension properties",
                        VulkanUtils::result_to_string(error.as_raw())
                    ),
                    Reason::VULKAN_NO_DEVICE,
                )
            })?;
        Ok(properties
            .iter()
            .map(|extension| {
                extension.extension_name_as_c_str().map_or_else(
                    |_| String::new(),
                    |name| name.to_string_lossy().into_owned(),
                )
            })
            .collect())
    }

    /// `enumerateFeatures`: a `VkPhysicalDeviceFeatures2` chain with each feature's struct, as
    /// the device fills it in.
    pub fn enumerate_features(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        features: &[VulkanFeature],
        arena: &mut PNextArena,
    ) -> *mut u8 {
        let device_features = arena.features2();
        for required_device_feature in features {
            // SAFETY: the chain's blocks live in the arena.
            unsafe {
                required_device_feature
                    .r#struct
                    .find_or_create_struct_in_p_next_chain(device_features, arena);
            }
        }
        // SAFETY: the head is a VkPhysicalDeviceFeatures2 with a chain of features structs.
        unsafe {
            instance.get_physical_device_features2(
                physical_device,
                &mut *device_features.cast::<vk::PhysicalDeviceFeatures2<'_>>(),
            );
        }
        device_features
    }
}
