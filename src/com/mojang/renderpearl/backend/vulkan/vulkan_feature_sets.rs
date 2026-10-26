//! com/mojang/renderpearl/backend/vulkan/VulkanFeatureSets.java
//!
//! What the game asks of a Vulkan device: the base it requires (dynamic rendering, push
//! descriptors, synchronization2, the vertex attribute divisor, the swapchain, and the features
//! its draws use), the portability subset where a device has it (MoltenVK), and what it takes if
//! it's there (wireframes, multi-draws, calibrated timestamps, a vendor's checkpoints). Java's
//! `ObjectOpenHashSet`s of them are lists in their declaration order here (see `FeatureSet`).

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::BackendCreationException;
use crate::com::mojang::renderpearl::backend::vulkan::init::feature_set::FeatureSet;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_feature::VulkanFeature;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_p_next_struct::VulkanPNextStruct;
use ash::vk;
use std::sync::LazyLock;

pub struct VulkanFeatureSets;

const VK10_FEATURES_STRUCT: &VulkanPNextStruct = &VulkanPNextStruct::VK_PHYSICAL_DEVICE_FEATURES2;
const VK11_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_VULKAN11_FEATURES;
const VK12_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_VULKAN12_FEATURES;
const SYNC2_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_SYNCHRONIZATION2_FEATURES;
const DYNAMIC_RENDERING_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_DYNAMIC_RENDERING_FEATURES;
const VERTEX_ATTRIB_DIVISOR_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
const PORTABILITY_SUBSET_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_PORTABILITY_SUBSET_FEATURES_KHR;
const MULTI_DRAW_FEATURES_STRUCT: &VulkanPNextStruct =
    &VulkanPNextStruct::VK_PHYSICAL_DEVICE_MULTI_DRAW_FEATURES_EXT;

pub static REQUIRED_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::new(
        "Minecraft base required",
        &[
            "VK_KHR_dynamic_rendering",
            "VK_KHR_push_descriptor",
            "VK_KHR_synchronization2",
            "VK_EXT_vertex_attribute_divisor",
            "VK_KHR_swapchain",
        ],
        vec![
            VulkanFeature::new(VK10_FEATURES_STRUCT, "multiDrawIndirect"),
            VulkanFeature::new(VK10_FEATURES_STRUCT, "drawIndirectFirstInstance"),
            VulkanFeature::new(VK10_FEATURES_STRUCT, "samplerAnisotropy"),
            VulkanFeature::new(VK11_FEATURES_STRUCT, "shaderDrawParameters"),
            VulkanFeature::new(VK12_FEATURES_STRUCT, "timelineSemaphore"),
            VulkanFeature::new(VK12_FEATURES_STRUCT, "hostQueryReset"),
            VulkanFeature::new(SYNC2_FEATURES_STRUCT, "synchronization2"),
            VulkanFeature::new(DYNAMIC_RENDERING_FEATURES_STRUCT, "dynamicRendering"),
            VulkanFeature::new(
                VERTEX_ATTRIB_DIVISOR_FEATURES_STRUCT,
                "vertexAttributeInstanceRateDivisor",
            ),
        ],
    )
});
pub static PORTABILITY_SUBSET_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::new(
        "Vulkan Portability Subset",
        &["VK_KHR_portability_subset"],
        vec![VulkanFeature::new(
            PORTABILITY_SUBSET_FEATURES_STRUCT,
            "triangleFans",
        )],
    )
});
pub static MULTI_DRAW_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::new(
        "EXT MultiDraw",
        &["VK_EXT_multi_draw"],
        vec![VulkanFeature::new(MULTI_DRAW_FEATURES_STRUCT, "multiDraw")],
    )
});
pub static AMD_BUFFER_MARKER_FEATURESET: LazyLock<FeatureSet> =
    LazyLock::new(|| FeatureSet::new("AMD Buffer Marker", &["VK_AMD_buffer_marker"], Vec::new()));
pub static NV_DIAGNOSTIC_CHECKPOINT_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::with_condition(
        "NV Diagnostic Checkpoint",
        &["VK_NV_device_diagnostic_checkpoints"],
        Vec::new(),
        vec![no_amd_buffer_marker],
    )
});
pub static WIREFRAME_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::new(
        "Wireframe",
        &[],
        vec![VulkanFeature::new(VK10_FEATURES_STRUCT, "fillModeNonSolid")],
    )
});
pub static CALIBRATED_TIMESTAMP_FEATURESET: LazyLock<FeatureSet> = LazyLock::new(|| {
    FeatureSet::new(
        "Calibrated Timestamps",
        &["VK_EXT_calibrated_timestamps"],
        Vec::new(),
    )
});

/// NV's checkpoints only where AMD's buffer markers aren't.
fn no_amd_buffer_marker(
    instance: &ash::Instance,
    device: vk::PhysicalDevice,
) -> Result<bool, BackendCreationException> {
    Ok(!AMD_BUFFER_MARKER_FEATURESET.is_supported_by(instance, device)?)
}

impl VulkanFeatureSets {
    pub fn required_feature_sets() -> Vec<&'static FeatureSet> {
        vec![&REQUIRED_FEATURESET]
    }

    pub fn required_if_extensions_available_feature_sets() -> Vec<&'static FeatureSet> {
        vec![&PORTABILITY_SUBSET_FEATURESET]
    }

    pub fn optional_feature_sets() -> Vec<&'static FeatureSet> {
        vec![
            &CALIBRATED_TIMESTAMP_FEATURESET,
            &WIREFRAME_FEATURESET,
            &NV_DIAGNOSTIC_CHECKPOINT_FEATURESET,
            &AMD_BUFFER_MARKER_FEATURESET,
            &MULTI_DRAW_FEATURESET,
        ]
    }
}
