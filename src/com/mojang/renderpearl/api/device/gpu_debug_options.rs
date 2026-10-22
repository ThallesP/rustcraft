//! com/mojang/renderpearl/api/device/GpuDebugOptions.java

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuDebugOptions {
    pub log_level: i32,
    pub synchronous_logs: bool,
    pub use_labels: bool,
    pub use_validation_layers: bool,
}
