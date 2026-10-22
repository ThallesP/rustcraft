//! com/mojang/renderpearl/api/device/DeviceType.java

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    OTHER,
    INTEGRATED,
    DISCRETE,
    VIRTUAL,
    CPU,
}
