//! com/mojang/renderpearl/api/device/DeviceInfo.java
//!
//! `underlyingExtensions` is a Java `HashSet` (unmodifiable), whose order vanilla logs the
//! extensions in, so it's a list in that order.

use crate::com::mojang::renderpearl::api::device::device_features::DeviceFeatures;
use crate::com::mojang::renderpearl::api::device::device_limits::DeviceLimits;
use crate::com::mojang::renderpearl::api::device::device_type::DeviceType;
use crate::com::mojang::renderpearl::api::device::hints_and_workarounds::HintsAndWorkarounds;

/// What a device is and can do, which the game and the shader compiler ask.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceInfo {
    pub name: String,
    pub vendor_name: String,
    pub driver_info: String,
    pub is_z_zero_to_one: bool,
    pub backend_name: String,
    pub timestamp_period: f32,
    pub limits: DeviceLimits,
    pub features: DeviceFeatures,
    pub underlying_extensions: Vec<String>,
    pub hints_and_workarounds: HintsAndWorkarounds,
    pub device_type: DeviceType,
}
