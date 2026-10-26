//! com/mojang/renderpearl/backend/vulkan/init/FeatureSet.java
//!
//! A named set of device extensions and features, with a condition on the device. Java's sets
//! are `Set.of`s and fastutil's `ObjectOpenHashSet`s, whose order changes from run to run (a
//! `Set.of`'s salt, a record holding a lambda hashed by identity), and only decides the order of
//! the logs and of the extensions the device is made with; the device's extension list, which
//! the game logs, is a Java `HashSet` of them, whose order is its own (`VulkanDevice`). So they're
//! lists here, in their declaration order. `supportedExtensions` and `supportedFeatures` aren't
//! ported: nothing calls them.

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::BackendCreationException;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_feature::VulkanFeature;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_p_next_struct::PNextArena;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk;
use std::collections::HashSet;

/// `FeatureSet.Condition`: a test of the device, all of whose tests have to hold; none is
/// `IDENTITY_CONDITION`.
pub type Condition =
    fn(&ash::Instance, vk::PhysicalDevice) -> Result<bool, BackendCreationException>;

#[derive(Clone, Debug)]
pub struct FeatureSet {
    pub name: &'static str,
    pub extensions: Vec<&'static str>,
    pub features: Vec<VulkanFeature>,
    pub condition: Vec<Condition>,
}

impl PartialEq for FeatureSet {
    /// A record's `equals`; conditions compare by the functions they are, like Java's lambdas.
    fn eq(&self, other: &FeatureSet) -> bool {
        self.name == other.name
            && self.extensions == other.extensions
            && self.features == other.features
            && self.condition.len() == other.condition.len()
            && self
                .condition
                .iter()
                .zip(&other.condition)
                .all(|(a, b)| std::ptr::fn_addr_eq(*a, *b))
    }
}

impl FeatureSet {
    pub fn new(
        name: &'static str,
        extensions: &[&'static str],
        features: Vec<VulkanFeature>,
    ) -> FeatureSet {
        FeatureSet::with_condition(name, extensions, features, Vec::new())
    }

    pub fn with_condition(
        name: &'static str,
        extensions: &[&'static str],
        features: Vec<VulkanFeature>,
        condition: Vec<Condition>,
    ) -> FeatureSet {
        FeatureSet {
            name,
            extensions: extensions.to_vec(),
            features,
            condition,
        }
    }

    /// `new FeatureSet(name, others)`: every extension and feature of the others, once, and all
    /// their conditions (`Condition.reduce`).
    pub fn composite_of(name: &'static str, others: &[&FeatureSet]) -> FeatureSet {
        let mut condition: Vec<Condition> = Vec::new();
        let mut extensions = Vec::new();
        let mut features = Vec::new();
        for other in others {
            for test in &other.condition {
                if !condition
                    .iter()
                    .any(|known| std::ptr::fn_addr_eq(*known, *test))
                {
                    condition.push(*test);
                }
            }
            for extension in &other.extensions {
                if !extensions.contains(extension) {
                    extensions.push(*extension);
                }
            }
            for feature in &other.features {
                if !features.contains(feature) {
                    features.push(*feature);
                }
            }
        }
        FeatureSet {
            name,
            extensions,
            features,
            condition,
        }
    }

    /// `contains(other)`: every extension and feature of the other.
    pub fn contains(&self, other: &FeatureSet) -> bool {
        other
            .extensions
            .iter()
            .all(|extension| self.extensions.contains(extension))
            && other
                .features
                .iter()
                .all(|feature| self.features.contains(feature))
    }

    pub fn check_condition(
        &self,
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> Result<bool, BackendCreationException> {
        for test in &self.condition {
            if !test(instance, device)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `isSupported(VkPhysicalDevice)`.
    pub fn is_supported_by(
        &self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<bool, BackendCreationException> {
        let extensions = VulkanUtils::enumerate_extensions(instance, physical_device)?;
        Ok(self.is_supported(instance, physical_device, &extensions))
    }

    /// `isSupported(VkPhysicalDevice, Set)`.
    pub fn is_supported(
        &self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device_supported_extensions: &HashSet<String>,
    ) -> bool {
        self.extensions
            .iter()
            .all(|extension| device_supported_extensions.contains(*extension))
            && self.all_features_supported(instance, physical_device)
    }

    pub fn all_features_supported(
        &self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        if !self.features.is_empty() {
            let mut arena = PNextArena::default();
            let supported_features = VulkanUtils::enumerate_features(
                instance,
                physical_device,
                &self.features,
                &mut arena,
            );
            for feature in &self.features {
                // SAFETY: the chain lives in the arena.
                if !unsafe { feature.get(supported_features) } {
                    return false;
                }
            }
        }
        true
    }

    pub fn unsupported_extensions(
        &self,
        device_supported_extensions: &HashSet<String>,
    ) -> Vec<&'static str> {
        self.extensions
            .iter()
            .copied()
            .filter(|extension| !device_supported_extensions.contains(*extension))
            .collect()
    }

    pub fn unsupported_features(
        &self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Vec<VulkanFeature> {
        let mut arena = PNextArena::default();
        let features_chain =
            VulkanUtils::enumerate_features(instance, physical_device, &self.features, &mut arena);
        self.features
            .iter()
            .copied()
            // SAFETY: the chain lives in the arena.
            .filter(|feature| !unsafe { feature.get(features_chain) })
            .collect()
    }
}
