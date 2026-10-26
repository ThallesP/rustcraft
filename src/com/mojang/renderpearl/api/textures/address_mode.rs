//! com/mojang/renderpearl/api/textures/AddressMode.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AddressMode {
    REPEAT,
    CLAMP_TO_EDGE,
}

impl AddressMode {
    /// `values()`.
    pub const VALUES: [AddressMode; 2] = [AddressMode::REPEAT, AddressMode::CLAMP_TO_EDGE];
}
