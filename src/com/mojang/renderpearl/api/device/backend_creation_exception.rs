//! com/mojang/renderpearl/api/device/BackendCreationException.java

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendCreationException {
    message: String,
    reason: Reason,
    missing_capabilities: Vec<String>,
}

impl BackendCreationException {
    pub fn new(message: String, reason: Reason) -> BackendCreationException {
        BackendCreationException {
            message,
            reason,
            missing_capabilities: Vec::new(),
        }
    }

    /// `BackendCreationException(message, reason, missingCapabilities)`.
    pub fn with_missing_capabilities(
        message: String,
        reason: Reason,
        missing_capabilities: Vec<String>,
    ) -> BackendCreationException {
        BackendCreationException {
            message,
            reason,
            missing_capabilities,
        }
    }

    pub fn get_reason(&self) -> Reason {
        self.reason
    }

    pub fn get_missing_capabilities(&self) -> &[String] {
        &self.missing_capabilities
    }
}

impl fmt::Display for BackendCreationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BackendCreationException {}

/// `BackendCreationException.Reason`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    PLATFORM_ERROR,
    VULKAN_LOADER_MISSING,
    VULKAN_INSTANCE_CREATION_FAILED,
    VULKAN_NO_DEVICE,
    VULKAN_KNOWN_PROBLEMATIC,
    VULKAN_DEVICE_VERSION_TOO_LOW,
    VULKAN_NO_GRAPHICS_QUEUE,
    VULKAN_MISSING_EXTENSION,
    VULKAN_MISSING_FEATURE,
    OPENGL_MISSING,
    OTHER,
}

impl Reason {
    pub fn display_name(self) -> &'static str {
        match self {
            Reason::PLATFORM_ERROR => "platform_error",
            Reason::VULKAN_LOADER_MISSING => "vulkan_loader_missing",
            Reason::VULKAN_INSTANCE_CREATION_FAILED => "vulkan_instance_creation_failed",
            Reason::VULKAN_NO_DEVICE => "vulkan_no_device",
            Reason::VULKAN_KNOWN_PROBLEMATIC => "vulkan_known_problematic",
            Reason::VULKAN_DEVICE_VERSION_TOO_LOW => "vulkan_device_version_too_low",
            Reason::VULKAN_NO_GRAPHICS_QUEUE => "vulkan_no_graphics_queue",
            Reason::VULKAN_MISSING_EXTENSION => "vulkan_missing_extension",
            Reason::VULKAN_MISSING_FEATURE => "vulkan_missing_feature",
            Reason::OPENGL_MISSING => "opengl_missing",
            Reason::OTHER => "other",
        }
    }
}
